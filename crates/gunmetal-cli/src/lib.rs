use std::{
    io::Write,
    net::{IpAddr, SocketAddr},
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use gunmetal_core::{KeyScope, KeyState, NewGunmetalKey, ProviderKind};
use gunmetal_daemon::DaemonState;
use gunmetal_providers::builtin_providers;
use gunmetal_storage::AppPaths;

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
    Providers {
        #[command(subcommand)]
        command: ProviderCommand,
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
}

#[derive(Debug, Subcommand)]
pub enum ProviderCommand {
    List,
}

pub async fn execute(command: Command, paths: &AppPaths, mut output: impl Write) -> Result<()> {
    match command {
        Command::Serve(args) => {
            let address = SocketAddr::new(args.host, args.port);
            writeln!(output, "Serving gunmetal on http://{address}")?;
            gunmetal_daemon::serve(address, DaemonState::new(paths.storage_handle()?)).await?;
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
        Command::Tui => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Cli, Command, KeyCommand};

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
}
