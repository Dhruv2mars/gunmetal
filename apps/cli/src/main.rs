use anyhow::Result;
use clap::Parser;
use gunmetal_cli::{Cli, Command};
use gunmetal_storage::AppPaths;
use gunmetal_tui::ServiceSnapshot;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let paths = AppPaths::resolve()?;

    match cli.command {
        Some(Command::Tui) | None => {
            let runtime = tokio::runtime::Runtime::new()?;
            let service = runtime.block_on(gunmetal_cli::ensure_default_daemon_running(&paths))?;
            drop(runtime);
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
        Some(command) => {
            let runtime = tokio::runtime::Runtime::new()?;
            runtime.block_on(gunmetal_cli::execute(command, &paths, std::io::stdout()))?
        }
    }

    Ok(())
}
