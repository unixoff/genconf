use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about = "YAML config reader")]
pub struct Cli {
    #[arg(short, long, default_value = "values.yaml")]
    pub config: PathBuf,
}
