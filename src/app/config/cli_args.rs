use std::path::PathBuf;

use clap::Parser;

fn default_config_path() -> PathBuf {
    dirs::home_dir()
        .expect("Failed to get home dir")
        .join(".config")
        .join("mondrian")
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[arg(short = 'c', long="config_path", help = "Path to the config file", default_value = default_config_path().into_os_string())]
    pub config_path: PathBuf,
}
