mod app;
mod cli;
mod config;
mod render;
mod writer;

use clap::Parser;
use cli::Cli;
use config::Config;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli: Cli = Cli::parse();

    let content: String = fs::read_to_string(&cli.config)?;
    let config: Config = serde_yaml_ng::from_str(&content)?;

    app::run(&config)?;

    Ok(())
}
