mod config;
mod discover;
mod local;
mod mcp;

use anyhow::{Result, bail};
use clap::{ArgAction, Args, Parser, Subcommand};
use config::{DEFAULT_PORT, ProfileConfig};
use serde_json::{Value, json};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::config::{load_profile, require_profile, resolve_status_source, save_profile};
use crate::discover::discover_devices;
use crate::local::create_local_project;
use crate::mcp::{MCPClient, maybe_base64_text};

#[derive(Parser, Debug)]
#[command(name = "codea")]
#[command(about = "Codea CLI — connect to Codea on your device.")]
struct Cli {
    #[arg(long, global = true, action = ArgAction::SetTrue, help = "Wait for Codea to become reachable before running the command.")]
    wait: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Discover(DiscoverArgs),
    Configure(ConfigureArgs),
    Status(ProfileArg),
    Ls(ProfileArg),
    Pull(PullArgs),
    Push(PushArgs),
    Run(RunArgs),
    Stop(ProfileArg),
    Restart(ProfileArg),
    Pause(ProfileArg),
    Resume(ProfileArg),
    Paused(PausedArgs),
    #[command(name = "exec")]
    Exec(ExecArgs),
    Screenshot(ScreenshotArgs),
    #[command(name = "idle-timer")]
    IdleTimer(IdleTimerArgs),
    Logs(LogsArgs),
    #[command(name = "clear-logs")]
    ClearLogs(ProfileArg),
    New(NewArgs),
    Rename(RenameArgs),
    Move(MoveArgs),
    Delete(DeleteArgs),
    Collections(CollectionsCommand),
    Templates(TemplatesCommand),
    Deps(DepsCommand),
    Autocomplete(AutocompleteArgs),
    Runtime(RuntimeArgs),
    Doc(DocArgs),
    #[command(name = "search-doc")]
    SearchDoc(SearchDocArgs),
}

#[derive(Args, Debug)]
struct ProfileArg {
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Args, Debug)]
struct DiscoverArgs {
    #[arg(long, default_value_t = 5.0)]
    timeout: f64,
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Args, Debug)]
struct ConfigureArgs {
    #[arg(long)]
    host: String,
    #[arg(long, default_value_t = DEFAULT_PORT)]
    port: u16,
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Args, Debug)]
struct PullArgs {
    project: String,
    files: Vec<String>,
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
    #[arg(long, action = ArgAction::SetTrue)]
    no_deps: bool,
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Args, Debug)]
struct PushArgs {
    project: String,
    files: Vec<String>,
    #[arg(short = 'i', long = "input")]
    input_dir: Option<PathBuf>,
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Args, Debug)]
struct RunArgs {
    project: String,
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Args, Debug)]
struct PausedArgs {
    state: Option<String>,
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Args, Debug)]
struct ExecArgs {
    code: Option<String>,
    #[arg(long = "file")]
    lua_file: Option<PathBuf>,
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Args, Debug)]
struct ScreenshotArgs {
    #[arg(short = 'o', long, default_value = "screenshot.png")]
    output: PathBuf,
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Args, Debug)]
struct IdleTimerArgs {
    state: Option<String>,
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Args, Debug)]
struct LogsArgs {
    #[arg(long)]
    tail: Option<i64>,
    #[arg(long)]
    head: Option<i64>,
    #[arg(short = 'f', long, action = ArgAction::SetTrue)]
    follow: bool,
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Args, Debug)]
struct NewArgs {
    name: String,
    #[arg(long)]
    collection: Option<String>,
    #[arg(long, action = ArgAction::SetTrue)]
    cloud: bool,
    #[arg(long)]
    template: Option<String>,
    #[arg(long, action = ArgAction::SetTrue)]
    folder: bool,
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Args, Debug)]
struct RenameArgs {
    project: String,
    newname: String,
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Args, Debug)]
struct MoveArgs {
    project: String,
    collection: String,
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Args, Debug)]
struct DeleteArgs {
    project: String,
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Subcommand, Debug)]
enum CollectionsSubcommand {
    Ls(ProfileArg),
    New(CollectionNameArgs),
    Delete(CollectionNameArgs),
}

#[derive(Args, Debug)]
struct CollectionsCommand {
    #[command(subcommand)]
    command: CollectionsSubcommand,
}

#[derive(Args, Debug)]
struct CollectionNameArgs {
    name: String,
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Subcommand, Debug)]
enum TemplatesSubcommand {
    Ls(ProfileArg),
    Add(TemplateAddArgs),
    Remove(TemplateRemoveArgs),
}

#[derive(Args, Debug)]
struct TemplatesCommand {
    #[command(subcommand)]
    command: TemplatesSubcommand,
}

#[derive(Args, Debug)]
struct TemplateAddArgs {
    project: String,
    #[arg(long)]
    name: Option<String>,
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Args, Debug)]
struct TemplateRemoveArgs {
    name: String,
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Subcommand, Debug)]
enum DepsSubcommand {
    Ls(ProjectOnlyArgs),
    Available(ProjectOnlyArgs),
    Add(DependencyArgs),
    Remove(DependencyArgs),
}

#[derive(Args, Debug)]
struct DepsCommand {
    #[command(subcommand)]
    command: DepsSubcommand,
}

#[derive(Args, Debug)]
struct ProjectOnlyArgs {
    project: String,
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Args, Debug)]
struct DependencyArgs {
    project: String,
    dependency: String,
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Args, Debug)]
struct AutocompleteArgs {
    project: String,
    code: String,
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Args, Debug)]
struct RuntimeArgs {
    project: String,
    #[arg(value_name = "type")]
    runtime_type: Option<String>,
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Args, Debug)]
struct DocArgs {
    function_name: String,
    #[arg(long, action = ArgAction::SetTrue)]
    legacy: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    modern: bool,
    #[arg(long)]
    project: Option<String>,
    #[arg(long, default_value = "default")]
    profile: String,
}

#[derive(Args, Debug)]
struct SearchDocArgs {
    query: String,
    #[arg(long, action = ArgAction::SetTrue)]
    legacy: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    modern: bool,
    #[arg(long)]
    project: Option<String>,
    #[arg(long, default_value = "default")]
    profile: String,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Discover(args) => discover_command(args),
        Commands::Configure(args) => configure_command(args),
        Commands::Status(args) => status_command(&args.profile),
        Commands::Ls(args) => ls_command(&args.profile, cli.wait),
        Commands::Pull(args) => pull_command(args, cli.wait),
        Commands::Push(args) => push_command(args, cli.wait),
        Commands::Run(args) => run_command(args, cli.wait),
        Commands::Stop(args) => stop_command(&args.profile, cli.wait),
        Commands::Restart(args) => restart_command(&args.profile, cli.wait),
        Commands::Pause(args) => pause_command(&args.profile, cli.wait),
        Commands::Resume(args) => resume_command(&args.profile, cli.wait),
        Commands::Paused(args) => paused_command(args, cli.wait),
        Commands::Exec(args) => exec_command(args, cli.wait),
        Commands::Screenshot(args) => screenshot_command(args, cli.wait),
        Commands::IdleTimer(args) => idle_timer_command(args, cli.wait),
        Commands::Logs(args) => logs_command(args, cli.wait),
        Commands::ClearLogs(args) => clear_logs_command(&args.profile, cli.wait),
        Commands::New(args) => new_command(args, cli.wait),
        Commands::Rename(args) => rename_command(args, cli.wait),
        Commands::Move(args) => move_command(args, cli.wait),
        Commands::Delete(args) => delete_command(args, cli.wait),
        Commands::Collections(args) => collections_command(args, cli.wait),
        Commands::Templates(args) => templates_command(args, cli.wait),
        Commands::Deps(args) => deps_command(args, cli.wait),
        Commands::Autocomplete(args) => autocomplete_command(args, cli.wait),
        Commands::Runtime(args) => runtime_command(args, cli.wait),
        Commands::Doc(args) => doc_command(args, cli.wait),
        Commands::SearchDoc(args) => search_doc_command(args, cli.wait),
    }
}

fn client_for_profile(profile: &str, wait: bool) -> Result<MCPClient> {
    let config = require_profile(profile)?;
    if wait {
        wait_for_device(&config.host, config.port)?;
    }
    MCPClient::new(&config.host, config.port, 30)
}

fn wait_for_device(host: &str, port: u16) -> Result<()> {
    eprintln!("Waiting for Codea...");
    let start = Instant::now();
    loop {
        match MCPClient::new(host, port, 3).and_then(|mut c| c.initialize()) {
            Ok(_) => {
                eprintln!(
                    "Codea ready (waited {:.1}s).",
                    start.elapsed().as_secs_f64()
                );
                return Ok(());
            }
            Err(_) => std::thread::sleep(Duration::from_secs(1)),
        }
    }
}

fn discover_command(args: DiscoverArgs) -> Result<()> {
    println!("Scanning for Codea devices ({:.0}s)...", args.timeout);
    let devices = discover_devices(Duration::from_secs_f64(args.timeout))?;
    if devices.is_empty() {
        println!("No devices found. Make sure Codea Air Code server is running on your device.");
        return Ok(());
    }

    for (index, device) in devices.iter().enumerate() {
        println!(
            "  {}. {}  ({}:{})",
            index + 1,
            device.name,
            device.host,
            device.port
        );
    }

    let choice = if devices.len() == 1 {
        1
    } else {
        prompt_selection(devices.len())?
    };
    let device = &devices[choice - 1];
    save_profile(&args.profile, &device.host, device.port)?;
    println!(
        "Saved {}:{} as profile '{}'.",
        device.host, device.port, args.profile
    );
    Ok(())
}

fn configure_command(args: ConfigureArgs) -> Result<()> {
    save_profile(&args.profile, &args.host, args.port)?;
    println!(
        "Saved {}:{} as profile '{}'.",
        args.host, args.port, args.profile
    );
    Ok(())
}

fn status_command(profile: &str) -> Result<()> {
    let Some((host, port, source)) = resolve_status_source(profile)? else {
        println!("Not configured. Run 'codea discover' or 'codea configure'.");
        return Ok(());
    };

    println!("Host:    {host}");
    println!("Port:    {port}");
    println!("Profile: {profile}");
    println!("Source:  {source}");

    match MCPClient::new(&host, port, 30).and_then(|mut client| client.get_device_state()) {
        Ok(state) => {
            println!();
            let project_state = state.get("state").and_then(Value::as_str).unwrap_or("none");
            let project_name = state.get("project").and_then(Value::as_str);
            let local_path = state.get("localPath").and_then(Value::as_str);
            let idle_disabled = state
                .get("idleTimerDisabled")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let paused = state
                .get("paused")
                .and_then(Value::as_bool)
                .unwrap_or(false);

            if project_state == "running" {
                let mut label = if let Some(project_name) = project_name {
                    format!("Running: {project_name}")
                } else {
                    "Running".to_string()
                };
                if paused {
                    label.push_str(" (paused)");
                }
                println!("State:   {label}");
            } else {
                println!("State:   No project running");
            }
            if let Some(local_path) = local_path {
                println!("Local path: {local_path}");
            }
            println!(
                "Idle timer: {}",
                if idle_disabled {
                    "off (screen stays on)"
                } else {
                    "on"
                }
            );
        }
        Err(_) => println!("\nState:   (device unreachable)"),
    }
    Ok(())
}

fn ls_command(profile: &str, wait: bool) -> Result<()> {
    let mut client = client_for_profile(profile, wait)?;
    for path in client.list_projects()? {
        println!("{path}");
    }
    Ok(())
}

fn pull_command(args: PullArgs, wait: bool) -> Result<()> {
    let mut client = client_for_profile(&args.profile, wait)?;
    let pname = project_name(&args.project);
    let output_dir = args.output.unwrap_or_else(|| PathBuf::from(&pname));

    if !args.files.is_empty() {
        println!(
            "Pulling {} from '{}' → {}/",
            args.files.join(", "),
            pname,
            output_dir.display()
        );
        pull_project_files(&mut client, &args.project, &output_dir, &args.files, None)?;
        println!("Done.");
        return Ok(());
    }

    println!("Pulling '{}' → {}/", pname, output_dir.display());
    pull_project_files(&mut client, &args.project, &output_dir, &[], None)?;

    if !args.no_deps {
        let deps = client.list_dependencies(&args.project).unwrap_or_default();
        if !deps.is_empty() {
            println!("Dependencies: {}", deps.join(", "));
            let all_projects = client.list_projects()?;
            for dep in deps {
                let dep_name = dep.split(':').next_back().unwrap_or(&dep).to_string();
                if let Some(dep_project) = all_projects
                    .iter()
                    .find(|p| project_name(p).eq_ignore_ascii_case(&dep_name))
                {
                    let dep_dir = output_dir.join("Dependencies").join(&dep_name);
                    println!("Pulling dependency '{}' → {}/", dep_name, dep_dir.display());
                    pull_project_files(&mut client, dep_project, &dep_dir, &[], Some(&dep_name))?;
                } else {
                    eprintln!("  Dependency '{}' not found on device.", dep_name);
                }
            }
        }
    }

    println!("Done.");
    Ok(())
}

fn push_command(args: PushArgs, wait: bool) -> Result<()> {
    let mut client = client_for_profile(&args.profile, wait)?;
    let pname = project_name(&args.project);
    let source_dir = args.input_dir.unwrap_or_else(|| PathBuf::from(&pname));
    if !source_dir.exists() {
        bail!("Directory '{}' does not exist.", source_dir.display());
    }

    if !args.files.is_empty() {
        for filename in &args.files {
            let local_path = source_dir.join(filename);
            if !local_path.exists() {
                eprintln!("  {} (not found, skipping)", filename);
                continue;
            }
            let file_path = format!("{}/{}", args.project, filename);
            push_file(&mut client, &local_path, &file_path, filename)?;
        }
        println!("Done.");
        return Ok(());
    }

    println!(
        "Pushing '{}/' → '{}' on device...",
        source_dir.display(),
        pname
    );
    let all_projects = client.list_projects()?;
    for local_path in walk_files(&source_dir)? {
        let relative = local_path.strip_prefix(&source_dir)?.to_path_buf();
        let parts = relative
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>();

        let file_path = if parts.len() >= 3 && parts[0] == "Dependencies" {
            let dep_name = &parts[1];
            if let Some(dep_project) = all_projects
                .iter()
                .find(|p| project_name(p).eq_ignore_ascii_case(dep_name))
            {
                format!("{}/{}", dep_project, parts[2..].join("/"))
            } else {
                eprintln!("  Skipping {} (dependency not found)", relative.display());
                continue;
            }
        } else {
            format!("{}/{}", args.project, parts.join("/"))
        };

        push_file(
            &mut client,
            &local_path,
            &file_path,
            &relative.display().to_string(),
        )?;
    }

    println!("Done.");
    Ok(())
}

fn run_command(args: RunArgs, wait: bool) -> Result<()> {
    let mut client = client_for_profile(&args.profile, wait)?;
    println!("{}", client.run_project(&args.project)?);
    Ok(())
}

fn stop_command(profile: &str, wait: bool) -> Result<()> {
    let mut client = client_for_profile(profile, wait)?;
    println!("{}", client.stop_project()?);
    Ok(())
}

fn restart_command(profile: &str, wait: bool) -> Result<()> {
    let mut client = client_for_profile(profile, wait)?;
    println!(
        "{}",
        MCPClient::text(&client.call_tool("restartProject", json!({}))?)
    );
    Ok(())
}

fn pause_command(profile: &str, wait: bool) -> Result<()> {
    let mut client = client_for_profile(profile, wait)?;
    let _ = client.execute_lua("viewer.paused = true")?;
    println!("Project paused");
    Ok(())
}

fn resume_command(profile: &str, wait: bool) -> Result<()> {
    let mut client = client_for_profile(profile, wait)?;
    let _ = client.execute_lua("viewer.paused = false")?;
    println!("Project resumed");
    Ok(())
}

fn paused_command(args: PausedArgs, wait: bool) -> Result<()> {
    let mut client = client_for_profile(&args.profile, wait)?;
    match args.state.as_deref() {
        None => println!(
            "{}",
            MCPClient::text(&client.call_tool("getProjectPaused", json!({}))?)
        ),
        Some("on") => {
            let _ = client.execute_lua("viewer.paused = true")?;
            println!("paused");
        }
        Some("off") => {
            let _ = client.execute_lua("viewer.paused = false")?;
            println!("not paused");
        }
        Some(other) => bail!("Invalid state '{}'. Use on or off.", other),
    }
    Ok(())
}

fn exec_command(args: ExecArgs, wait: bool) -> Result<()> {
    if args.lua_file.is_some() && args.code.is_some() {
        bail!("Provide either CODE or --file, not both.");
    }
    let code = match (args.code, args.lua_file) {
        (Some(code), None) => code,
        (None, Some(path)) => fs::read_to_string(path)?,
        (None, None) => bail!("Provide either CODE or --file."),
        (Some(_), Some(_)) => unreachable!(),
    };
    let mut client = client_for_profile(&args.profile, wait)?;
    let result = client.execute_lua(&code)?;
    if !result.is_empty() {
        println!("{result}");
    }
    Ok(())
}

fn screenshot_command(args: ScreenshotArgs, wait: bool) -> Result<()> {
    let mut client = client_for_profile(&args.profile, wait)?;
    let Some(data) = client.capture_screenshot()? else {
        bail!("Screenshot capture failed or not supported.");
    };
    fs::write(&args.output, &data)?;
    println!(
        "Screenshot saved to {} ({} bytes)",
        args.output.display(),
        data.len()
    );
    Ok(())
}

fn idle_timer_command(args: IdleTimerArgs, wait: bool) -> Result<()> {
    let mut client = client_for_profile(&args.profile, wait)?;
    match args.state.as_deref() {
        None => println!(
            "{}",
            MCPClient::text(&client.call_tool("getIdleTimerDisabled", json!({}))?)
        ),
        Some("on") => println!(
            "{}",
            MCPClient::text(&client.call_tool("setIdleTimerDisabled", json!({"disabled": false}))?)
        ),
        Some("off") => println!(
            "{}",
            MCPClient::text(&client.call_tool("setIdleTimerDisabled", json!({"disabled": true}))?)
        ),
        Some(other) => bail!("Invalid state '{}'. Use on or off.", other),
    }
    Ok(())
}

fn logs_command(args: LogsArgs, wait: bool) -> Result<()> {
    let mut client = client_for_profile(&args.profile, wait)?;
    if args.follow {
        for line in client.stream_logs()? {
            println!("{}", line?);
        }
        return Ok(());
    }

    let mut payload = serde_json::Map::new();
    if let Some(tail) = args.tail {
        payload.insert("tail".to_string(), json!(tail));
    }
    if let Some(head) = args.head {
        payload.insert("head".to_string(), json!(head));
    }
    println!(
        "{}",
        MCPClient::text(&client.call_tool("getLogs", Value::Object(payload))?)
    );
    Ok(())
}

fn clear_logs_command(profile: &str, wait: bool) -> Result<()> {
    let mut client = client_for_profile(profile, wait)?;
    println!(
        "{}",
        MCPClient::text(&client.call_tool("clearLogs", json!({}))?)
    );
    Ok(())
}

fn new_command(args: NewArgs, wait: bool) -> Result<()> {
    let project_storage = resolve_project_storage(&args.profile, wait)?;
    if project_storage == "filesystem" {
        if args.collection.is_some() {
            bail!("--collection is only supported for collection-backed targets.");
        }
        if args.cloud {
            bail!("--cloud is only supported for collection-backed targets.");
        }
        let destination = create_local_project(&args.name, args.template.as_deref(), args.folder)?;
        println!(
            "Created project '{}'. Path: {}",
            destination
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("project"),
            destination.display()
        );
        return Ok(());
    }

    let mut client = client_for_profile(&args.profile, wait)?;
    let (name, collection, cloud) =
        parse_collection_project(&args.name, args.collection, args.cloud);
    if args.folder {
        bail!("--folder is only supported for local filesystem-backed targets.");
    }

    let mut payload = serde_json::Map::new();
    payload.insert("name".to_string(), json!(name));
    if let Some(collection) = collection {
        payload.insert("collection".to_string(), json!(collection));
    }
    if cloud {
        payload.insert("cloud".to_string(), json!(true));
    }
    if let Some(template) = args.template {
        payload.insert("template".to_string(), json!(template));
    }

    println!(
        "{}",
        MCPClient::text(&client.call_tool("createProject", Value::Object(payload))?)
    );
    Ok(())
}

fn rename_command(args: RenameArgs, wait: bool) -> Result<()> {
    let mut client = client_for_profile(&args.profile, wait)?;
    println!(
        "{}",
        MCPClient::text(&client.call_tool(
            "renameProject",
            json!({"path": args.project, "newName": args.newname})
        )?)
    );
    Ok(())
}

fn move_command(args: MoveArgs, wait: bool) -> Result<()> {
    let mut client = client_for_profile(&args.profile, wait)?;
    println!(
        "{}",
        MCPClient::text(&client.call_tool(
            "moveProject",
            json!({"path": args.project, "collection": args.collection})
        )?)
    );
    Ok(())
}

fn delete_command(args: DeleteArgs, wait: bool) -> Result<()> {
    let mut client = client_for_profile(&args.profile, wait)?;
    let pname = project_name(&args.project);
    if !prompt_confirm(&format!("Delete '{pname}'? This cannot be undone."))? {
        return Ok(());
    }
    println!(
        "{}",
        MCPClient::text(&client.call_tool("deleteProject", json!({"path": args.project}))?)
    );
    Ok(())
}

fn collections_command(args: CollectionsCommand, wait: bool) -> Result<()> {
    match args.command {
        CollectionsSubcommand::Ls(args) => collections_ls_command(&args.profile, wait),
        CollectionsSubcommand::New(args) => collections_new_command(args, wait),
        CollectionsSubcommand::Delete(args) => collections_delete_command(args, wait),
    }
}

fn collections_ls_command(profile: &str, wait: bool) -> Result<()> {
    let mut client = client_for_profile(profile, wait)?;
    for name in client.list_collections()? {
        println!("{name}");
    }
    Ok(())
}

fn collections_new_command(args: CollectionNameArgs, wait: bool) -> Result<()> {
    let mut client = client_for_profile(&args.profile, wait)?;
    println!("{}", client.create_collection(&args.name)?);
    Ok(())
}

fn collections_delete_command(args: CollectionNameArgs, wait: bool) -> Result<()> {
    let mut client = client_for_profile(&args.profile, wait)?;
    if !prompt_confirm(&format!(
        "Delete collection '{}'? This cannot be undone.",
        args.name
    ))? {
        return Ok(());
    }
    println!("{}", client.delete_collection(&args.name)?);
    Ok(())
}

fn templates_command(args: TemplatesCommand, wait: bool) -> Result<()> {
    match args.command {
        TemplatesSubcommand::Ls(args) => templates_ls_command(&args.profile, wait),
        TemplatesSubcommand::Add(args) => templates_add_command(args, wait),
        TemplatesSubcommand::Remove(args) => templates_remove_command(args, wait),
    }
}

fn templates_ls_command(profile: &str, wait: bool) -> Result<()> {
    let mut client = client_for_profile(profile, wait)?;
    for entry in client.list_templates()? {
        println!("{entry}");
    }
    Ok(())
}

fn templates_add_command(args: TemplateAddArgs, wait: bool) -> Result<()> {
    let mut client = client_for_profile(&args.profile, wait)?;
    println!(
        "{}",
        client.add_template(&args.project, args.name.as_deref())?
    );
    Ok(())
}

fn templates_remove_command(args: TemplateRemoveArgs, wait: bool) -> Result<()> {
    let mut client = client_for_profile(&args.profile, wait)?;
    if !prompt_confirm(&format!(
        "Remove template '{}'? This cannot be undone.",
        args.name
    ))? {
        return Ok(());
    }
    println!("{}", client.remove_template(&args.name)?);
    Ok(())
}

fn deps_command(args: DepsCommand, wait: bool) -> Result<()> {
    match args.command {
        DepsSubcommand::Ls(args) => deps_ls_command(args, wait),
        DepsSubcommand::Available(args) => deps_available_command(args, wait),
        DepsSubcommand::Add(args) => deps_add_command(args, wait),
        DepsSubcommand::Remove(args) => deps_remove_command(args, wait),
    }
}

fn deps_ls_command(args: ProjectOnlyArgs, wait: bool) -> Result<()> {
    let mut client = client_for_profile(&args.profile, wait)?;
    for dep in client.list_dependencies(&args.project)? {
        println!("{dep}");
    }
    Ok(())
}

fn deps_available_command(args: ProjectOnlyArgs, wait: bool) -> Result<()> {
    let mut client = client_for_profile(&args.profile, wait)?;
    for dep in client.list_available_dependencies(&args.project)? {
        println!("{dep}");
    }
    Ok(())
}

fn deps_add_command(args: DependencyArgs, wait: bool) -> Result<()> {
    let mut client = client_for_profile(&args.profile, wait)?;
    println!(
        "{}",
        client.add_dependency(&args.project, &args.dependency)?
    );
    Ok(())
}

fn deps_remove_command(args: DependencyArgs, wait: bool) -> Result<()> {
    let mut client = client_for_profile(&args.profile, wait)?;
    println!(
        "{}",
        client.remove_dependency(&args.project, &args.dependency)?
    );
    Ok(())
}

fn autocomplete_command(args: AutocompleteArgs, wait: bool) -> Result<()> {
    let mut client = client_for_profile(&args.profile, wait)?;
    let result = client.get_completions(&args.project, &args.code)?;
    let items = result
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if items.is_empty() {
        println!("(no completions)");
        return Ok(());
    }

    for item in items {
        let label = item.get("label").and_then(Value::as_str).unwrap_or("");
        if let Some(kind_name) = item
            .get("kind")
            .and_then(Value::as_i64)
            .and_then(completion_kind_name)
        {
            println!("{label} ({kind_name})");
        } else {
            println!("{label}");
        }
    }
    Ok(())
}

fn runtime_command(args: RuntimeArgs, wait: bool) -> Result<()> {
    let mut client = client_for_profile(&args.profile, wait)?;
    if let Some(runtime_type) = args.runtime_type {
        if runtime_type != "legacy" && runtime_type != "modern" {
            bail!("Runtime type must be 'legacy' or 'modern'.");
        }
        println!("{}", client.set_runtime(&args.project, &runtime_type)?);
    } else {
        println!("{}", client.get_runtime(&args.project)?);
    }
    Ok(())
}

fn doc_command(args: DocArgs, wait: bool) -> Result<()> {
    let filter_runtime = resolve_runtime_filter(args.legacy, args.modern)?;
    let mut client = client_for_profile(&args.profile, wait)?;
    let filter_runtime = match (&args.project, filter_runtime) {
        (Some(project), None) => Some(client.get_runtime(project)?),
        (_, filter_runtime) => filter_runtime,
    };

    let result = client.get_function_help(&args.function_name)?;
    let mut modern = result.get("modern").cloned();
    let mut legacy = result.get("legacy").cloned();

    match filter_runtime.as_deref() {
        Some("modern") => legacy = None,
        Some("legacy") => modern = None,
        _ => {}
    }

    if modern.is_none() && legacy.is_none() {
        if let Some(filter_runtime) = filter_runtime {
            bail!(
                "No {} documentation found for '{}'.",
                filter_runtime,
                args.function_name
            );
        }
        bail!("No documentation found for '{}'.", args.function_name);
    }

    let name = result
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(&args.function_name);
    println!("{name}");
    println!("{}", "=".repeat(name.len()));
    println!();

    match (modern.as_ref(), legacy.as_ref()) {
        (Some(modern), Some(legacy)) => {
            print_doc_section(Some("Modern"), modern);
            print_doc_section(Some("Legacy"), legacy);
        }
        (Some(modern), None) => print_doc_section(None, modern),
        (None, Some(legacy)) => print_doc_section(None, legacy),
        (None, None) => {}
    }

    let see_also = result
        .get("seeAlso")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if !see_also.is_empty() {
        println!("See also: {}", see_also.join(", "));
    }
    Ok(())
}

fn search_doc_command(args: SearchDocArgs, wait: bool) -> Result<()> {
    let filter_runtime = resolve_runtime_filter(args.legacy, args.modern)?;
    let mut client = client_for_profile(&args.profile, wait)?;
    let filter_runtime = match (&args.project, filter_runtime) {
        (Some(project), None) => Some(client.get_runtime(project)?),
        (_, filter_runtime) => filter_runtime,
    };

    let mut results = client
        .search_docs(&args.query)?
        .as_array()
        .cloned()
        .unwrap_or_default();

    if let Some(filter_runtime) = filter_runtime.as_deref() {
        results.retain(|item| {
            item.get("runtime")
                .and_then(Value::as_str)
                .map(|runtime| runtime == filter_runtime || runtime == "both")
                .unwrap_or(false)
        });
    }

    if results.is_empty() {
        if let Some(filter_runtime) = filter_runtime {
            println!(
                "No {} documentation found matching '{}'.",
                filter_runtime, args.query
            );
        } else {
            println!("No documentation found matching '{}'.", args.query);
        }
        return Ok(());
    }

    for item in results {
        let name = item.get("name").and_then(Value::as_str).unwrap_or("");
        let desc = item
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("");
        let runtime = item.get("runtime").and_then(Value::as_str).unwrap_or("");
        let tag = if runtime.is_empty() {
            String::new()
        } else {
            format!("[{runtime}]")
        };
        if desc.is_empty() {
            println!("  {name}  {tag}");
        } else {
            println!("  {name}  – {desc}  {tag}");
        }
    }

    Ok(())
}

fn resolve_project_storage(profile: &str, wait: bool) -> Result<String> {
    let Some(ProfileConfig { host, port }) = load_profile(profile)? else {
        return Ok("filesystem".to_string());
    };
    match MCPClient::new(&host, port, 30).and_then(|mut client| {
        if wait {
            wait_for_device(&host, port)?;
        }
        client.get_device_state()
    }) {
        Ok(state) => Ok(state
            .get("projectStorage")
            .and_then(Value::as_str)
            .unwrap_or("collections")
            .to_string()),
        Err(_) => Ok("filesystem".to_string()),
    }
}

fn parse_collection_project(
    name: &str,
    collection: Option<String>,
    cloud: bool,
) -> (String, Option<String>, bool) {
    let mut collection = collection;
    let mut cloud = cloud;
    let mut name_out = name.to_string();

    if name.contains('/') && collection.is_none() {
        let mut parts = name.split('/').map(|s| s.to_string()).collect::<Vec<_>>();
        if parts
            .first()
            .map(|s| s.eq_ignore_ascii_case("icloud"))
            .unwrap_or(false)
        {
            cloud = true;
            parts.remove(0);
        }
        if parts.len() >= 2 {
            collection = Some(parts[..parts.len() - 1].join("/"));
            name_out = parts.last().cloned().unwrap_or_default();
        }
    }

    (name_out, collection, cloud)
}

fn resolve_runtime_filter(legacy: bool, modern: bool) -> Result<Option<String>> {
    if legacy && modern {
        bail!("Use only one of --legacy or --modern.");
    }
    if legacy {
        Ok(Some("legacy".to_string()))
    } else if modern {
        Ok(Some("modern".to_string()))
    } else {
        Ok(None)
    }
}

fn pull_project_files(
    client: &mut MCPClient,
    project_path: &str,
    output_dir: &Path,
    files: &[String],
    label: Option<&str>,
) -> Result<()> {
    let prefix = label.map(|s| format!("[{s}] ")).unwrap_or_default();
    let mut all_files = match client.list_files(project_path) {
        Ok(files) => files,
        Err(error) => {
            eprintln!("{}Warning: could not list files: {}", prefix, error);
            return Ok(());
        }
    };

    fs::create_dir_all(output_dir)?;

    if !files.is_empty() {
        let wanted = files
            .iter()
            .map(|s| s.to_ascii_lowercase())
            .collect::<Vec<_>>();
        all_files.retain(|path| {
            let filename = path
                .trim_end_matches('/')
                .split('/')
                .next_back()
                .unwrap_or("")
                .to_ascii_lowercase();
            wanted.contains(&filename)
        });
    }

    for file_path in all_files {
        let filename = file_path
            .trim_end_matches('/')
            .split('/')
            .next_back()
            .unwrap_or("file");
        let local_path = output_dir.join(filename);
        match client.read_file(&file_path) {
            Ok(content) => {
                fs::write(&local_path, content)?;
                println!("{}  {}", prefix, filename);
            }
            Err(error) => eprintln!("{}  {} (error: {})", prefix, filename, error),
        }
    }
    Ok(())
}

fn push_file(
    client: &mut MCPClient,
    local_path: &Path,
    file_path: &str,
    label: &str,
) -> Result<()> {
    let bytes = fs::read(local_path)?;
    let content = maybe_base64_text(&bytes);
    match client.write_file(file_path, &content) {
        Ok(_) => println!("  {}", label),
        Err(error) => eprintln!("  {} (error: {})", label, error),
    }
    Ok(())
}

fn walk_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn prompt_selection(max: usize) -> Result<usize> {
    print!("Select device [1]: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();
    if input.is_empty() {
        return Ok(1);
    }
    let choice = input.parse::<usize>()?;
    if !(1..=max).contains(&choice) {
        bail!("Selection must be between 1 and {}", max);
    }
    Ok(choice)
}

fn prompt_confirm(prompt: &str) -> Result<bool> {
    print!("{prompt} [y/N]: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_ascii_lowercase();
    Ok(matches!(input.as_str(), "y" | "yes"))
}

fn completion_kind_name(kind: i64) -> Option<&'static str> {
    match kind {
        1 => Some("text"),
        2 => Some("method"),
        3 => Some("function"),
        4 => Some("constructor"),
        5 => Some("field"),
        6 => Some("variable"),
        7 => Some("class"),
        12 => Some("value"),
        14 => Some("keyword"),
        15 => Some("snippet"),
        21 => Some("constant"),
        _ => None,
    }
}

fn print_doc_section(title: Option<&str>, doc: &Value) {
    if let Some(title) = title {
        println!("{title}");
        println!("{}", "-".repeat(title.len()));
    }

    let signatures = doc
        .get("signatures")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let descriptions = signatures
        .iter()
        .filter_map(|sig| {
            sig.get("description")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
        .collect::<Vec<_>>();
    let shared_desc = descriptions
        .first()
        .cloned()
        .filter(|first| !first.is_empty() && descriptions.iter().all(|desc| desc == first));

    if let Some(shared_desc) = shared_desc.as_deref() {
        println!("{shared_desc}");
        println!();
    }

    for sig in signatures {
        let label = sig.get("label").and_then(Value::as_str).unwrap_or("");
        println!("  {label}");

        let description = sig
            .get("description")
            .and_then(Value::as_str)
            .filter(|_| shared_desc.is_none());
        if let Some(description) = description {
            println!("    {description}");
        }

        if let Some(params) = sig.get("parameters").and_then(Value::as_array) {
            for param in params {
                let name = param.get("name").and_then(Value::as_str).unwrap_or("");
                let ptype = param.get("type").and_then(Value::as_str);
                let desc = param.get("description").and_then(Value::as_str);
                let optional = param
                    .get("optional")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let mut parts = vec![format!("    {name}")];
                if let Some(ptype) = ptype {
                    parts.push(ptype.to_string());
                }
                if let Some(desc) = desc {
                    parts.push(format!("– {desc}"));
                } else if optional {
                    parts.push("(optional)".to_string());
                }
                println!("{}", parts.join("  "));
            }
        }

        if let Some(returns) = sig.get("returns").and_then(Value::as_array) {
            for ret in returns {
                let rtype = ret.get("type").and_then(Value::as_str);
                let rdesc = ret.get("description").and_then(Value::as_str);
                if rtype.is_some() || rdesc.is_some() {
                    let mut parts = vec!["→".to_string()];
                    if let Some(rtype) = rtype {
                        parts.push(rtype.to_string());
                    }
                    if let Some(rdesc) = rdesc {
                        parts.push(format!("– {rdesc}"));
                    }
                    println!("    {}", parts.join(" "));
                }
            }
        }

        println!();
    }
}

fn project_name(path: &str) -> String {
    path.split('/').next_back().unwrap_or(path).to_string()
}

#[cfg(test)]
mod tests {
    use super::{completion_kind_name, parse_collection_project, resolve_runtime_filter};

    #[test]
    fn parse_collection_project_supports_icloud_prefix() {
        let (name, collection, cloud) =
            parse_collection_project("iCloud/Documents/Foo", None, false);
        assert_eq!(name, "Foo");
        assert_eq!(collection.as_deref(), Some("Documents"));
        assert!(cloud);
    }

    #[test]
    fn parse_collection_project_keeps_explicit_collection() {
        let (name, collection, cloud) =
            parse_collection_project("Foo/Bar", Some("Examples".to_string()), false);
        assert_eq!(name, "Foo/Bar");
        assert_eq!(collection.as_deref(), Some("Examples"));
        assert!(!cloud);
    }

    #[test]
    fn resolve_runtime_filter_rejects_conflict() {
        assert!(resolve_runtime_filter(true, true).is_err());
    }

    #[test]
    fn resolve_runtime_filter_handles_single_flag() {
        assert_eq!(
            resolve_runtime_filter(true, false).unwrap().as_deref(),
            Some("legacy")
        );
        assert_eq!(
            resolve_runtime_filter(false, true).unwrap().as_deref(),
            Some("modern")
        );
    }

    #[test]
    fn completion_kind_name_matches_expected_values() {
        assert_eq!(completion_kind_name(3), Some("function"));
        assert_eq!(completion_kind_name(999), None);
    }
}
