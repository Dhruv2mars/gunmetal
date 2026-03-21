use anyhow::Result;
use clap::Parser;
use gunmetal_cli::{Cli, Command};
use gunmetal_storage::AppPaths;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let paths = AppPaths::resolve()?;

    match cli.command {
        Some(Command::Tui) | None => gunmetal_tui::run(&paths)?,
        Some(command) => gunmetal_cli::execute(command, &paths, std::io::stdout()).await?,
    }

    Ok(())
}
