mod app;
mod cli;
mod config;
mod render;
mod writer;

use clap::Parser;
use cli::Cli;
use config::load_config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli: Cli = Cli::parse();

    let config = load_config(&cli.configs)?;

    app::run(&config)?;

    Ok(())
}
