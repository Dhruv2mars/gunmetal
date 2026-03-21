use std::{
    fs::{self, OpenOptions},
    io::Write,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    process::{Command as ProcessCommand, Stdio},
    thread,
    time::Duration,
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use gunmetal_core::{KeyScope, KeyState, NewGunmetalKey, NewProviderProfile, ProviderKind};
use gunmetal_daemon::DaemonState;
use gunmetal_providers::{ProviderHub, builtin_providers};
use gunmetal_storage::AppPaths;
use serde_json::{Map, Value, json};

#[cfg(unix)]
use std::os::unix::process::CommandExt;
#[cfg(windows)]
use std::os::windows::process::CommandExt;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 4684;

#[derive(Debug, Parser)]
#[command(name = "gunmetal")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Start(StartArgs),
    Serve(ServeArgs),
    Stop(StopArgs),
    Status(StatusArgs),
    Keys {
        #[command(subcommand)]
        command: KeyCommand,
    },
    Models {
        #[command(subcommand)]
        command: ModelCommand,
    },
    Profiles {
        #[command(subcommand)]
        command: ProfileCommand,
    },
    Providers {
        #[command(subcommand)]
        command: ProviderCommand,
    },
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    Logs {
        #[command(subcommand)]
        command: LogCommand,
    },
    Tui,
}

#[derive(Debug, clap::Args)]
pub struct StartArgs {
    #[arg(long, default_value = DEFAULT_HOST)]
    pub host: IpAddr,
    #[arg(long, default_value_t = DEFAULT_PORT)]
    pub port: u16,
}

#[derive(Debug, clap::Args)]
pub struct ServeArgs {
    #[arg(long, default_value = DEFAULT_HOST)]
    pub host: IpAddr,
    #[arg(long, default_value_t = DEFAULT_PORT)]
    pub port: u16,
}

#[derive(Debug, clap::Args)]
pub struct StopArgs {
    #[arg(long, default_value = DEFAULT_HOST)]
    pub host: IpAddr,
    #[arg(long, default_value_t = DEFAULT_PORT)]
    pub port: u16,
}

#[derive(Debug, clap::Args)]
pub struct StatusArgs {
    #[arg(long, default_value = DEFAULT_HOST)]
    pub host: IpAddr,
    #[arg(long, default_value_t = DEFAULT_PORT)]
    pub port: u16,
}

#[derive(Debug, Subcommand)]
pub enum KeyCommand {
    Create {
        #[arg(long)]
        name: String,
        #[arg(long = "scope", value_delimiter = ',')]
        scopes: Vec<KeyScope>,
        #[arg(long = "provider", value_delimiter = ',')]
        providers: Vec<ProviderKind>,
    },
    List,
    Disable {
        id: uuid::Uuid,
    },
    Revoke {
        id: uuid::Uuid,
    },
    Delete {
        id: uuid::Uuid,
    },
}

#[derive(Debug, Subcommand)]
pub enum ModelCommand {
    List,
    Sync { profile: uuid::Uuid },
}

#[derive(Debug, Subcommand)]
pub enum ProfileCommand {
    Create {
        #[arg(long)]
        provider: ProviderKind,
        #[arg(long)]
        name: String,
        #[arg(long)]
        base_url: Option<String>,
        #[arg(long)]
        api_key: Option<String>,
        #[arg(long)]
        bin_path: Option<PathBuf>,
        #[arg(long)]
        cwd: Option<PathBuf>,
        #[arg(long)]
        http_referer: Option<String>,
        #[arg(long)]
        title: Option<String>,
    },
    List,
}

#[derive(Debug, Subcommand)]
pub enum ProviderCommand {
    List,
}

#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    Status {
        profile: uuid::Uuid,
    },
    Login {
        profile: uuid::Uuid,
        #[arg(long)]
        no_open: bool,
    },
    Logout {
        profile: uuid::Uuid,
    },
}

#[derive(Debug, Subcommand)]
pub enum LogCommand {
    List {
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
}

pub async fn execute(command: Command, paths: &AppPaths, mut output: impl Write) -> Result<()> {
    let providers = ProviderHub::new(paths.clone());

    match command {
        Command::Start(args) => {
            let status = ensure_daemon_running(paths, args.host, args.port).await?;
            writeln!(output, "{} {}", status.state, status.url)?;
            if let Some(pid) = status.pid {
                writeln!(output, "pid: {pid}")?;
            }
        }
        Command::Serve(args) => {
            let address = SocketAddr::new(args.host, args.port);
            writeln!(output, "Serving gunmetal on http://{address}")?;
            gunmetal_daemon::serve(address, DaemonState::new(paths.clone())?).await?;
        }
        Command::Stop(args) => {
            let status = stop_daemon(paths, args.host, args.port).await?;
            writeln!(output, "{} {}", status.state, status.url)?;
        }
        Command::Status(args) => {
            let status = daemon_status(paths, args.host, args.port).await?;
            writeln!(output, "{} {}", status.state, status.url)?;
            if let Some(pid) = status.pid {
                writeln!(output, "pid: {pid}")?;
            }
            if let Some(health) = status.health {
                writeln!(output, "health: {health}")?;
            }
        }
        Command::Keys { command } => {
            let storage = paths.storage_handle()?;
            match command {
                KeyCommand::Create {
                    name,
                    scopes,
                    providers,
                } => {
                    let created = storage.create_key(NewGunmetalKey {
                        name,
                        scopes,
                        allowed_providers: providers,
                        expires_at: None,
                    })?;
                    writeln!(output, "created key {}", created.record.name)?;
                    writeln!(output, "id: {}", created.record.id)?;
                    writeln!(output, "secret: {}", created.secret)?;
                }
                KeyCommand::List => {
                    for key in storage.list_keys()? {
                        writeln!(output, "{} {} {}", key.id, key.name, key.state)?;
                    }
                }
                KeyCommand::Disable { id } => {
                    storage.set_key_state(id, KeyState::Disabled)?;
                    writeln!(output, "disabled key {id}")?;
                }
                KeyCommand::Revoke { id } => {
                    storage.set_key_state(id, KeyState::Revoked)?;
                    writeln!(output, "revoked key {id}")?;
                }
                KeyCommand::Delete { id } => {
                    storage.delete_key(id)?;
                    writeln!(output, "deleted key {id}")?;
                }
            }
        }
        Command::Models { command } => match command {
            ModelCommand::List => {
                for model in paths.storage_handle()?.list_models()? {
                    writeln!(output, "{} {}", model.id, model.display_name)?;
                }
            }
            ModelCommand::Sync { profile } => {
                let storage = paths.storage_handle()?;
                let profile_record = require_profile(&storage, profile)?;
                let models = providers.sync_models(&profile_record).await?;
                storage.replace_models_for_profile(
                    &profile_record.provider,
                    Some(profile_record.id),
                    &models,
                )?;
                writeln!(
                    output,
                    "synced {} models for profile {}",
                    models.len(),
                    profile_record.name
                )?;
            }
        },
        Command::Profiles { command } => match command {
            ProfileCommand::Create {
                provider,
                name,
                base_url,
                api_key,
                bin_path,
                cwd,
                http_referer,
                title,
            } => {
                let profile = paths.storage_handle()?.create_profile(NewProviderProfile {
                    provider,
                    name,
                    base_url,
                    enabled: true,
                    credentials: profile_credentials(api_key, bin_path, cwd, http_referer, title),
                })?;
                writeln!(output, "created profile {}", profile.name)?;
                writeln!(output, "id: {}", profile.id)?;
            }
            ProfileCommand::List => {
                for profile in paths.storage_handle()?.list_profiles()? {
                    writeln!(
                        output,
                        "{} {} {} enabled={}",
                        profile.id, profile.provider, profile.name, profile.enabled
                    )?;
                }
            }
        },
        Command::Providers { command } => match command {
            ProviderCommand::List => {
                for provider in builtin_providers() {
                    writeln!(
                        output,
                        "{} {:?} priority={}",
                        provider.kind, provider.class, provider.priority
                    )?;
                }
            }
        },
        Command::Auth { command } => match command {
            AuthCommand::Status { profile } => {
                let storage = paths.storage_handle()?;
                let profile_record = require_profile(&storage, profile)?;
                let status = providers.auth_status(&profile_record).await?;
                writeln!(output, "{} {}", profile_record.name, status.label)?;
                writeln!(output, "state: {:?}", status.state)?;
            }
            AuthCommand::Login { profile, no_open } => {
                let storage = paths.storage_handle()?;
                let profile_record = require_profile(&storage, profile)?;
                let session = providers.login(&profile_record, !no_open).await?;
                writeln!(output, "login url: {}", session.auth_url)?;
                writeln!(output, "login id: {}", session.login_id)?;
                if let Some(user_code) = session.user_code {
                    writeln!(output, "user code: {user_code}")?;
                }
                if let Some(interval_seconds) = session.interval_seconds {
                    writeln!(output, "poll every: {}s", interval_seconds)?;
                }
            }
            AuthCommand::Logout { profile } => {
                let storage = paths.storage_handle()?;
                let profile_record = require_profile(&storage, profile)?;
                providers.logout(&profile_record).await?;
                writeln!(output, "logged out {}", profile_record.name)?;
            }
        },
        Command::Logs { command } => match command {
            LogCommand::List { limit } => {
                for log in paths.storage_handle()?.list_request_logs(limit)? {
                    writeln!(
                        output,
                        "{} {} {} {} {} {}ms tokens={}",
                        log.started_at,
                        log.provider,
                        log.model,
                        log.endpoint,
                        log.status_code.unwrap_or_default(),
                        log.duration_ms,
                        log.usage.total_tokens.unwrap_or_default()
                    )?;
                }
            }
        },
        Command::Tui => {}
    }

    Ok(())
}

pub async fn ensure_daemon_running(
    paths: &AppPaths,
    host: IpAddr,
    port: u16,
) -> Result<ServiceStatus> {
    let current = daemon_status(paths, host, port).await?;
    if current.running {
        return Ok(current);
    }
    if current.state == "starting" {
        return wait_for_health(paths, host, port, 20).await;
    }

    start_daemon_process(paths, host, port)?;
    wait_for_health(paths, host, port, 20).await
}

pub async fn ensure_default_daemon_running(paths: &AppPaths) -> Result<ServiceStatus> {
    ensure_daemon_running(
        paths,
        DEFAULT_HOST.parse::<IpAddr>().expect("default host"),
        DEFAULT_PORT,
    )
    .await
}

async fn stop_daemon(paths: &AppPaths, host: IpAddr, port: u16) -> Result<ServiceStatus> {
    let pid_file = paths.daemon_pid_file();
    let Some(pid) = read_pid(&pid_file)? else {
        let mut status = daemon_status(paths, host, port).await?;
        status.state = "stopped".to_owned();
        return Ok(status);
    };

    terminate_pid(pid)?;
    for _ in 0..20 {
        thread::sleep(Duration::from_millis(150));
        let status = daemon_status(paths, host, port).await?;
        if !status.running {
            let _ = fs::remove_file(&pid_file);
            return Ok(ServiceStatus {
                state: "stopped".to_owned(),
                ..status
            });
        }
    }

    let _ = fs::remove_file(&pid_file);
    Ok(ServiceStatus {
        state: "stopping".to_owned(),
        ..daemon_status(paths, host, port).await?
    })
}

pub async fn daemon_status(paths: &AppPaths, host: IpAddr, port: u16) -> Result<ServiceStatus> {
    let url = format!("http://{host}:{port}");
    let health_url = format!("{url}/health");
    let pid = read_pid(&paths.daemon_pid_file())?;
    match reqwest::get(&health_url).await {
        Ok(response) => {
            let health = response.text().await.ok();
            Ok(ServiceStatus {
                state: "running".to_owned(),
                running: true,
                pid,
                url,
                health,
            })
        }
        Err(_) => {
            if let Some(pid) = pid {
                if process_exists(pid) {
                    return Ok(ServiceStatus {
                        state: "starting".to_owned(),
                        running: false,
                        pid: Some(pid),
                        url,
                        health: None,
                    });
                }
                let _ = fs::remove_file(paths.daemon_pid_file());
            }
            Ok(ServiceStatus {
                state: "stopped".to_owned(),
                running: false,
                pid: None,
                url,
                health: None,
            })
        }
    }
}

fn start_daemon_process(paths: &AppPaths, host: IpAddr, port: u16) -> Result<()> {
    let stdout = OpenOptions::new()
        .create(true)
        .append(true)
        .open(paths.daemon_stdout_log())?;
    let stderr = OpenOptions::new()
        .create(true)
        .append(true)
        .open(paths.daemon_stderr_log())?;
    let mut command = ProcessCommand::new(std::env::current_exe()?);
    #[cfg(unix)]
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    #[cfg(windows)]
    command.creation_flags(0x00000008);
    command
        .arg("serve")
        .arg("--host")
        .arg(host.to_string())
        .arg("--port")
        .arg(port.to_string())
        .env("GUNMETAL_HOME", &paths.root)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    let child = command.spawn()?;
    fs::write(paths.daemon_pid_file(), child.id().to_string())?;
    Ok(())
}

async fn wait_for_health(
    paths: &AppPaths,
    host: IpAddr,
    port: u16,
    attempts: usize,
) -> Result<ServiceStatus> {
    for _ in 0..attempts {
        let status = daemon_status(paths, host, port).await?;
        if status.running {
            return Ok(status);
        }
        thread::sleep(Duration::from_millis(150));
    }
    daemon_status(paths, host, port).await
}

fn read_pid(path: &std::path::Path) -> Result<Option<u32>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path)?;
    Ok(raw.trim().parse::<u32>().ok())
}

fn process_exists(pid: u32) -> bool {
    #[cfg(windows)]
    {
        return ProcessCommand::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
            .output()
            .ok()
            .map(|output| {
                let text = String::from_utf8_lossy(&output.stdout);
                text.contains(&format!(",\"{pid}\"")) || text.starts_with('"')
            })
            .unwrap_or(false);
    }

    #[cfg(unix)]
    {
        unsafe {
            let result = libc::kill(pid as i32, 0);
            if result == 0 {
                return true;
            }
            std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
        }
    }
}

fn terminate_pid(pid: u32) -> Result<()> {
    #[cfg(windows)]
    {
        let status = ProcessCommand::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .status()?;
        if !status.success() {
            anyhow::bail!("failed to stop daemon pid {pid}");
        }
    }

    #[cfg(not(windows))]
    {
        let status = ProcessCommand::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status()?;
        if !status.success() {
            anyhow::bail!("failed to stop daemon pid {pid}");
        }
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceStatus {
    pub state: String,
    pub running: bool,
    pub pid: Option<u32>,
    pub url: String,
    pub health: Option<String>,
}

fn require_profile(
    storage: &gunmetal_storage::StorageHandle,
    profile_id: uuid::Uuid,
) -> Result<gunmetal_core::ProviderProfile> {
    storage
        .get_profile(profile_id)?
        .ok_or_else(|| anyhow::anyhow!("profile '{}' not found", profile_id))
}

fn profile_credentials(
    api_key: Option<String>,
    bin_path: Option<PathBuf>,
    cwd: Option<PathBuf>,
    http_referer: Option<String>,
    title: Option<String>,
) -> Option<Value> {
    let mut object = Map::new();
    if let Some(api_key) = api_key {
        object.insert("api_key".to_owned(), json!(api_key));
    }
    if let Some(bin_path) = bin_path {
        object.insert("bin_path".to_owned(), json!(bin_path));
    }
    if let Some(cwd) = cwd {
        object.insert("cwd".to_owned(), json!(cwd));
    }
    if let Some(http_referer) = http_referer {
        object.insert("http_referer".to_owned(), json!(http_referer));
    }
    if let Some(title) = title {
        object.insert("title".to_owned(), json!(title));
    }
    (!object.is_empty()).then_some(Value::Object(object))
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Cli, Command, KeyCommand, LogCommand, ModelCommand, ProfileCommand};

    #[test]
    fn parses_key_create_command() {
        let cli = Cli::parse_from([
            "gunmetal",
            "keys",
            "create",
            "--name",
            "default",
            "--scope",
            "inference,models_read",
            "--provider",
            "codex,copilot",
        ]);

        match cli.command.unwrap() {
            Command::Keys { command } => match command {
                KeyCommand::Create { name, scopes, .. } => {
                    assert_eq!(name, "default");
                    assert_eq!(scopes.len(), 2);
                }
                _ => panic!("unexpected subcommand"),
            },
            _ => panic!("unexpected command"),
        }
    }

    #[test]
    fn defaults_to_no_command_for_tui_launch() {
        let cli = Cli::parse_from(["gunmetal"]);
        assert!(cli.command.is_none());
    }

    #[test]
    fn parses_profile_and_model_sync_commands() {
        let cli = Cli::parse_from([
            "gunmetal",
            "profiles",
            "create",
            "--provider",
            "openrouter",
            "--name",
            "gateway",
            "--base-url",
            "https://openrouter.ai/api/v1",
            "--api-key",
            "sk-or-test",
            "--http-referer",
            "https://gunmetal.dev",
            "--title",
            "gunmetal",
        ]);

        match cli.command.unwrap() {
            Command::Profiles { command } => match command {
                ProfileCommand::Create {
                    provider,
                    name,
                    base_url,
                    api_key,
                    ..
                } => {
                    assert_eq!(provider, gunmetal_core::ProviderKind::OpenRouter);
                    assert_eq!(name, "gateway");
                    assert_eq!(base_url.as_deref(), Some("https://openrouter.ai/api/v1"));
                    assert_eq!(api_key.as_deref(), Some("sk-or-test"));
                }
                _ => panic!("unexpected subcommand"),
            },
            _ => panic!("unexpected command"),
        }

        let cli = Cli::parse_from([
            "gunmetal",
            "models",
            "sync",
            "00000000-0000-0000-0000-000000000000",
        ]);
        match cli.command.unwrap() {
            Command::Models { command } => match command {
                ModelCommand::Sync { .. } => {}
                _ => panic!("unexpected model command"),
            },
            _ => panic!("unexpected command"),
        }
    }

    #[test]
    fn parses_service_commands() {
        let cli = Cli::parse_from(["gunmetal", "start"]);
        assert!(matches!(cli.command.unwrap(), Command::Start(_)));

        let cli = Cli::parse_from(["gunmetal", "stop"]);
        assert!(matches!(cli.command.unwrap(), Command::Stop(_)));

        let cli = Cli::parse_from(["gunmetal", "status"]);
        assert!(matches!(cli.command.unwrap(), Command::Status(_)));
    }

    #[test]
    fn parses_logs_list_command() {
        let cli = Cli::parse_from(["gunmetal", "logs", "list", "--limit", "12"]);

        match cli.command.unwrap() {
            Command::Logs { command } => match command {
                LogCommand::List { limit } => assert_eq!(limit, 12),
            },
            _ => panic!("unexpected command"),
        }
    }
}
