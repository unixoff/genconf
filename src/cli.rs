use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about = "YAML config reader")]
pub struct Cli {
    #[arg(
        short,
        long = "config",
        value_name = "CONFIG",
        action = clap::ArgAction::Append,
        help = "YAML config file. Can be repeated; later configs override earlier ones"
    )]
    pub configs: Vec<PathBuf>,
}
