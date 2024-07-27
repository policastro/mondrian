use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[arg(short = 'l', long = "log", help = "Enables logging to file", default_value_t = false)]
    pub log: bool,
}
