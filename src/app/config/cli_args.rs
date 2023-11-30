use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[arg(short = 'c', long="config_path", help = "Path to the config file")]
    pub config_path: Option<std::path::PathBuf>,
}