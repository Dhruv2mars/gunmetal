use anyhow::Result;
use clap::Parser;
use gunmetal_cli::{Cli, Command};
use gunmetal_storage::AppPaths;
use gunmetal_tui::ServiceSnapshot;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let paths = AppPaths::resolve()?;

    match cli.command {
        Some(Command::Tui) | None => {
            let service = gunmetal_cli::ensure_default_daemon_running(&paths).await?;
            gunmetal_tui::run(
                &paths,
                ServiceSnapshot {
                    state: service.state,
                    running: service.running,
                    url: service.url,
                    pid: service.pid,
                },
            )?
        }
        Some(command) => gunmetal_cli::execute(command, &paths, std::io::stdout()).await?,
    }

    Ok(())
}
