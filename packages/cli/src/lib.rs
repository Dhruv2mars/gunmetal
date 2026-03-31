use std::{
    fs::{self, OpenOptions},
    io::{self, IsTerminal, Write},
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
const SETUP_WAIT_ATTEMPTS: usize = 90;
const BASE_URL: &str = "http://127.0.0.1:4684/v1";
const HELP_FOOTER: &str = "Golden path:\n  gunmetal setup           connect a provider, sync models, create a key\n  gunmetal web             open the local browser UI\n  gunmetal start           keep the local API running\n  gunmetal status          confirm the service is live\n\nUse with apps:\n  Base URL  http://127.0.0.1:4684/v1\n  API Key   your Gunmetal key\n  Model     provider/model  ex: codex/gpt-5.4\n\nFirst test:\n  curl http://127.0.0.1:4684/v1/models -H 'Authorization: Bearer gm_...'";
const SETUP_HELP_FOOTER: &str = "Golden path:\n  gunmetal setup\n\nWhat setup does:\n  1. create or save one provider profile\n  2. auth that profile\n  3. sync models\n  4. create one Gunmetal key\n  5. show one working request snippet\n\nAdvanced flags stay optional.";

#[derive(Debug, Parser)]
#[command(
    name = "gunmetal",
    about = "Local-first AI switchboard.",
    long_about = None,
    after_help = HELP_FOOTER
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Setup(SetupArgs),
    Web(WebArgs),
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
pub struct WebArgs {
    #[arg(long, default_value = DEFAULT_HOST)]
    pub host: IpAddr,
    #[arg(long, default_value_t = DEFAULT_PORT)]
    pub port: u16,
    #[arg(long)]
    pub no_open: bool,
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

#[derive(Debug, clap::Args)]
#[command(
    about = "Guided first-run flow. Best default path for new users.",
    after_help = SETUP_HELP_FOOTER
)]
pub struct SetupArgs {
    #[arg(long)]
    pub provider: Option<ProviderKind>,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub base_url: Option<String>,
    #[arg(long)]
    pub api_key: Option<String>,
    #[arg(long, help_heading = "Advanced")]
    pub bin_path: Option<PathBuf>,
    #[arg(long, help_heading = "Advanced")]
    pub cwd: Option<PathBuf>,
    #[arg(long, help_heading = "Advanced")]
    pub http_referer: Option<String>,
    #[arg(long, help_heading = "Advanced")]
    pub title: Option<String>,
    #[arg(long)]
    pub key_name: Option<String>,
    #[arg(long, help_heading = "Advanced")]
    pub no_open: bool,
    #[arg(long, help_heading = "Advanced")]
    pub no_wait: bool,
    #[arg(long, help_heading = "Advanced")]
    pub no_sync: bool,
    #[arg(long, help_heading = "Advanced")]
    pub no_key: bool,
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
        key: String,
    },
    Revoke {
        key: String,
    },
    Delete {
        key: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ModelCommand {
    List,
    Sync { profile: String },
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
        profile: String,
    },
    Login {
        profile: String,
        #[arg(long)]
        no_open: bool,
        #[arg(long)]
        no_wait: bool,
    },
    Logout {
        profile: String,
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
        Command::Setup(args) => {
            setup(paths, &providers, &mut output, args).await?;
        }
        Command::Web(args) => {
            let status = ensure_daemon_running(paths, args.host, args.port).await?;
            let app_url = format!("{}/app", status.url);
            writeln!(output, "Gunmetal browser UI")?;
            if let Some(note) = &status.note {
                writeln!(output, "{note}")?;
            }
            writeln!(output, "Open: {app_url}")?;
            writeln!(output, "API: {}/v1", status.url)?;
            if let Some(pid) = status.pid {
                writeln!(output, "PID: {pid}")?;
            }
            if !args.no_open {
                if let Err(error) = webbrowser::open(&app_url) {
                    writeln!(output, "Browser open failed: {error}")?;
                } else {
                    writeln!(output, "Opened in your default browser.")?;
                }
            }
        }
        Command::Start(args) => {
            let status = ensure_daemon_running(paths, args.host, args.port).await?;
            write_service_report(&mut output, &status, ServiceVerb::Start)?;
        }
        Command::Serve(args) => {
            let address = SocketAddr::new(args.host, args.port);
            writeln!(output, "Serving gunmetal on http://{address}")?;
            gunmetal_daemon::serve(address, DaemonState::new(paths.clone())?).await?;
        }
        Command::Stop(args) => {
            let status = stop_daemon(paths, args.host, args.port).await?;
            write_service_report(&mut output, &status, ServiceVerb::Stop)?;
        }
        Command::Status(args) => {
            let status = daemon_status(paths, args.host, args.port).await?;
            write_service_report(&mut output, &status, ServiceVerb::Status)?;
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
                        scopes: normalize_scopes(scopes),
                        allowed_providers: providers,
                        expires_at: None,
                    })?;
                    writeln!(output, "created key {}", created.record.name)?;
                    writeln!(output, "id: {}", created.record.id)?;
                    writeln!(output, "secret: {}", created.secret)?;
                    writeln!(output, "base url: http://127.0.0.1:4684/v1")?;
                }
                KeyCommand::List => {
                    let keys = storage.list_keys()?;
                    if keys.is_empty() {
                        writeln!(
                            output,
                            "No keys yet. Run `gunmetal setup`, then `gunmetal keys list` again."
                        )?;
                    }
                    for key in keys {
                        let providers = if key.allowed_providers.is_empty() {
                            "all".to_owned()
                        } else {
                            key.allowed_providers
                                .iter()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                                .join(",")
                        };
                        let scopes = key
                            .scopes
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(",");
                        writeln!(
                            output,
                            "{} {} {} scopes={} providers={}",
                            key.prefix, key.name, key.state, scopes, providers
                        )?;
                    }
                }
                KeyCommand::Disable { key } => {
                    let key_record = require_key(&storage, &key)?;
                    storage.set_key_state(key_record.id, KeyState::Disabled)?;
                    writeln!(output, "disabled key {}", key_record.name)?;
                }
                KeyCommand::Revoke { key } => {
                    let key_record = require_key(&storage, &key)?;
                    storage.set_key_state(key_record.id, KeyState::Revoked)?;
                    writeln!(output, "revoked key {}", key_record.name)?;
                }
                KeyCommand::Delete { key } => {
                    let key_record = require_key(&storage, &key)?;
                    storage.delete_key(key_record.id)?;
                    writeln!(output, "deleted key {}", key_record.name)?;
                }
            }
        }
        Command::Models { command } => match command {
            ModelCommand::List => {
                let models = paths.storage_handle()?.list_models()?;
                if models.is_empty() {
                    writeln!(
                        output,
                        "No models synced yet. Run `gunmetal setup` or `gunmetal models sync <profile>`."
                    )?;
                }
                for model in models {
                    writeln!(output, "{} {}", model.id, model.display_name)?;
                }
            }
            ModelCommand::Sync { profile } => {
                let storage = paths.storage_handle()?;
                let profile_record = require_profile(&storage, &profile)?;
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
                let profiles = paths.storage_handle()?.list_profiles()?;
                if profiles.is_empty() {
                    writeln!(
                        output,
                        "No profiles yet. Run `gunmetal setup` or `gunmetal profiles create ...`."
                    )?;
                }
                for profile in profiles {
                    writeln!(
                        output,
                        "{} {} id={} enabled={}",
                        profile.provider, profile.name, profile.id, profile.enabled
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
                let profile_record = require_profile(&storage, &profile)?;
                let status = providers.auth_status(&profile_record).await?;
                writeln!(
                    output,
                    "Profile: {} ({})",
                    profile_record.name, profile_record.provider
                )?;
                writeln!(output, "Auth: {}", status.label)?;
                writeln!(output, "State: {:?}", status.state)?;
                if supports_browser_login(&profile_record.provider) {
                    writeln!(
                        output,
                        "Next: run `gunmetal auth login {}` if re-auth is needed.",
                        profile_record.name
                    )?;
                } else {
                    writeln!(
                        output,
                        "Next: update the saved API key if auth fails, then run `gunmetal auth status {}` again.",
                        profile_record.name
                    )?;
                }
            }
            AuthCommand::Login {
                profile,
                no_open,
                no_wait,
            } => {
                let storage = paths.storage_handle()?;
                let profile_record = require_profile(&storage, &profile)?;
                let session = providers.login(&profile_record, !no_open).await?;
                let user_code = session.user_code.clone();
                let interval_seconds = session.interval_seconds;
                writeln!(
                    output,
                    "Open this URL to finish auth for {} ({}):",
                    profile_record.name, profile_record.provider
                )?;
                writeln!(output, "{}", session.auth_url)?;
                writeln!(output, "Login id: {}", session.login_id)?;
                if let Some(user_code) = user_code {
                    writeln!(output, "User code: {user_code}")?;
                }
                if let Some(interval_seconds) = interval_seconds {
                    writeln!(output, "Gunmetal will check every {}s.", interval_seconds)?;
                }
                if !no_wait && supports_browser_login(&profile_record.provider) {
                    wait_for_provider_auth(
                        &providers,
                        &profile_record,
                        interval_seconds.unwrap_or(5).max(2),
                        &mut output,
                    )
                    .await?;
                } else {
                    writeln!(
                        output,
                        "Next: finish auth, then run `gunmetal auth status {}`.",
                        profile_record.name
                    )?;
                }
            }
            AuthCommand::Logout { profile } => {
                let storage = paths.storage_handle()?;
                let profile_record = require_profile(&storage, &profile)?;
                providers.logout(&profile_record).await?;
                writeln!(output, "logged out {}", profile_record.name)?;
            }
        },
        Command::Logs { command } => match command {
            LogCommand::List { limit } => {
                let logs = paths.storage_handle()?.list_request_logs(limit)?;
                if logs.is_empty() {
                    writeln!(
                        output,
                        "No logs yet. Start Gunmetal with `gunmetal start`, then make one request."
                    )?;
                }
                for log in logs {
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
        return Ok(ServiceStatus {
            note: Some("Gunmetal was already running.".to_owned()),
            ..current
        });
    }
    if current.state == "starting" {
        return wait_for_health(paths, host, port, 20).await;
    }

    start_daemon_process(paths, host, port)?;
    let status = wait_for_health(paths, host, port, 20).await?;
    if status.running {
        return Ok(ServiceStatus {
            note: Some("Gunmetal started.".to_owned()),
            ..status
        });
    }

    anyhow::bail!("{}", diagnose_start_failure(paths, port))
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
        status.note = Some("Gunmetal was already stopped.".to_owned());
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
                note: Some("Gunmetal stopped.".to_owned()),
                ..status
            });
        }
    }

    Ok(stop_timeout_status(daemon_status(paths, host, port).await?))
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
                note: None,
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
                        note: Some("Gunmetal is still starting.".to_owned()),
                    });
                }
                let _ = fs::remove_file(paths.daemon_pid_file());
                return Ok(ServiceStatus {
                    state: "stopped".to_owned(),
                    running: false,
                    pid: None,
                    url,
                    health: None,
                    note: Some("Removed stale daemon state.".to_owned()),
                });
            }
            Ok(ServiceStatus {
                state: "stopped".to_owned(),
                running: false,
                pid: None,
                url,
                health: None,
                note: None,
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
    pub note: Option<String>,
}

fn stop_timeout_status(status: ServiceStatus) -> ServiceStatus {
    ServiceStatus {
        state: "stopping".to_owned(),
        note: Some("Gunmetal is still shutting down. Run `gunmetal status` again.".to_owned()),
        ..status
    }
}

async fn setup(
    paths: &AppPaths,
    providers: &ProviderHub,
    output: &mut impl Write,
    args: SetupArgs,
) -> Result<()> {
    let interactive = io::stdin().is_terminal();
    writeln!(output, "Gunmetal setup")?;
    writeln!(
        output,
        "This creates one provider profile, checks auth, syncs models, and creates one local key that works across providers."
    )?;
    writeln!(output)?;
    let provider = match args.provider {
        Some(provider) => provider,
        None => prompt_provider(output, interactive)?,
    };
    let name = prompt_or_value(
        output,
        interactive,
        "Profile name",
        args.name,
        Some(provider.to_string()),
    )?;
    let base_url = if supports_base_url(&provider) {
        prompt_optional(
            output,
            interactive,
            "Base URL",
            args.base_url,
            default_base_url(&provider).map(str::to_owned),
        )?
    } else {
        args.base_url
    };
    let api_key = if needs_api_key(&provider) {
        Some(prompt_or_value(
            output,
            interactive,
            "API key",
            args.api_key,
            None,
        )?)
    } else {
        args.api_key
    };

    let storage = paths.storage_handle()?;
    let profile = storage.create_profile(NewProviderProfile {
        provider: provider.clone(),
        name,
        base_url,
        enabled: true,
        credentials: profile_credentials(
            api_key,
            args.bin_path,
            args.cwd,
            args.http_referer,
            args.title,
        ),
    })?;
    writeln!(
        output,
        "Saved profile {} ({})",
        profile.name, profile.provider
    )?;

    if supports_browser_login(&provider) {
        let session = providers.login(&profile, !args.no_open).await?;
        writeln!(output, "Open this URL to finish auth: {}", session.auth_url)?;
        if let Some(user_code) = session.user_code.clone() {
            writeln!(output, "User code: {user_code}")?;
        }
        if !args.no_wait {
            wait_for_provider_auth(
                providers,
                &profile,
                session.interval_seconds.unwrap_or(5).max(2),
                output,
            )
            .await?;
        } else {
            writeln!(
                output,
                "Auth still needs to finish. Run `gunmetal auth status {}` when done.",
                profile.name
            )?;
        }
    } else {
        let status = providers.auth_status(&profile).await?;
        writeln!(output, "Auth: {}", status.label)?;
    }

    let mut models = Vec::new();
    if !args.no_sync {
        models = providers.sync_models(&profile).await?;
        storage.replace_models_for_profile(&profile.provider, Some(profile.id), &models)?;
        writeln!(output, "Synced {} models.", models.len())?;
    }

    let mut created_secret = None;
    if !args.no_key {
        let key_name = args
            .key_name
            .unwrap_or_else(|| format!("{}-key", profile.name));
        let created = storage.create_key(NewGunmetalKey {
            name: key_name,
            scopes: default_scopes(),
            allowed_providers: Vec::new(),
            expires_at: None,
        })?;
        created_secret = Some(created.secret.clone());
        writeln!(output, "Created key {}.", created.record.name)?;
        writeln!(output, "API key: {}", created.secret)?;
    }

    if let Some(model) = models.first() {
        writeln!(output, "First model: {}", model.id)?;
    }

    writeln!(output)?;
    writeln!(output, "What just happened")?;
    writeln!(
        output,
        "- profile saved: {} ({})",
        profile.name, profile.provider
    )?;
    if !args.no_sync {
        writeln!(output, "- models synced: {}", models.len())?;
    }
    if !args.no_key {
        writeln!(output, "- local key created")?;
    }
    writeln!(output)?;
    writeln!(output, "What to do next")?;
    writeln!(
        output,
        "1. Start Gunmetal: gunmetal web  (or gunmetal start)"
    )?;
    writeln!(output, "2. Base URL: {BASE_URL}")?;
    writeln!(output, "3. Model format: provider/model")?;
    if let (Some(secret), Some(model)) = (created_secret, models.first()) {
        writeln!(output, "4. First test:")?;
        writeln!(
            output,
            "   curl {BASE_URL}/models -H 'Authorization: Bearer {secret}'"
        )?;
        writeln!(
            output,
            "   curl {BASE_URL}/chat/completions -H 'Authorization: Bearer {secret}' -H 'Content-Type: application/json' -d '{{\"model\":\"{}\",\"messages\":[{{\"role\":\"user\",\"content\":\"say ok\"}}]}}'",
            model.id
        )?;
    }
    Ok(())
}

async fn wait_for_provider_auth(
    providers: &ProviderHub,
    profile: &gunmetal_core::ProviderProfile,
    interval_seconds: u64,
    output: &mut impl Write,
) -> Result<()> {
    for _ in 0..SETUP_WAIT_ATTEMPTS {
        let status = providers.auth_status(profile).await?;
        if format!("{:?}", status.state) == "Connected" {
            writeln!(output, "auth complete: {}", status.label)?;
            return Ok(());
        }
        writeln!(output, "waiting for auth... {:?}", status.state)?;
        thread::sleep(Duration::from_secs(interval_seconds));
    }

    anyhow::bail!(
        "authentication did not finish in time for profile '{}'. finish in the browser, then run `gunmetal auth status {}` or `gunmetal auth login {}` again",
        profile.name,
        profile.name,
        profile.name
    )
}

fn require_profile(
    storage: &gunmetal_storage::StorageHandle,
    selector: &str,
) -> Result<gunmetal_core::ProviderProfile> {
    if let Ok(id) = uuid::Uuid::parse_str(selector)
        && let Some(profile) = storage.get_profile(id)?
    {
        return Ok(profile);
    }

    let matches = storage
        .list_profiles()?
        .into_iter()
        .filter(|profile| {
            profile.name.eq_ignore_ascii_case(selector)
                || format!("{}:{}", profile.provider, profile.name).eq_ignore_ascii_case(selector)
        })
        .collect::<Vec<_>>();

    match matches.len() {
        1 => Ok(matches.into_iter().next().expect("single match")),
        0 => anyhow::bail!(
            "profile '{}' not found. run `gunmetal profiles list` or `gunmetal setup`.",
            selector
        ),
        _ => anyhow::bail!(
            "profile '{}' is ambiguous. use the id from `gunmetal profiles list`.",
            selector
        ),
    }
}

fn require_key(
    storage: &gunmetal_storage::StorageHandle,
    selector: &str,
) -> Result<gunmetal_core::GunmetalKey> {
    if let Ok(id) = uuid::Uuid::parse_str(selector)
        && let Some(key) = storage.get_key(id)?
    {
        return Ok(key);
    }

    let matches = storage
        .list_keys()?
        .into_iter()
        .filter(|key| {
            key.name.eq_ignore_ascii_case(selector) || key.prefix.eq_ignore_ascii_case(selector)
        })
        .collect::<Vec<_>>();

    match matches.len() {
        1 => Ok(matches.into_iter().next().expect("single match")),
        0 => anyhow::bail!(
            "key '{}' not found. run `gunmetal keys list` or create one in `gunmetal setup`.",
            selector
        ),
        _ => anyhow::bail!(
            "key '{}' is ambiguous. use the prefix from `gunmetal keys list`.",
            selector
        ),
    }
}

fn normalize_scopes(scopes: Vec<KeyScope>) -> Vec<KeyScope> {
    if scopes.is_empty() {
        default_scopes()
    } else {
        scopes
    }
}

fn default_scopes() -> Vec<KeyScope> {
    vec![KeyScope::Inference, KeyScope::ModelsRead]
}

fn supports_browser_login(provider: &ProviderKind) -> bool {
    matches!(provider, ProviderKind::Codex | ProviderKind::Copilot)
}

fn needs_api_key(provider: &ProviderKind) -> bool {
    matches!(
        provider,
        ProviderKind::OpenRouter
            | ProviderKind::Zen
            | ProviderKind::OpenAi
            | ProviderKind::Azure
            | ProviderKind::Nvidia
    )
}

fn supports_base_url(provider: &ProviderKind) -> bool {
    matches!(
        provider,
        ProviderKind::OpenRouter
            | ProviderKind::Zen
            | ProviderKind::OpenAi
            | ProviderKind::Azure
            | ProviderKind::Nvidia
    )
}

fn default_base_url(provider: &ProviderKind) -> Option<&'static str> {
    match provider {
        ProviderKind::OpenRouter => Some("https://openrouter.ai/api/v1"),
        ProviderKind::Zen => Some("https://opencode.ai/zen/v1"),
        ProviderKind::OpenAi => Some("https://api.openai.com/v1"),
        _ => None,
    }
}

fn prompt_provider(output: &mut impl Write, interactive: bool) -> Result<ProviderKind> {
    let value = prompt_or_value(
        output,
        interactive,
        "Provider (codex, copilot, openrouter, zen, openai)",
        None,
        Some("openai".to_owned()),
    )?;
    value
        .parse::<ProviderKind>()
        .map_err(|error| anyhow::anyhow!(error))
}

fn prompt_or_value(
    output: &mut impl Write,
    interactive: bool,
    label: &str,
    value: Option<String>,
    default: Option<String>,
) -> Result<String> {
    match value {
        Some(value) if !value.trim().is_empty() => Ok(value),
        _ if interactive => prompt_line(output, label, default),
        _ => anyhow::bail!(
            "missing {}. rerun `gunmetal setup` interactively or read `gunmetal setup --help`.",
            label.to_lowercase()
        ),
    }
}

#[derive(Debug, Clone, Copy)]
enum ServiceVerb {
    Start,
    Stop,
    Status,
}

fn write_service_report(
    output: &mut impl Write,
    status: &ServiceStatus,
    verb: ServiceVerb,
) -> Result<()> {
    match verb {
        ServiceVerb::Start if status.running => writeln!(output, "Gunmetal is running.")?,
        ServiceVerb::Stop if status.running => writeln!(output, "Gunmetal is still stopping.")?,
        ServiceVerb::Stop => writeln!(output, "Gunmetal is stopped.")?,
        ServiceVerb::Status if status.running => writeln!(output, "Gunmetal is running.")?,
        ServiceVerb::Status => writeln!(output, "Gunmetal is not running.")?,
        ServiceVerb::Start => writeln!(output, "Gunmetal is not running.")?,
    }

    if let Some(note) = &status.note {
        writeln!(output, "{note}")?;
    }
    writeln!(output, "Base URL: {}/v1", status.url)?;
    if let Some(pid) = status.pid {
        writeln!(output, "PID: {pid}")?;
    }
    if let Some(health) = &status.health {
        writeln!(output, "Health: {health}")?;
    }
    if !status.running {
        writeln!(output, "Run `gunmetal start` or open `gunmetal`.")?;
    }
    Ok(())
}

fn diagnose_start_failure(paths: &AppPaths, port: u16) -> String {
    let stderr = fs::read_to_string(paths.daemon_stderr_log()).unwrap_or_default();
    if stderr.contains("Address already in use")
        || stderr.contains("os error 48")
        || stderr.contains("os error 98")
        || stderr.contains("os error 10013")
    {
        return format!(
            "Gunmetal could not start because port {} is already in use. Stop the other process or rerun `gunmetal start --port <port>`.",
            port
        );
    }

    "Gunmetal did not become healthy. Run `gunmetal status` and inspect ~/.gunmetal/daemon.stderr.log.".to_owned()
}

fn prompt_optional(
    output: &mut impl Write,
    interactive: bool,
    label: &str,
    value: Option<String>,
    default: Option<String>,
) -> Result<Option<String>> {
    match value {
        Some(value) => Ok((!value.trim().is_empty()).then_some(value)),
        None if interactive => {
            let value = prompt_line_allow_empty(output, label, default)?;
            Ok((!value.trim().is_empty()).then_some(value))
        }
        None => Ok(default),
    }
}

fn prompt_line(output: &mut impl Write, label: &str, default: Option<String>) -> Result<String> {
    loop {
        let value = prompt_line_allow_empty(output, label, default.clone())?;
        if !value.trim().is_empty() {
            return Ok(value);
        }
    }
}

fn prompt_line_allow_empty(
    output: &mut impl Write,
    label: &str,
    default: Option<String>,
) -> Result<String> {
    match &default {
        Some(default) => write!(output, "{} [{}]: ", label, default)?,
        None => write!(output, "{}: ", label)?,
    }
    output.flush()?;
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    let trimmed = buffer.trim().to_owned();
    if trimmed.is_empty() {
        Ok(default.unwrap_or_default())
    } else {
        Ok(trimmed)
    }
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
    use clap::{CommandFactory, Parser};
    use tempfile::TempDir;

    use super::{
        AuthCommand, Cli, Command, KeyCommand, LogCommand, ModelCommand, ProfileCommand,
        StatusArgs, execute,
    };

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

        let cli = Cli::parse_from(["gunmetal", "web", "--no-open"]);
        assert!(matches!(cli.command.unwrap(), Command::Web(_)));

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

    #[test]
    fn parses_setup_command() {
        let cli = Cli::parse_from([
            "gunmetal",
            "setup",
            "--provider",
            "openai",
            "--name",
            "father-openai",
            "--api-key",
            "sk-test",
        ]);

        assert!(matches!(cli.command.unwrap(), Command::Setup(_)));
    }

    #[test]
    fn parses_named_profile_selectors() {
        let cli = Cli::parse_from(["gunmetal", "auth", "status", "father-openai"]);
        match cli.command.unwrap() {
            Command::Auth { command } => match command {
                AuthCommand::Status { profile } => assert_eq!(profile, "father-openai"),
                _ => panic!("unexpected auth command"),
            },
            _ => panic!("unexpected command"),
        }
    }

    #[test]
    fn root_help_points_at_golden_path() {
        let help = Cli::command().render_help().to_string();

        assert!(help.contains("gunmetal setup"));
        assert!(help.contains("gunmetal web"));
        assert!(help.contains("gunmetal start"));
        assert!(help.contains("gunmetal status"));
        assert!(help.contains("http://127.0.0.1:4684/v1"));
        assert!(help.contains("provider/model"));
        assert!(help.contains("/v1/models"));
    }

    #[test]
    fn setup_help_keeps_happy_path_and_advanced_flags_separate() {
        let mut command = Cli::command();
        let setup = command
            .find_subcommand_mut("setup")
            .expect("setup subcommand");
        let help = setup.render_help().to_string();

        assert!(help.contains("Guided first-run flow"));
        assert!(help.contains("gunmetal setup"));
        assert!(help.contains("Advanced"));
        assert!(help.contains("--no-open"));
        assert!(help.contains("--no-sync"));
    }

    #[test]
    fn stop_timeout_keeps_pid_and_running_state() {
        let status = super::ServiceStatus {
            state: "running".to_owned(),
            running: true,
            pid: Some(42),
            url: "http://127.0.0.1:4684".to_owned(),
            health: Some("{\"status\":\"ok\"}".to_owned()),
            note: None,
        };

        let result = super::stop_timeout_status(status);

        assert_eq!(result.state, "stopping");
        assert!(result.running);
        assert_eq!(result.pid, Some(42));
        assert_eq!(
            result.note.as_deref(),
            Some("Gunmetal is still shutting down. Run `gunmetal status` again.")
        );
    }

    #[tokio::test]
    async fn status_output_tells_user_how_to_recover_when_daemon_is_stopped() {
        let temp = TempDir::new().unwrap();
        let paths =
            gunmetal_storage::AppPaths::from_root(temp.path().join("gunmetal-home")).unwrap();
        let mut output = Vec::new();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        execute(
            Command::Status(StatusArgs {
                host: "127.0.0.1".parse().unwrap(),
                port,
            }),
            &paths,
            &mut output,
        )
        .await
        .unwrap();

        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("Gunmetal is not running."));
        assert!(text.contains("Run `gunmetal start` or open `gunmetal`."));
        assert!(text.contains(&format!("http://127.0.0.1:{port}/v1")));
    }

    #[tokio::test]
    async fn empty_lists_are_explicit_and_point_at_next_action() {
        let temp = TempDir::new().unwrap();
        let paths =
            gunmetal_storage::AppPaths::from_root(temp.path().join("gunmetal-home")).unwrap();

        for command in [
            Command::Profiles {
                command: ProfileCommand::List,
            },
            Command::Keys {
                command: KeyCommand::List,
            },
            Command::Models {
                command: ModelCommand::List,
            },
            Command::Logs {
                command: LogCommand::List { limit: 20 },
            },
        ] {
            let mut output = Vec::new();
            execute(command, &paths, &mut output).await.unwrap();
            let text = String::from_utf8(output).unwrap();
            assert!(text.contains("gunmetal setup") || text.contains("gunmetal start"));
        }
    }

    #[tokio::test]
    async fn missing_profile_errors_tell_user_how_to_recover() {
        let temp = TempDir::new().unwrap();
        let paths =
            gunmetal_storage::AppPaths::from_root(temp.path().join("gunmetal-home")).unwrap();
        let mut output = Vec::new();

        let error = execute(
            Command::Auth {
                command: AuthCommand::Status {
                    profile: "missing".to_owned(),
                },
            },
            &paths,
            &mut output,
        )
        .await
        .unwrap_err();

        let message = error.to_string();
        assert!(message.contains("profile 'missing' not found"));
        assert!(message.contains("gunmetal setup"));
        assert!(message.contains("gunmetal profiles list"));
    }
}
