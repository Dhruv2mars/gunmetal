use std::{
    fs::{self, OpenOptions},
    io::{self, IsTerminal, Write},
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    process::{Command as ProcessCommand, Stdio},
    thread,
    time::{Duration, Instant},
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use futures_util::StreamExt;
use gunmetal_core::{
    ChatMessage, ChatRole, KeyScope, KeyState, ModelDescriptor, NewGunmetalKey, NewProviderProfile,
    ProviderAuthState, ProviderKind, ProviderLoginSession, RequestLogEntry, TokenUsage,
};
use gunmetal_daemon::DaemonState;
use gunmetal_providers::{builtin_provider_hub, builtin_providers};
use gunmetal_sdk::ProviderHub;
use gunmetal_storage::AppPaths;
use serde_json::{Map, Value, json};
use uuid::Uuid;

#[cfg(test)]
fn provider_definition_fixture(
    kind: ProviderKind,
    class: gunmetal_sdk::ProviderClass,
    priority: usize,
) -> gunmetal_sdk::ProviderDefinition {
    let (label, auth_method, supports_base_url, helper_title, helper_body, base_url_placeholder) =
        match kind {
            ProviderKind::Codex => (
                "codex",
                gunmetal_sdk::ProviderAuthMethod::BrowserSession,
                false,
                "Browser sign-in provider",
                "Save the provider, then auth it in the browser.",
                "not used for this provider",
            ),
            ProviderKind::Copilot => (
                "copilot",
                gunmetal_sdk::ProviderAuthMethod::BrowserSession,
                false,
                "Browser sign-in provider",
                "Save the provider, then auth it in the browser.",
                "not used for this provider",
            ),
            ProviderKind::OpenRouter => (
                "openrouter",
                gunmetal_sdk::ProviderAuthMethod::ApiKey,
                true,
                "Gateway provider",
                "Save the upstream API key here.",
                "https://openrouter.ai/api/v1",
            ),
            ProviderKind::Zen => (
                "zen",
                gunmetal_sdk::ProviderAuthMethod::ApiKey,
                true,
                "Gateway provider",
                "Save the upstream API key here.",
                "https://opencode.ai/zen/v1",
            ),
            ProviderKind::OpenAi => (
                "openai",
                gunmetal_sdk::ProviderAuthMethod::ApiKey,
                true,
                "Direct provider",
                "Save the upstream API key here.",
                "https://api.openai.com/v1",
            ),
            ProviderKind::Custom(_) | ProviderKind::Azure | ProviderKind::Nvidia => (
                "custom",
                gunmetal_sdk::ProviderAuthMethod::ApiKey,
                true,
                "Direct provider",
                "Save the upstream API key here.",
                "optional override",
            ),
        };
    gunmetal_sdk::ProviderDefinition {
        kind,
        label,
        class,
        priority,
        capabilities: gunmetal_sdk::ProviderCapabilities {
            auth_method,
            supports_base_url,
            supports_model_sync: true,
            supports_chat_completions: true,
            supports_responses_api: true,
            supports_streaming: true,
        },
        ux: gunmetal_sdk::ProviderUxHints {
            helper_title,
            helper_body,
            suggested_name: label,
            base_url_placeholder,
        },
    }
}

#[cfg(unix)]
use std::os::unix::process::CommandExt;
#[cfg(windows)]
use std::os::windows::process::CommandExt;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 4684;
const SETUP_WAIT_ATTEMPTS: usize = 90;
const BASE_URL: &str = "http://127.0.0.1:4684/v1";
const HELP_FOOTER: &str = "Golden path:\n  gunmetal setup           connect a provider, sync models, create a key\n  gunmetal web             open the local browser UI\n  gunmetal chat            test a key against one synced model\n  gunmetal logs summary    inspect recent provider/model traffic\n  gunmetal start           keep the local API running\n  gunmetal status          confirm the service is live\n\nUse with apps:\n  Base URL  http://127.0.0.1:4684/v1\n  API Key   your Gunmetal key\n  Model     provider/model  ex: codex/gpt-5.4\n\nFirst test:\n  curl http://127.0.0.1:4684/v1/models -H 'Authorization: Bearer gm_...'";
const SETUP_HELP_FOOTER: &str = "Golden path:\n  gunmetal setup\n\nWhat setup does:\n  1. connect one provider\n  2. auth that provider\n  3. sync models\n  4. create one Gunmetal key\n  5. show one working request snippet\n\nAdvanced flags stay optional.";
const CHAT_HELP_FOOTER: &str = "Examples:\n  gunmetal chat\n  gunmetal chat --api-key gm_... --model codex/gpt-5.4\n  gunmetal chat --mode responses --prompt 'say ok'\n\nInteractive commands:\n  /clear   reset conversation history\n  /quit    exit the playground";
const WEB_HELP_FOOTER: &str = "Golden path:\n  gunmetal web\n\nWhat it does:\n  1. starts Gunmetal if needed\n  2. opens the local browser UI at http://127.0.0.1:4684/app\n  3. keeps the API at http://127.0.0.1:4684/v1 on the same machine";
const START_HELP_FOOTER: &str = "Use this when you want the local API running in the background.\nThen point apps at http://127.0.0.1:4684/v1 or open `gunmetal web`.";
const STATUS_HELP_FOOTER: &str = "Shows whether the managed local Gunmetal service is live.\nIf it is not running, start it with `gunmetal start` or open `gunmetal web`.";
const PROVIDERS_LIST_HELP_FOOTER: &str = "Lists built-in provider support, auth mode, request modes, and priority.\nUse `gunmetal profiles list` for the providers you already saved locally.";
const LOGS_LIST_HELP_FOOTER: &str = "Examples:\n  gunmetal logs list\n  gunmetal logs list --provider codex\n  gunmetal logs list --query timeout --status error\n  gunmetal logs list --model openai/gpt-5.4";

#[derive(Debug, Parser)]
#[command(
    name = "gunmetal",
    about = "Local inference middle layer.",
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
    Chat(ChatArgs),
    #[command(about = "Open the local browser UI. Starts Gunmetal if needed.")]
    Web(WebArgs),
    #[command(about = "Start the local Gunmetal API in the background.")]
    Start(StartArgs),
    #[command(about = "Run the local Gunmetal API in the foreground.")]
    Serve(ServeArgs),
    #[command(about = "Stop the managed local Gunmetal service.")]
    Stop(StopArgs),
    #[command(about = "Check whether the local Gunmetal service is live.")]
    Status(StatusArgs),
    #[command(about = "Manage Gunmetal-local API keys.")]
    Keys {
        #[command(subcommand)]
        command: KeyCommand,
    },
    #[command(about = "List or sync the local model registry.")]
    Models {
        #[command(subcommand)]
        command: ModelCommand,
    },
    #[command(about = "Manage saved provider connections on this machine.")]
    Profiles {
        #[command(subcommand)]
        command: ProfileCommand,
    },
    #[command(about = "Inspect built-in provider support shipped by Gunmetal.")]
    Providers {
        #[command(subcommand)]
        command: ProviderCommand,
    },
    #[command(about = "Check auth state or log browser providers in and out.")]
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    #[command(about = "Inspect recent request history and token usage.")]
    Logs {
        #[command(subcommand)]
        command: LogCommand,
    },
}

#[derive(Debug, Clone, Copy, clap::ValueEnum, PartialEq, Eq)]
pub enum ChatMode {
    Chat,
    Responses,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum, PartialEq, Eq)]
pub enum LogStatus {
    Success,
    Error,
}

#[derive(Debug, clap::Args)]
#[command(
    about = "Interactive local playground for one Gunmetal key and one synced model.",
    after_help = CHAT_HELP_FOOTER
)]
pub struct ChatArgs {
    #[arg(long, help = "Gunmetal-local API key to use for the playground.")]
    pub api_key: Option<String>,
    #[arg(long, help = "Synced model id in provider/model form.")]
    pub model: Option<String>,
    #[arg(long, value_enum, default_value_t = ChatMode::Chat, help = "Request shape to use for the playground.")]
    pub mode: ChatMode,
    #[arg(long, help = "Run one prompt non-interactively and exit.")]
    pub prompt: Option<String>,
}

#[derive(Debug, clap::Args)]
#[command(
    about = "Start the local Gunmetal API in the background.",
    after_help = START_HELP_FOOTER
)]
pub struct StartArgs {
    #[arg(long, default_value = DEFAULT_HOST, help = "Host for the local Gunmetal service.")]
    pub host: IpAddr,
    #[arg(long, default_value_t = DEFAULT_PORT, help = "Port for the local Gunmetal service.")]
    pub port: u16,
}

#[derive(Debug, clap::Args)]
#[command(
    about = "Open the local browser UI. Starts Gunmetal if needed.",
    after_help = WEB_HELP_FOOTER
)]
pub struct WebArgs {
    #[arg(long, default_value = DEFAULT_HOST, help = "Host for the local Gunmetal service.")]
    pub host: IpAddr,
    #[arg(long, default_value_t = DEFAULT_PORT, help = "Port for the local Gunmetal service.")]
    pub port: u16,
    #[arg(
        long,
        help = "Start the local browser UI without opening a browser window."
    )]
    pub no_open: bool,
}

#[derive(Debug, clap::Args)]
#[command(about = "Run the local Gunmetal API in the foreground.")]
pub struct ServeArgs {
    #[arg(long, default_value = DEFAULT_HOST, help = "Host for the local Gunmetal service.")]
    pub host: IpAddr,
    #[arg(long, default_value_t = DEFAULT_PORT, help = "Port for the local Gunmetal service.")]
    pub port: u16,
}

#[derive(Debug, clap::Args)]
#[command(about = "Stop the managed local Gunmetal service.")]
pub struct StopArgs {
    #[arg(long, default_value = DEFAULT_HOST, help = "Host for the managed local Gunmetal service.")]
    pub host: IpAddr,
    #[arg(long, default_value_t = DEFAULT_PORT, help = "Port for the managed local Gunmetal service.")]
    pub port: u16,
}

#[derive(Debug, clap::Args)]
#[command(
    about = "Check whether the local Gunmetal service is live.",
    after_help = STATUS_HELP_FOOTER
)]
pub struct StatusArgs {
    #[arg(long, default_value = DEFAULT_HOST, help = "Host for the managed local Gunmetal service.")]
    pub host: IpAddr,
    #[arg(long, default_value_t = DEFAULT_PORT, help = "Port for the managed local Gunmetal service.")]
    pub port: u16,
}

#[derive(Debug, clap::Args)]
#[command(
    about = "Guided first-run flow. Best default path for new users.",
    after_help = SETUP_HELP_FOOTER
)]
pub struct SetupArgs {
    #[arg(
        long,
        help = "Provider kind to connect, such as codex, openrouter, or openai."
    )]
    pub provider: Option<ProviderKind>,
    #[arg(long, help = "Saved local name for this provider connection.")]
    pub name: Option<String>,
    #[arg(
        long,
        help = "Optional upstream base URL override when the provider supports it."
    )]
    pub base_url: Option<String>,
    #[arg(long, help = "Upstream API key for gateway and direct providers.")]
    pub api_key: Option<String>,
    #[arg(
        long,
        help_heading = "Advanced",
        help = "Custom provider binary path when supported."
    )]
    pub bin_path: Option<PathBuf>,
    #[arg(
        long,
        help_heading = "Advanced",
        help = "Working directory for provider runtimes that need one."
    )]
    pub cwd: Option<PathBuf>,
    #[arg(
        long,
        help_heading = "Advanced",
        help = "HTTP referer override for providers that require it."
    )]
    pub http_referer: Option<String>,
    #[arg(
        long,
        help_heading = "Advanced",
        help = "Custom app title header for providers that support it."
    )]
    pub title: Option<String>,
    #[arg(long, help = "Local Gunmetal key name to create at the end of setup.")]
    pub key_name: Option<String>,
    #[arg(
        long,
        help_heading = "Advanced",
        help = "Print browser auth URLs without opening a browser window."
    )]
    pub no_open: bool,
    #[arg(
        long,
        help_heading = "Advanced",
        help = "Start browser auth and return immediately without waiting for completion."
    )]
    pub no_wait: bool,
    #[arg(long, help_heading = "Advanced", help = "Skip model sync after auth.")]
    pub no_sync: bool,
    #[arg(
        long,
        help_heading = "Advanced",
        help = "Skip Gunmetal key creation at the end of setup."
    )]
    pub no_key: bool,
}

#[derive(Debug, Subcommand)]
pub enum KeyCommand {
    #[command(about = "Create one Gunmetal-local API key.")]
    Create {
        #[arg(long, help = "Human-readable name for the new key.")]
        name: String,
        #[arg(
            long = "scope",
            value_delimiter = ',',
            help = "Comma-separated scopes. Defaults to inference,models_read."
        )]
        scopes: Vec<KeyScope>,
        #[arg(
            long = "provider",
            value_delimiter = ',',
            help = "Optional provider allowlist for this key."
        )]
        providers: Vec<ProviderKind>,
    },
    #[command(about = "List saved Gunmetal-local API keys.")]
    List,
    #[command(about = "Disable one key without deleting it.")]
    Disable { key: String },
    #[command(about = "Revoke one key so it cannot be used again.")]
    Revoke { key: String },
    #[command(about = "Delete one key record from local storage.")]
    Delete { key: String },
}

#[derive(Debug, Subcommand)]
pub enum ModelCommand {
    #[command(about = "List synced models in the local registry.")]
    List,
    #[command(about = "Sync models for one saved provider connection.")]
    Sync { profile: String },
}

#[derive(Debug, Subcommand)]
pub enum ProfileCommand {
    #[command(about = "Create or update one saved provider connection.")]
    Create {
        #[arg(long, help = "Provider kind to connect, such as codex or openrouter.")]
        provider: ProviderKind,
        #[arg(long, help = "Saved local name for this provider connection.")]
        name: String,
        #[arg(long, help = "Optional custom upstream base URL.")]
        base_url: Option<String>,
        #[arg(long, help = "Upstream API key for gateway and direct providers.")]
        api_key: Option<String>,
        #[arg(long, help = "Custom provider binary path when supported.")]
        bin_path: Option<PathBuf>,
        #[arg(long, help = "Working directory for provider runtimes that need one.")]
        cwd: Option<PathBuf>,
        #[arg(long, help = "HTTP referer override for providers that require it.")]
        http_referer: Option<String>,
        #[arg(long, help = "Custom app title header for providers that support it.")]
        title: Option<String>,
    },
    #[command(about = "List saved provider connections on this machine.")]
    List,
}

#[derive(Debug, Subcommand)]
pub enum ProviderCommand {
    #[command(
        about = "List built-in provider support shipped by Gunmetal.",
        after_help = PROVIDERS_LIST_HELP_FOOTER
    )]
    List,
}

#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    #[command(about = "Check auth state for one saved provider connection.")]
    Status {
        #[arg(help = "Saved provider name or id.")]
        profile: String,
    },
    #[command(about = "Start or resume auth for one saved provider connection.")]
    Login {
        #[arg(help = "Saved provider name or id.")]
        profile: String,
        #[arg(long, help = "Print the auth URL without opening a browser window.")]
        no_open: bool,
        #[arg(
            long,
            help = "Start auth and return immediately without waiting for completion."
        )]
        no_wait: bool,
    },
    #[command(about = "Log one saved provider connection out locally.")]
    Logout {
        #[arg(help = "Saved provider name or id.")]
        profile: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum LogCommand {
    #[command(
        about = "List recent requests with optional filters.",
        after_help = LOGS_LIST_HELP_FOOTER
    )]
    List {
        #[arg(
            long,
            default_value_t = 20,
            help = "Maximum number of recent requests to inspect."
        )]
        limit: usize,
        #[arg(long, help = "Filter by provider name.")]
        provider: Option<String>,
        #[arg(long, help = "Filter by model id.")]
        model: Option<String>,
        #[arg(
            long,
            help = "Free-text filter over provider, key, endpoint, mode, or error."
        )]
        query: Option<String>,
        #[arg(long, value_enum, help = "Filter by success or error requests.")]
        status: Option<LogStatus>,
    },
    #[command(about = "Summarize recent request traffic by provider and model.")]
    Summary {
        #[arg(
            long,
            default_value_t = 24,
            help = "Maximum number of recent requests to summarize."
        )]
        limit: usize,
    },
}

pub async fn execute(command: Command, paths: &AppPaths, mut output: impl Write) -> Result<()> {
    let providers = builtin_provider_hub(paths.clone());

    match command {
        Command::Setup(args) => {
            setup(paths, &providers, &mut output, args).await?;
        }
        Command::Chat(args) => {
            chat(paths, &mut output, args).await?;
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
                        "No models synced yet. Run `gunmetal setup` or `gunmetal models sync <saved-provider>`."
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
                    "synced {} models for provider {}",
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
                writeln!(output, "saved provider {}", profile.name)?;
                writeln!(output, "id: {}", profile.id)?;
            }
            ProfileCommand::List => {
                let storage = paths.storage_handle()?;
                let profiles = storage.list_profiles()?;
                let models = storage.list_models()?;
                if profiles.is_empty() {
                    writeln!(
                        output,
                        "No providers yet. Run `gunmetal setup` or `gunmetal profiles create ...`."
                    )?;
                }
                for profile in profiles {
                    let model_count = models
                        .iter()
                        .filter(|model| model.profile_id == Some(profile.id))
                        .count();
                    writeln!(
                        output,
                        "{} {} enabled={} models={} id={}",
                        profile.provider, profile.name, profile.enabled, model_count, profile.id
                    )?;
                }
            }
        },
        Command::Providers { command } => match command {
            ProviderCommand::List => {
                for provider in builtin_providers() {
                    writeln!(
                        output,
                        "{} {:?} auth={} modes={}{} priority={}",
                        provider.kind,
                        provider.class,
                        if provider.supports_browser_login() {
                            "browser"
                        } else {
                            "api_key"
                        },
                        [
                            provider
                                .capabilities
                                .supports_chat_completions
                                .then_some("chat/completions"),
                            provider
                                .capabilities
                                .supports_responses_api
                                .then_some("responses"),
                        ]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>()
                        .join("+"),
                        if provider.capabilities.supports_base_url {
                            " base_url"
                        } else {
                            ""
                        },
                        provider.priority
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
                    "Provider: {} ({})",
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
                let browser_login = supports_browser_login(&profile_record.provider);
                let session = if browser_login {
                    start_browser_auth_via_daemon(
                        paths,
                        profile_record.id,
                        DEFAULT_HOST.parse().expect("default host ip"),
                        DEFAULT_PORT,
                    )
                    .await?
                } else {
                    providers.login(&profile_record, !no_open).await?
                };
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
                if !no_open
                    && browser_login
                    && let Err(error) = webbrowser::open(&session.auth_url)
                {
                    writeln!(output, "Browser open failed: {error}")?;
                }
                if !no_wait && browser_login {
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
            LogCommand::List {
                limit,
                provider,
                model,
                query,
                status,
            } => {
                let storage = paths.storage_handle()?;
                let profiles = storage.list_profiles()?;
                let keys = storage.list_keys()?;
                let logs = storage
                    .list_request_logs(limit)?
                    .into_iter()
                    .filter(|log| {
                        log_matches_filters(
                            log,
                            provider.as_deref(),
                            model.as_deref(),
                            query.as_deref(),
                            status,
                            &profiles,
                            &keys,
                        )
                    })
                    .collect::<Vec<_>>();
                if logs.is_empty() {
                    writeln!(
                        output,
                        "No logs yet. Start Gunmetal with `gunmetal start`, then make one request."
                    )?;
                }
                for log in logs {
                    let profile_name = profiles
                        .iter()
                        .find(|profile| Some(profile.id) == log.profile_id)
                        .map(|profile| profile.name.as_str())
                        .unwrap_or("-");
                    let key_name = keys
                        .iter()
                        .find(|key| Some(key.id) == log.key_id)
                        .map(|key| key.name.as_str())
                        .unwrap_or("-");
                    writeln!(
                        output,
                        "{} {} {} {} {} {} {} {} {}ms in={} out={} total={}",
                        log.started_at,
                        log.provider,
                        request_mode_label(&log.endpoint),
                        profile_name,
                        key_name,
                        log.model,
                        log.endpoint,
                        log.status_code.unwrap_or_default(),
                        log.duration_ms,
                        log.usage.input_tokens.unwrap_or_default(),
                        log.usage.output_tokens.unwrap_or_default(),
                        log.usage.total_tokens.unwrap_or_default()
                    )?;
                }
            }
            LogCommand::Summary { limit } => {
                let logs = paths.storage_handle()?.list_request_logs(limit)?;
                if logs.is_empty() {
                    writeln!(
                        output,
                        "No logs yet. Start Gunmetal with `gunmetal start`, then make one request."
                    )?;
                } else {
                    write_log_summary(&mut output, &logs)?;
                }
            }
        },
    }

    Ok(())
}

async fn ensure_daemon_running(paths: &AppPaths, host: IpAddr, port: u16) -> Result<ServiceStatus> {
    let current = daemon_status(paths, host, port).await?;
    if current.running {
        ensure_daemon_matches_home(&current, paths)?;
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
    ensure_daemon_matches_home(&status, paths)?;
    if status.running {
        return Ok(ServiceStatus {
            note: Some("Gunmetal started.".to_owned()),
            ..status
        });
    }

    anyhow::bail!("{}", diagnose_start_failure(paths, port))
}

async fn ensure_default_daemon_running(paths: &AppPaths) -> Result<ServiceStatus> {
    ensure_daemon_running(
        paths,
        DEFAULT_HOST.parse::<IpAddr>().expect("default host"),
        DEFAULT_PORT,
    )
    .await
}

async fn stop_daemon(paths: &AppPaths, host: IpAddr, port: u16) -> Result<ServiceStatus> {
    let pid_file = paths.daemon_pid_file();
    let Some(pid) = managed_daemon_pid(paths)? else {
        let mut status = daemon_status(paths, host, port).await?;
        if status.running {
            status.note = Some(
                "Gunmetal is running, but not under managed daemon state. Stop the foreground `gunmetal serve` process directly.".to_owned(),
            );
            return Ok(status);
        }
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

async fn daemon_status(paths: &AppPaths, host: IpAddr, port: u16) -> Result<ServiceStatus> {
    let url = format!("http://{host}:{port}");
    let health_url = format!("{url}/health");
    let pid = managed_daemon_pid(paths)?;
    match reqwest::get(&health_url).await {
        Ok(response) => {
            let health = response.text().await.ok();
            let home = daemon_home(&url).await;
            Ok(ServiceStatus {
                state: "running".to_owned(),
                running: true,
                pid,
                url,
                health,
                home,
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
                        home: None,
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
                    home: None,
                    note: Some("Removed stale daemon state.".to_owned()),
                });
            }
            Ok(ServiceStatus {
                state: "stopped".to_owned(),
                running: false,
                pid: None,
                url,
                health: None,
                home: None,
                note: None,
            })
        }
    }
}

async fn daemon_home(url: &str) -> Option<String> {
    let response = reqwest::get(format!("{url}/app/api/state")).await.ok()?;
    let body = response.json::<serde_json::Value>().await.ok()?;
    body.get("service")
        .and_then(|service| service.get("home"))
        .and_then(|home| home.as_str())
        .map(ToOwned::to_owned)
}

async fn start_browser_auth_via_daemon(
    paths: &AppPaths,
    profile_id: Uuid,
    host: IpAddr,
    port: u16,
) -> Result<ProviderLoginSession> {
    let status = ensure_daemon_running(paths, host, port).await?;
    start_browser_auth_via_service(&status.url, profile_id).await
}

async fn start_browser_auth_via_service(
    service_url: &str,
    profile_id: Uuid,
) -> Result<ProviderLoginSession> {
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{service_url}/app/api/profiles/{profile_id}/auth"))
        .send()
        .await?;
    let response_status = response.status();
    let body = response.json::<serde_json::Value>().await?;
    parse_browser_auth_session(response_status, body)
}

fn parse_browser_auth_session(
    response_status: reqwest::StatusCode,
    body: serde_json::Value,
) -> Result<ProviderLoginSession> {
    if !response_status.is_success() {
        let message = body
            .get("error")
            .and_then(|value| value.get("message"))
            .and_then(|value| value.as_str())
            .unwrap_or("auth request failed");
        anyhow::bail!(message.to_owned());
    }

    Ok(ProviderLoginSession {
        login_id: "daemon-flow".to_owned(),
        auth_url: body
            .get("auth_url")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_owned(),
        user_code: body
            .get("user_code")
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned),
        interval_seconds: None,
    })
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

fn managed_daemon_pid(paths: &AppPaths) -> Result<Option<u32>> {
    let pid_file = paths.daemon_pid_file();
    let Some(pid) = read_pid(&pid_file)? else {
        return Ok(None);
    };
    if process_exists(pid) {
        return Ok(Some(pid));
    }
    let _ = fs::remove_file(pid_file);
    Ok(None)
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
struct ServiceStatus {
    state: String,
    running: bool,
    pid: Option<u32>,
    url: String,
    health: Option<String>,
    home: Option<String>,
    note: Option<String>,
}

fn ensure_daemon_matches_home(status: &ServiceStatus, paths: &AppPaths) -> Result<()> {
    let expected = paths.root.display().to_string();
    if let Some(home) = status.home.as_deref()
        && home != expected
    {
        anyhow::bail!(
            "port already serves Gunmetal home {}. stop it first or switch back to that home.",
            home
        );
    }
    Ok(())
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
        "This connects one provider, checks auth, syncs models, and creates one local key that works across providers."
    )?;
    writeln!(output)?;
    let provider = match args.provider {
        Some(provider) => provider,
        None => prompt_provider(output, interactive)?,
    };
    let name = prompt_or_value(
        output,
        interactive,
        "Provider name",
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
        "Saved provider {} ({})",
        profile.name, profile.provider
    )?;

    let browser_login = supports_browser_login(&provider);
    let mut auth_ready_for_sync = true;
    if browser_login {
        let session = start_browser_auth_via_daemon(
            paths,
            profile.id,
            DEFAULT_HOST.parse().expect("default host ip"),
            DEFAULT_PORT,
        )
        .await?;
        writeln!(output, "Open this URL to finish auth: {}", session.auth_url)?;
        if let Some(user_code) = session.user_code.clone() {
            writeln!(output, "User code: {user_code}")?;
        }
        if !args.no_open
            && let Err(error) = webbrowser::open(&session.auth_url)
        {
            writeln!(output, "Browser open failed: {error}")?;
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
            auth_ready_for_sync = false;
            writeln!(
                output,
                "Auth still needs to finish. Run `gunmetal auth status {}` when done.",
                profile.name
            )?;
        }
    } else {
        let status = providers.auth_status(&profile).await?;
        writeln!(output, "Auth: {}", status.label)?;
        auth_ready_for_sync = auth_state_is_connected(&status.state);
    }

    let mut models = Vec::new();
    if args.no_sync {
        writeln!(output, "Skipping model sync because `--no-sync` was set.")?;
    } else if auth_ready_for_sync {
        models = providers.sync_models(&profile).await?;
        storage.replace_models_for_profile(&profile.provider, Some(profile.id), &models)?;
        writeln!(output, "Synced {} models.", models.len())?;
    } else if browser_login {
        writeln!(
            output,
            "Skipping model sync until browser auth finishes for {}.",
            profile.name
        )?;
    } else {
        writeln!(
            output,
            "Skipping model sync until auth works for {}.",
            profile.name
        )?;
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
        "- provider saved: {} ({})",
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

struct ChatTurnResult {
    content: String,
    usage: Option<TokenUsage>,
    duration_ms: u64,
}

async fn chat(paths: &AppPaths, output: &mut impl Write, args: ChatArgs) -> Result<()> {
    let interactive = io::stdin().is_terminal() && args.prompt.is_none();
    let status = ensure_default_daemon_running(paths).await?;
    let storage = paths.storage_handle()?;
    let models = storage.list_models()?;
    if models.is_empty() {
        anyhow::bail!(
            "no synced models yet. run `gunmetal setup` or `gunmetal models sync <saved-provider>` first."
        );
    }

    let api_key = resolve_chat_api_key(output, interactive, args.api_key)?;
    let model = resolve_chat_model(output, interactive, args.model, &models)?;
    let client = reqwest::Client::new();
    let mut history = Vec::<ChatMessage>::new();

    writeln!(output, "Gunmetal chat")?;
    writeln!(output, "Base URL: {}/v1", status.url)?;
    writeln!(
        output,
        "Mode: {}",
        match args.mode {
            ChatMode::Chat => "chat/completions",
            ChatMode::Responses => "responses",
        }
    )?;
    writeln!(output, "Model: {model}")?;

    if let Some(prompt) = args.prompt {
        history.push(ChatMessage {
            role: ChatRole::User,
            content: prompt,
        });
        let result = run_chat_turn(
            &client,
            &status.url,
            &api_key,
            &model,
            args.mode,
            &history,
            output,
        )
        .await?;
        writeln!(output)?;
        write_chat_summary(output, &result)?;
        return Ok(());
    }

    writeln!(
        output,
        "Commands: /clear resets the conversation, /quit exits."
    )?;

    loop {
        let prompt = prompt_line_allow_empty(output, "you", None)?;
        let trimmed = prompt.trim();
        if trimmed.is_empty() {
            continue;
        }
        if matches!(trimmed, "/quit" | "/exit") {
            break;
        }
        if trimmed == "/clear" {
            history.clear();
            writeln!(output, "Conversation cleared.")?;
            continue;
        }

        history.push(ChatMessage {
            role: ChatRole::User,
            content: prompt,
        });
        let result = run_chat_turn(
            &client,
            &status.url,
            &api_key,
            &model,
            args.mode,
            &history,
            output,
        )
        .await?;
        writeln!(output)?;
        write_chat_summary(output, &result)?;
        history.push(ChatMessage {
            role: ChatRole::Assistant,
            content: result.content,
        });
    }

    Ok(())
}

fn resolve_chat_api_key(
    output: &mut impl Write,
    interactive: bool,
    value: Option<String>,
) -> Result<String> {
    match value
        .or_else(|| std::env::var("GUNMETAL_API_KEY").ok())
        .filter(|value| !value.trim().is_empty())
    {
        Some(value) => Ok(value),
        None if interactive => prompt_line(output, "Gunmetal key", None),
        None => anyhow::bail!(
            "missing Gunmetal key. pass `--api-key`, set `GUNMETAL_API_KEY`, or run interactively."
        ),
    }
}

fn resolve_chat_model(
    output: &mut impl Write,
    interactive: bool,
    value: Option<String>,
    models: &[ModelDescriptor],
) -> Result<String> {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        return Ok(value);
    }

    if models.len() == 1 {
        return Ok(models[0].id.clone());
    }

    if interactive {
        writeln!(output, "Synced models:")?;
        for model in models.iter().take(12) {
            writeln!(output, "- {}", model.id)?;
        }
        return prompt_line(output, "Model", Some(models[0].id.clone()));
    }

    anyhow::bail!(
        "missing model. pass `--model provider/model` or run interactively after syncing models."
    )
}

fn chat_request_payload(mode: ChatMode, model: &str, messages: &[ChatMessage]) -> Value {
    match mode {
        ChatMode::Chat => json!({
            "model": model,
            "stream": true,
            "messages": messages,
        }),
        ChatMode::Responses => json!({
            "model": model,
            "stream": true,
            "input": messages.iter().map(|message| {
                let role = match message.role {
                    ChatRole::System => "developer",
                    ChatRole::User => "user",
                    ChatRole::Assistant => "assistant",
                };
                json!({
                    "role": role,
                    "content": [{ "type": "input_text", "text": message.content }],
                })
            }).collect::<Vec<_>>(),
        }),
    }
}

async fn run_chat_turn(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    model: &str,
    mode: ChatMode,
    messages: &[ChatMessage],
    output: &mut impl Write,
) -> Result<ChatTurnResult> {
    let endpoint = match mode {
        ChatMode::Chat => "/v1/chat/completions",
        ChatMode::Responses => "/v1/responses",
    };
    let started_at = Instant::now();
    let response = client
        .post(format!("{base_url}{endpoint}"))
        .bearer_auth(api_key)
        .json(&chat_request_payload(mode, model, messages))
        .send()
        .await?;

    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        let message = serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|value| {
                value
                    .get("error")
                    .and_then(|error| error.get("message"))
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            })
            .unwrap_or_else(|| {
                if text.trim().is_empty() {
                    "request failed".to_owned()
                } else {
                    text
                }
            });
        anyhow::bail!(message);
    }

    write!(output, "assistant> ")?;
    output.flush()?;

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut content = String::new();
    let mut usage = None;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(boundary) = buffer.find("\n\n") {
            let raw_event = buffer[..boundary].to_owned();
            buffer = buffer[(boundary + 2)..].to_owned();
            let Some((event_name, data)) = parse_sse_event(&raw_event) else {
                continue;
            };
            if data == "[DONE]" {
                continue;
            }

            let parsed: Value = serde_json::from_str(&data)?;
            match mode {
                ChatMode::Chat => {
                    if let Some(delta) = parsed
                        .get("choices")
                        .and_then(|choices| choices.get(0))
                        .and_then(|choice| choice.get("delta"))
                        .and_then(|delta| delta.get("content"))
                        .and_then(Value::as_str)
                    {
                        write!(output, "{delta}")?;
                        output.flush()?;
                        content.push_str(delta);
                    }
                }
                ChatMode::Responses => {
                    if event_name == "response.output_text.delta" {
                        if let Some(delta) = parsed.get("delta").and_then(Value::as_str) {
                            write!(output, "{delta}")?;
                            output.flush()?;
                            content.push_str(delta);
                        }
                    } else if event_name == "response.completed" {
                        if let Some(text) = parsed
                            .get("response")
                            .and_then(|response| response.get("output_text"))
                            .and_then(Value::as_str)
                        {
                            content = text.to_owned();
                        }
                        usage = parsed
                            .get("response")
                            .and_then(|response| response.get("usage"))
                            .and_then(token_usage_from_value);
                    }
                }
            }
        }
    }

    Ok(ChatTurnResult {
        content,
        usage,
        duration_ms: started_at.elapsed().as_millis() as u64,
    })
}

fn parse_sse_event(raw_event: &str) -> Option<(String, String)> {
    let mut event_name = "message".to_owned();
    let mut data = Vec::new();
    for line in raw_event.lines() {
        if let Some(rest) = line.strip_prefix("event:") {
            event_name = rest.trim().to_owned();
        } else if let Some(rest) = line.strip_prefix("data:") {
            data.push(rest.trim_start().to_owned());
        }
    }

    (!data.is_empty()).then(|| (event_name, data.join("\n")))
}

fn token_usage_from_value(value: &Value) -> Option<TokenUsage> {
    Some(TokenUsage {
        input_tokens: value
            .get("input_tokens")
            .and_then(Value::as_u64)
            .map(|value| value as u32),
        output_tokens: value
            .get("output_tokens")
            .and_then(Value::as_u64)
            .map(|value| value as u32),
        total_tokens: value
            .get("total_tokens")
            .and_then(Value::as_u64)
            .map(|value| value as u32),
    })
}

fn write_chat_summary(output: &mut impl Write, result: &ChatTurnResult) -> Result<()> {
    let usage = result.usage.as_ref().map_or_else(
        || "tokens logged in request history".to_owned(),
        |usage| {
            format!(
                "tokens in {} · out {} · total {}",
                usage.input_tokens.unwrap_or_default(),
                usage.output_tokens.unwrap_or_default(),
                usage.total_tokens.unwrap_or_default()
            )
        },
    );
    writeln!(output, "{}", usage)?;
    writeln!(output, "latency {} ms", result.duration_ms)?;
    Ok(())
}

#[derive(Debug, Default)]
struct LogSummaryAccumulator {
    requests: usize,
    success_count: usize,
    error_count: usize,
    latency_total_ms: u64,
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
}

impl LogSummaryAccumulator {
    fn observe(&mut self, log: &RequestLogEntry) {
        self.requests += 1;
        if log_succeeded(log) {
            self.success_count += 1;
        }
        if log_failed(log) {
            self.error_count += 1;
        }
        self.latency_total_ms += log.duration_ms;
        self.input_tokens += u64::from(log.usage.input_tokens.unwrap_or_default());
        self.output_tokens += u64::from(log.usage.output_tokens.unwrap_or_default());
        self.total_tokens += u64::from(log.usage.total_tokens.unwrap_or_default());
    }

    fn avg_latency_ms(&self) -> Option<u64> {
        (self.requests > 0).then(|| self.latency_total_ms / self.requests as u64)
    }
}

fn log_matches_filters(
    log: &RequestLogEntry,
    provider: Option<&str>,
    model: Option<&str>,
    query: Option<&str>,
    status: Option<LogStatus>,
    profiles: &[gunmetal_core::ProviderProfile],
    keys: &[gunmetal_core::GunmetalKey],
) -> bool {
    let provider_matches = provider
        .map(|value| {
            let query = value.trim().to_ascii_lowercase();
            log.provider
                .to_string()
                .to_ascii_lowercase()
                .contains(&query)
        })
        .unwrap_or(true);
    let model_matches = model
        .map(|value| {
            let query = value.trim().to_ascii_lowercase();
            log.model.to_ascii_lowercase().contains(&query)
        })
        .unwrap_or(true);
    let status_matches = match status {
        Some(LogStatus::Success) => log_succeeded(log),
        Some(LogStatus::Error) => log_failed(log),
        None => true,
    };
    let profile_name = profiles
        .iter()
        .find(|profile| Some(profile.id) == log.profile_id)
        .map(|profile| profile.name.as_str())
        .unwrap_or("");
    let key_name = keys
        .iter()
        .find(|key| Some(key.id) == log.key_id)
        .map(|key| key.name.as_str())
        .unwrap_or("");
    let query_matches = query
        .map(|value| {
            let query = value.trim().to_ascii_lowercase();
            [
                log.provider.to_string(),
                profile_name.to_owned(),
                key_name.to_owned(),
                log.model.clone(),
                log.endpoint.clone(),
                request_mode_label(&log.endpoint).to_owned(),
                log.error_message.clone().unwrap_or_default(),
            ]
            .join(" ")
            .to_ascii_lowercase()
            .contains(&query)
        })
        .unwrap_or(true);
    provider_matches && model_matches && query_matches && status_matches
}

fn log_succeeded(log: &RequestLogEntry) -> bool {
    log.status_code.is_some_and(|code| code < 400) && log.error_message.is_none()
}

fn log_failed(log: &RequestLogEntry) -> bool {
    log.status_code.is_some_and(|code| code >= 400) || log.error_message.is_some()
}

fn request_mode_label(endpoint: &str) -> &'static str {
    if endpoint.contains("/responses") {
        "responses"
    } else if endpoint.contains("/chat/completions") {
        "chat/completions"
    } else {
        "request"
    }
}

fn write_log_summary(output: &mut impl Write, logs: &[RequestLogEntry]) -> Result<()> {
    let mut provider_summary = std::collections::BTreeMap::<String, LogSummaryAccumulator>::new();
    let mut model_summary = std::collections::BTreeMap::<String, LogSummaryAccumulator>::new();
    let mut overall = LogSummaryAccumulator::default();

    for log in logs {
        overall.observe(log);
        provider_summary
            .entry(log.provider.to_string())
            .or_default()
            .observe(log);
        model_summary
            .entry(log.model.clone())
            .or_default()
            .observe(log);
    }

    writeln!(
        output,
        "recent={} success={} errors={} avg_latency={}ms tokens_in={} tokens_out={} tokens_total={}",
        overall.requests,
        overall.success_count,
        overall.error_count,
        overall.avg_latency_ms().unwrap_or_default(),
        overall.input_tokens,
        overall.output_tokens,
        overall.total_tokens
    )?;

    let mut provider_rows = provider_summary.into_iter().collect::<Vec<_>>();
    provider_rows.sort_by(|left, right| {
        right
            .1
            .total_tokens
            .cmp(&left.1.total_tokens)
            .then_with(|| right.1.requests.cmp(&left.1.requests))
            .then_with(|| left.0.cmp(&right.0))
    });
    writeln!(output, "providers:")?;
    for (provider, summary) in provider_rows.into_iter().take(5) {
        writeln!(
            output,
            "  {} req={} tokens={} errors={} avg={}ms",
            provider,
            summary.requests,
            summary.total_tokens,
            summary.error_count,
            summary.avg_latency_ms().unwrap_or_default()
        )?;
    }

    let mut model_rows = model_summary.into_iter().collect::<Vec<_>>();
    model_rows.sort_by(|left, right| {
        right
            .1
            .total_tokens
            .cmp(&left.1.total_tokens)
            .then_with(|| right.1.requests.cmp(&left.1.requests))
            .then_with(|| left.0.cmp(&right.0))
    });
    writeln!(output, "models:")?;
    for (model, summary) in model_rows.into_iter().take(5) {
        writeln!(
            output,
            "  {} req={} tokens={} errors={} avg={}ms",
            model,
            summary.requests,
            summary.total_tokens,
            summary.error_count,
            summary.avg_latency_ms().unwrap_or_default()
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
        "authentication did not finish in time for provider '{}'. finish in the browser, then run `gunmetal auth status {}` or `gunmetal auth login {}` again",
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
            "provider '{}' not found. run `gunmetal profiles list` or `gunmetal setup`.",
            selector
        ),
        _ => anyhow::bail!(
            "provider '{}' is ambiguous. use the id from `gunmetal profiles list`.",
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

fn auth_state_is_connected(state: &ProviderAuthState) -> bool {
    matches!(state, ProviderAuthState::Connected)
}

fn supports_browser_login(provider: &ProviderKind) -> bool {
    provider_definition(provider).is_some_and(|definition| definition.supports_browser_login())
}

fn needs_api_key(provider: &ProviderKind) -> bool {
    provider_definition(provider).is_some_and(|definition| definition.requires_api_key())
}

fn supports_base_url(provider: &ProviderKind) -> bool {
    provider_definition(provider)
        .is_some_and(|definition| definition.capabilities.supports_base_url)
}

fn provider_definition(provider: &ProviderKind) -> Option<gunmetal_sdk::ProviderDefinition> {
    builtin_providers()
        .into_iter()
        .find(|definition| &definition.kind == provider)
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
        ServiceVerb::Stop if status.state == "stopping" => {
            writeln!(output, "Gunmetal is still stopping.")?
        }
        ServiceVerb::Stop if status.running => writeln!(output, "Gunmetal is running.")?,
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
        writeln!(output, "Next: run `gunmetal start` or `gunmetal web`.")?;
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
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    use async_trait::async_trait;
    use clap::{CommandFactory, Parser};
    use gunmetal_core::{ProviderAuthState, ProviderAuthStatus, ProviderKind, ProviderProfile};
    use gunmetal_sdk::{
        ProviderAdapter, ProviderAuthResult, ProviderChatResult, ProviderClass, ProviderDefinition,
        ProviderLoginResult, ProviderModelSyncResult, ProviderRegistry,
    };
    use gunmetal_storage::AppPaths;
    use tempfile::TempDir;

    use super::{
        AuthCommand, ChatArgs, ChatMode, Cli, Command, KeyCommand, LogCommand, ModelCommand,
        ProfileCommand, SetupArgs, StatusArgs, execute, provider_definition_fixture,
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
    fn defaults_to_no_command_for_help() {
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
        let cli = Cli::parse_from([
            "gunmetal",
            "logs",
            "list",
            "--limit",
            "12",
            "--provider",
            "codex",
            "--model",
            "gpt-5",
            "--query",
            "timeout",
            "--status",
            "error",
        ]);

        match cli.command.unwrap() {
            Command::Logs {
                command:
                    LogCommand::List {
                        limit,
                        provider,
                        model,
                        query,
                        status,
                    },
            } => {
                assert_eq!(limit, 12);
                assert_eq!(provider.as_deref(), Some("codex"));
                assert_eq!(model.as_deref(), Some("gpt-5"));
                assert_eq!(query.as_deref(), Some("timeout"));
                assert_eq!(status, Some(super::LogStatus::Error));
            }
            _ => panic!("unexpected command"),
        }
    }

    #[test]
    fn parses_logs_summary_command() {
        let cli = Cli::parse_from(["gunmetal", "logs", "summary", "--limit", "30"]);

        match cli.command.unwrap() {
            Command::Logs {
                command: LogCommand::Summary { limit },
            } => assert_eq!(limit, 30),
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
    fn parses_chat_command() {
        let cli = Cli::parse_from([
            "gunmetal",
            "chat",
            "--api-key",
            "gm_test",
            "--model",
            "openai/gpt-5.4",
            "--mode",
            "responses",
            "--prompt",
            "say ok",
        ]);

        match cli.command.unwrap() {
            Command::Chat(ChatArgs {
                api_key,
                model,
                mode,
                prompt,
            }) => {
                assert_eq!(api_key.as_deref(), Some("gm_test"));
                assert_eq!(model.as_deref(), Some("openai/gpt-5.4"));
                assert_eq!(mode, ChatMode::Responses);
                assert_eq!(prompt.as_deref(), Some("say ok"));
            }
            _ => panic!("unexpected command"),
        }
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
        assert!(help.contains("gunmetal chat"));
        assert!(help.contains("gunmetal logs summary"));
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
    fn service_and_logs_help_explain_public_paths() {
        let mut command = Cli::command();
        let web = command.find_subcommand_mut("web").expect("web subcommand");
        let web_help = web.render_help().to_string();
        assert!(web_help.contains("Open the local browser UI"));
        assert!(web_help.contains("http://127.0.0.1:4684/app"));

        let mut command = Cli::command();
        let start = command
            .find_subcommand_mut("start")
            .expect("start subcommand");
        let start_help = start.render_help().to_string();
        assert!(start_help.contains("Start the local Gunmetal API in the background."));
        assert!(start_help.contains("gunmetal web"));
        assert!(!start_help.contains("gunmetal tui"));

        let mut command = Cli::command();
        let status = command
            .find_subcommand_mut("status")
            .expect("status subcommand");
        let status_help = status.render_help().to_string();
        assert!(status_help.contains("Check whether the local Gunmetal service is live."));
        assert!(status_help.contains("gunmetal start"));

        let mut command = Cli::command();
        let logs = command
            .find_subcommand_mut("logs")
            .expect("logs subcommand");
        let list = logs
            .find_subcommand_mut("list")
            .expect("logs list subcommand");
        let logs_help = list.render_help().to_string();
        assert!(logs_help.contains("List recent requests with optional filters."));
        assert!(logs_help.contains("--query"));
        assert!(logs_help.contains("timeout"));
    }

    #[test]
    fn providers_help_points_at_builtin_support() {
        let mut command = Cli::command();
        let providers = command
            .find_subcommand_mut("providers")
            .expect("providers subcommand");
        let list = providers
            .find_subcommand_mut("list")
            .expect("providers list subcommand");
        let help = list.render_help().to_string();

        assert!(help.contains("List built-in provider support shipped by Gunmetal."));
        assert!(help.contains("gunmetal profiles list"));
    }

    #[test]
    fn stop_timeout_keeps_pid_and_running_state() {
        let status = super::ServiceStatus {
            state: "running".to_owned(),
            running: true,
            pid: Some(42),
            url: "http://127.0.0.1:4684".to_owned(),
            health: Some("{\"status\":\"ok\"}".to_owned()),
            home: None,
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

    #[test]
    fn managed_daemon_pid_clears_stale_pid_files() {
        let temp = TempDir::new().unwrap();
        let paths =
            gunmetal_storage::AppPaths::from_root(temp.path().join("gunmetal-home")).unwrap();
        std::fs::write(paths.daemon_pid_file(), "999999").unwrap();

        let pid = super::managed_daemon_pid(&paths).unwrap();

        assert_eq!(pid, None);
        assert!(!paths.daemon_pid_file().exists());
    }

    #[tokio::test]
    async fn status_output_tells_user_how_to_recover_when_daemon_is_stopped() {
        let temp = TempDir::new().unwrap();
        let paths =
            gunmetal_storage::AppPaths::from_root(temp.path().join("gunmetal-home")).unwrap();
        let mut output = Vec::new();
        let port = 46849;

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
        assert!(text.contains("Next: run `gunmetal start` or `gunmetal web`."));
        assert!(text.contains(&format!("http://127.0.0.1:{port}/v1")));
    }

    #[test]
    fn stop_output_for_unmanaged_running_daemon_does_not_claim_shutdown() {
        let mut output = Vec::new();

        super::write_service_report(
            &mut output,
            &super::ServiceStatus {
                state: "running".to_owned(),
                running: true,
                pid: None,
                url: "http://127.0.0.1:4684".to_owned(),
                health: Some("{\"status\":\"ok\"}".to_owned()),
                home: None,
                note: Some(
                    "Gunmetal is running, but not under managed daemon state. Stop the foreground `gunmetal serve` process directly.".to_owned(),
                ),
            },
            super::ServiceVerb::Stop,
        )
        .unwrap();

        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("Gunmetal is running."));
        assert!(!text.contains("Gunmetal is still stopping."));
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
                command: LogCommand::List {
                    limit: 20,
                    provider: None,
                    model: None,
                    query: None,
                    status: None,
                },
            },
        ] {
            let mut output = Vec::new();
            execute(command, &paths, &mut output).await.unwrap();
            let text = String::from_utf8(output).unwrap();
            assert!(text.contains("gunmetal setup") || text.contains("gunmetal start"));
        }
    }

    #[tokio::test]
    async fn missing_provider_errors_tell_user_how_to_recover() {
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
        assert!(message.contains("provider 'missing' not found"));
        assert!(message.contains("gunmetal setup"));
        assert!(message.contains("gunmetal profiles list"));
    }

    #[test]
    fn daemon_home_mismatch_is_reported_clearly() {
        let temp = TempDir::new().unwrap();
        let paths =
            gunmetal_storage::AppPaths::from_root(temp.path().join("gunmetal-home")).unwrap();
        let error = super::ensure_daemon_matches_home(
            &super::ServiceStatus {
                state: "running".to_owned(),
                running: true,
                pid: Some(7),
                url: "http://127.0.0.1:4684".to_owned(),
                health: Some("{\"status\":\"ok\"}".to_owned()),
                home: Some("/tmp/other-home".to_owned()),
                note: None,
            },
            &paths,
        )
        .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("port already serves Gunmetal home /tmp/other-home")
        );
    }

    #[test]
    fn daemon_home_match_or_unknown_is_allowed() {
        let temp = TempDir::new().unwrap();
        let paths =
            gunmetal_storage::AppPaths::from_root(temp.path().join("gunmetal-home")).unwrap();

        super::ensure_daemon_matches_home(
            &super::ServiceStatus {
                state: "running".to_owned(),
                running: true,
                pid: Some(7),
                url: "http://127.0.0.1:4684".to_owned(),
                health: Some("{\"status\":\"ok\"}".to_owned()),
                home: Some(paths.root.display().to_string()),
                note: None,
            },
            &paths,
        )
        .unwrap();

        super::ensure_daemon_matches_home(
            &super::ServiceStatus {
                state: "running".to_owned(),
                running: true,
                pid: Some(7),
                url: "http://127.0.0.1:4684".to_owned(),
                health: Some("{\"status\":\"ok\"}".to_owned()),
                home: None,
                note: None,
            },
            &paths,
        )
        .unwrap();
    }

    #[derive(Clone)]
    struct SetupAuthGateAdapter {
        sync_called: Arc<AtomicBool>,
    }

    #[async_trait]
    impl ProviderAdapter for SetupAuthGateAdapter {
        fn definition(&self) -> ProviderDefinition {
            provider_definition_fixture(ProviderKind::OpenRouter, ProviderClass::Gateway, 1)
        }

        async fn auth_status(
            &self,
            _profile: &ProviderProfile,
            _paths: &AppPaths,
        ) -> anyhow::Result<ProviderAuthResult> {
            Ok(ProviderAuthResult {
                credentials: None,
                status: ProviderAuthStatus {
                    state: ProviderAuthState::SignedOut,
                    label: "User not found.".to_owned(),
                },
            })
        }

        async fn login(
            &self,
            _profile: &ProviderProfile,
            _paths: &AppPaths,
            _open_browser: bool,
        ) -> anyhow::Result<ProviderLoginResult> {
            anyhow::bail!("browser login not used in this test")
        }

        async fn logout(
            &self,
            _profile: &ProviderProfile,
            _paths: &AppPaths,
        ) -> anyhow::Result<Option<serde_json::Value>> {
            Ok(None)
        }

        async fn sync_models(
            &self,
            _profile: &ProviderProfile,
            _paths: &AppPaths,
        ) -> anyhow::Result<ProviderModelSyncResult> {
            self.sync_called.store(true, Ordering::SeqCst);
            anyhow::bail!("sync should have been skipped")
        }

        async fn chat_completion(
            &self,
            _profile: &ProviderProfile,
            _paths: &AppPaths,
            _request: &gunmetal_core::ChatCompletionRequest,
        ) -> anyhow::Result<ProviderChatResult> {
            anyhow::bail!("chat not used in this test")
        }
    }

    #[tokio::test]
    async fn setup_skips_sync_when_auth_is_not_connected() {
        let temp = TempDir::new().unwrap();
        let paths =
            gunmetal_storage::AppPaths::from_root(temp.path().join("gunmetal-home")).unwrap();
        let sync_called = Arc::new(AtomicBool::new(false));
        let mut registry = ProviderRegistry::default();
        registry.register(SetupAuthGateAdapter {
            sync_called: sync_called.clone(),
        });
        let providers = gunmetal_providers::ProviderHub::with_registry(paths.clone(), registry);
        let mut output = Vec::new();

        super::setup(
            &paths,
            &providers,
            &mut output,
            SetupArgs {
                provider: Some(ProviderKind::OpenRouter),
                name: Some("gateway".to_owned()),
                base_url: None,
                api_key: Some("bad-key".to_owned()),
                key_name: None,
                bin_path: None,
                cwd: None,
                http_referer: None,
                title: None,
                no_open: false,
                no_wait: false,
                no_sync: false,
                no_key: true,
            },
        )
        .await
        .unwrap();

        let text = String::from_utf8(output).unwrap();
        assert!(text.contains("Auth: User not found."));
        assert!(text.contains("Skipping model sync until auth works for gateway."));
        assert!(!sync_called.load(Ordering::SeqCst));

        let storage = paths.storage_handle().unwrap();
        let profiles = storage.list_profiles().unwrap();
        let models = storage.list_models().unwrap();
        assert_eq!(profiles.len(), 1);
        assert!(models.is_empty());
    }

    #[tokio::test]
    async fn browser_auth_service_parses_daemon_session() {
        let session = super::parse_browser_auth_session(
            reqwest::StatusCode::OK,
            serde_json::json!({
                "auth_url": "https://example.com/auth",
                "user_code": "ABCD-EFGH",
            }),
        )
        .unwrap();
        assert_eq!(session.login_id, "daemon-flow");
        assert_eq!(session.auth_url, "https://example.com/auth");
        assert_eq!(session.user_code.as_deref(), Some("ABCD-EFGH"));
    }
}
