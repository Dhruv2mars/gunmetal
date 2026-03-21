use std::{
    io::Write,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use gunmetal_core::{KeyScope, KeyState, NewGunmetalKey, NewProviderProfile, ProviderKind};
use gunmetal_daemon::DaemonState;
use gunmetal_providers::{ProviderHub, builtin_providers};
use gunmetal_storage::AppPaths;
use serde_json::{Map, Value, json};

#[derive(Debug, Parser)]
#[command(name = "gunmetal")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Serve(ServeArgs),
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
    Tui,
}

#[derive(Debug, clap::Args)]
pub struct ServeArgs {
    #[arg(long, default_value = "127.0.0.1")]
    pub host: IpAddr,
    #[arg(long, default_value_t = 4684)]
    pub port: u16,
}

#[derive(Debug, clap::Args)]
pub struct StatusArgs {
    #[arg(long, default_value = "127.0.0.1")]
    pub host: IpAddr,
    #[arg(long, default_value_t = 4684)]
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

pub async fn execute(command: Command, paths: &AppPaths, mut output: impl Write) -> Result<()> {
    let providers = ProviderHub::new(paths.clone());

    match command {
        Command::Serve(args) => {
            let address = SocketAddr::new(args.host, args.port);
            writeln!(output, "Serving gunmetal on http://{address}")?;
            gunmetal_daemon::serve(address, DaemonState::new(paths.clone())?).await?;
        }
        Command::Status(args) => {
            let url = format!("http://{}:{}/health", args.host, args.port);
            match reqwest::get(&url).await {
                Ok(response) => {
                    let body = response.text().await?;
                    writeln!(output, "{body}")?;
                }
                Err(error) => {
                    writeln!(output, "gunmetal not reachable at {url}: {error}")?;
                }
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
        Command::Tui => {}
    }

    Ok(())
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

    use super::{Cli, Command, KeyCommand, ModelCommand, ProfileCommand};

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
}
