use anyhow::Result;
use clap::{CommandFactory, Parser};
use gunmetal_cli::Cli;
use gunmetal_storage::AppPaths;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let Some(command) = cli.command else {
        let mut command = Cli::command();
        command.print_help()?;
        println!();
        return Ok(());
    };

    let paths = AppPaths::resolve()?;
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(gunmetal_cli::execute(command, &paths, std::io::stdout()))?;

    Ok(())
}
