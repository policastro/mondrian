use clap::Parser;

fn less_than_2(l: &str) -> Result<u8, String> {
    const ERROR_MSG: &str = "LOG_TYPE must be 0 (no log file), 1 (error log file) or 2 (all log files)";
    let log_type = l.parse::<u8>().map_err(|_| ERROR_MSG)?;
    if log_type <= 2 {
        Ok(log_type)
    } else {
        Err(ERROR_MSG.to_owned())
    }
}

fn valid_log_level(l: &str) -> Result<u8, String> {
    const ERROR_MSG: &str = "LOGLEVEL must be 0 (off), 1 (trace), 2 (debug), 3 (info), 4 (warn) or 5 (error)";
    let log_level = l.parse::<u8>().map_err(|_| ERROR_MSG)?;
    if log_level <= 5 {
        Ok(log_level)
    } else {
        Err(ERROR_MSG.to_owned())
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    #[arg(id = "LOG_TYPE",  long = "log", help = "Can be 0 (no log file is created), 1 (error log file is created) or 2 (all log files are created)", value_parser=less_than_2, default_value_t = 1)]
    log_enabled: u8,

    #[arg( long = "loglevel", help = "Can be 0 (off), 1 (trace), 2 (debug), 3 (info), 4 (warn) or 5 (error)", value_parser=valid_log_level, default_value_t = 3)]
    log_level: u8,

    #[arg(
        long = "dumpstateinfo",
        help = "Dump app state info to a file",
        default_value_t = false
    )]
    pub dump_info: bool,

    #[arg(long = "healthcheck", help = "Enable health check", default_value_t = false)]
    pub health_check: bool,
}

impl CliArgs {
    pub fn is_file_all_enabled(&self) -> bool {
        self.log_enabled == 2
    }

    pub fn is_file_error_enabled(&self) -> bool {
        self.log_enabled >= 1
    }

    pub fn get_log_level(&self) -> log::LevelFilter {
        match self.log_level {
            0 => log::LevelFilter::Off,
            1 => log::LevelFilter::Trace,
            2 => log::LevelFilter::Debug,
            3 => log::LevelFilter::Info,
            4 => log::LevelFilter::Warn,
            5 => log::LevelFilter::Error,
            _ => log::LevelFilter::Info,
        }
    }
}
