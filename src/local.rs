use anyhow::{Result, bail};
use plist::{Dictionary, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub const MODERN_MAIN_LUA: &str = r#"-- Modern

-- Use this function to perform your initial setup
function setup()
    print("Hello World!")
end

-- This function gets called once every frame
function draw()
    -- This sets a dark background color 
    background(40, 40, 50)

    -- This sets the line thickness
    style.strokeWidth(5)

    -- Do your drawing here
    
end
"#;

pub fn resolve_local_project_path(name: &str, folder: bool) -> Result<PathBuf> {
    let path = expand_user_path(name);
    let resolved = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()?.join(path)
    };
    let resolved = resolved.canonicalize().unwrap_or(resolved);

    if folder
        || resolved
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("codea"))
            .unwrap_or(false)
    {
        Ok(resolved)
    } else {
        Ok(resolved.with_extension("codea"))
    }
}

pub fn create_local_project(name: &str, template: Option<&str>, folder: bool) -> Result<PathBuf> {
    validate_template(template)?;
    let destination = resolve_local_project_path(name, folder)?;
    ensure_empty_project_directory(&destination)?;

    fs::write(destination.join("Main.lua"), MODERN_MAIN_LUA)?;

    let mut dict = Dictionary::new();
    dict.insert(
        "Buffer Order".to_string(),
        Value::Array(vec![Value::String("Main".to_string())]),
    );
    dict.insert(
        "Runtime Type".to_string(),
        Value::String("modern".to_string()),
    );
    Value::Dictionary(dict).to_file_xml(destination.join("Info.plist"))?;

    Ok(destination)
}

fn validate_template(template: Option<&str>) -> Result<()> {
    if let Some(template) = template {
        if !template.trim().eq_ignore_ascii_case("modern") {
            bail!("Only the Modern template is supported for local project creation.");
        }
    }
    Ok(())
}

fn ensure_empty_project_directory(destination: &Path) -> Result<()> {
    if destination.exists() {
        if !destination.is_dir() {
            bail!(
                "Destination already exists and is not a directory: {}",
                destination.display()
            );
        }
        let visible_contents = fs::read_dir(destination)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name() != ".DS_Store")
            .collect::<Vec<_>>();
        if !visible_contents.is_empty() {
            bail!(
                "Destination already exists and is not empty: {}",
                destination.display()
            );
        }
    } else {
        fs::create_dir_all(destination)?;
    }
    Ok(())
}

fn expand_user_path(input: &str) -> PathBuf {
    if input == "~" {
        return home_dir();
    }
    if let Some(stripped) = input.strip_prefix("~/") {
        return home_dir().join(stripped);
    }
    PathBuf::from(input)
}

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
