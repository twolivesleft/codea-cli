use anyhow::{Context, Result, anyhow};
#[cfg(target_os = "windows")]
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

pub const DEFAULT_PORT: u16 = 18513;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfileConfig {
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_port() -> u16 {
    DEFAULT_PORT
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigFile {
    #[serde(default)]
    pub profiles: BTreeMap<String, ProfileConfig>,
}

pub fn config_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(project_dirs) = ProjectDirs::from("com", "twolivesleft", "codea") {
            return project_dirs.config_dir().to_path_buf();
        }
    }

    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".codea")
}

pub fn config_file() -> PathBuf {
    config_dir().join("config.json")
}

pub fn load_config_file() -> Result<ConfigFile> {
    let path = config_file();
    if !path.exists() {
        return Ok(ConfigFile::default());
    }
    let text =
        fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    Ok(serde_json::from_str(&text)
        .with_context(|| format!("Failed to parse {}", path.display()))?)
}

pub fn load_profile(profile: &str) -> Result<Option<ProfileConfig>> {
    if let Some(host) = env::var_os("CODEA_HOST") {
        let port = env::var("CODEA_PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(DEFAULT_PORT);
        return Ok(Some(ProfileConfig {
            host: host.to_string_lossy().into_owned(),
            port,
        }));
    }

    let config = load_config_file()?;
    Ok(config.profiles.get(profile).cloned())
}

pub fn save_profile(profile: &str, host: &str, port: u16) -> Result<()> {
    let dir = config_dir();
    fs::create_dir_all(&dir).with_context(|| format!("Failed to create {}", dir.display()))?;

    let mut config = load_config_file()?;
    config.profiles.insert(
        profile.to_string(),
        ProfileConfig {
            host: host.to_string(),
            port,
        },
    );

    let text = serde_json::to_string_pretty(&config)?;
    fs::write(config_file(), text)?;
    Ok(())
}

pub fn clear_profile(profile: &str) -> Result<bool> {
    let path = config_file();
    if !path.exists() {
        return Ok(false);
    }

    let mut config = load_config_file()?;
    let removed = config.profiles.remove(profile).is_some();
    if !removed {
        return Ok(false);
    }

    if config.profiles.is_empty() {
        fs::remove_file(&path).with_context(|| format!("Failed to remove {}", path.display()))?;
    } else {
        let text = serde_json::to_string_pretty(&config)?;
        fs::write(&path, text).with_context(|| format!("Failed to write {}", path.display()))?;
    }

    Ok(true)
}

pub fn require_profile(profile: &str) -> Result<ProfileConfig> {
    load_profile(profile)?.ok_or_else(|| {
        anyhow!(
            "No device configured. Run 'codea discover' or 'codea configure' first.\nOr set CODEA_HOST (and optionally CODEA_PORT) environment variables."
        )
    })
}

pub fn resolve_status_source(profile: &str) -> Result<Option<(String, u16, String)>> {
    if let Some(host) = env::var_os("CODEA_HOST") {
        let port = env::var("CODEA_PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(DEFAULT_PORT);
        return Ok(Some((
            host.to_string_lossy().into_owned(),
            port,
            "environment variables".to_string(),
        )));
    }

    let path = config_file();
    if !path.exists() {
        return Ok(None);
    }

    let text =
        fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    let raw: Value = serde_json::from_str(&text)?;
    let profiles = raw
        .get("profiles")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    if let Some(value) = profiles.get(profile) {
        let host = value
            .get("host")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("Profile '{}' is missing host", profile))?
            .to_string();
        let port = value
            .get("port")
            .and_then(Value::as_u64)
            .and_then(|n| u16::try_from(n).ok())
            .unwrap_or(DEFAULT_PORT);
        return Ok(Some((host, port, path.display().to_string())));
    }

    if !profiles.is_empty() {
        let names = profiles.keys().cloned().collect::<Vec<_>>().join(", ");
        return Err(anyhow!(
            "Profile '{}' not found. Available profiles: {}",
            profile,
            names
        ));
    }

    Ok(None)
}
