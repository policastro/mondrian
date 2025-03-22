use crate::app::app_lock::AppLock;
use crate::app::assets::Asset;
use crate::app::cli_args::CliArgs;
use crate::app::configs::AppConfigs;
use crate::app::mondrian_message::MondrianMessage;
use crate::app::structs::info_entry::{InfoEntry, InfoEntryIcon};
use crate::modules::events_monitor::module::EventsMonitorModule;
use crate::modules::file_watcher::module::FileWatcherModule;
use crate::modules::keybindings::module::KeybindingsModule;
use crate::modules::logger::module::LoggerModule;
use crate::modules::overlays::module::OverlaysModule;
use crate::modules::tiles_manager::module::TilesManagerModule;
use crate::modules::tray::module::TrayModule;
use crate::modules::Module;
use crate::win32::api::gdiplus::{init_gdiplus, shutdown_gdiplus};
use crate::win32::api::monitor::enum_display_monitors;
use clap::Parser;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::threshold::ThresholdFilter;
use std::collections::HashMap;
use std::path::PathBuf;

pub fn main() {
    let args = CliArgs::parse();

    init_logger(
        args.is_file_all_enabled(),
        args.is_file_error_enabled(),
        args.get_log_level(),
    );

    let _app_lock = match AppLock::init() {
        Ok(v) => v,
        Err(_) => {
            log::error!("Mondrian is already running!");
            std::process::exit(1);
        }
    };

    let cfg_file = dirs::home_dir()
        .expect("Failed to get home dir")
        .join(".config/mondrian/mondrian.toml");

    log_info();
    start_app(&cfg_file, args.dump_info);
}

fn start_app(cfg_file: &PathBuf, dump_info: bool) {
    init_gdiplus();

    let mut configs = match init_configs(cfg_file) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to initialize configs: {}", e);
            log::warn!("Using default configs ...");
            AppConfigs::default()
        }
    };

    let (bus_tx, bus_rx) = std::sync::mpsc::channel();

    let modules: Vec<Box<dyn Module>> = vec![
        Box::new(LoggerModule::new()),
        Box::new(EventsMonitorModule::new(bus_tx.clone())),
        Box::new(TilesManagerModule::new(bus_tx.clone())),
        Box::new(OverlaysModule::new(bus_tx.clone())),
        Box::new(TrayModule::new(bus_tx.clone())),
        Box::new(KeybindingsModule::new(bus_tx.clone())),
        Box::new(FileWatcherModule::new(bus_tx.clone(), cfg_file)),
    ];

    let mut modules_map: HashMap<String, Box<dyn Module>> = HashMap::new();
    for m in modules {
        if let Some(m) = modules_map.insert(m.name().to_lowercase(), m) {
            panic!("Module '{}' is already registered", m.name())
        }
    }

    log::info!("Starting modules ...");

    modules_map.values_mut().for_each(|m| {
        m.handle(&MondrianMessage::Configure, &configs);
        m.start();
        log::info!("Module '{}' started", m.name());
    });

    if dump_info {
        bus_tx.send(MondrianMessage::QueryInfo).ok();
    }

    log::info!("Application started!");
    loop {
        let event = if let Ok(e) = bus_rx.recv() { e } else { continue };

        if event == MondrianMessage::QueryInfo {
            bus_tx
                .send(MondrianMessage::QueryInfoResponse {
                    name: "General".to_string(),
                    icon: InfoEntryIcon::General,
                    infos: get_info_entries(cfg_file),
                })
                .ok();
        }

        match event.clone() {
            MondrianMessage::OpenConfig => drop(open::that(cfg_file.clone())),
            MondrianMessage::OpenLogFolder => drop(open::that("logs")),
            MondrianMessage::About => drop(open::that("https://github.com/policastro/mondrian")),
            MondrianMessage::RefreshConfig => {
                configs = init_configs(cfg_file).unwrap_or_else(|e| {
                    log::error!("Can't read config file: {}", e);
                    configs.clone()
                });
                modules_map.values_mut().for_each(|m| m.handle(&event, &configs));
            }
            MondrianMessage::PauseModule(name, m) => {
                if let Some(module) = modules_map.get_mut(&name) {
                    module.handle(&MondrianMessage::Pause(m), &configs)
                };
            }
            event => modules_map.values_mut().for_each(|m| m.handle(&event, &configs)),
        }

        if event == MondrianMessage::Quit {
            break;
        }
    }

    shutdown_gdiplus();

    log::info!("Application stopped!");
}

fn init_configs(app_cfg_file: &PathBuf) -> Result<AppConfigs, String> {
    if !app_cfg_file.parent().unwrap().exists() {
        std::fs::create_dir_all(app_cfg_file.parent().unwrap()).unwrap();
    }

    if !app_cfg_file.exists() {
        let default_cfg = Asset::get_string("configs/mondrian.toml").map_err(|e| e.to_string())?;
        std::fs::write(app_cfg_file, default_cfg).map_err(|e| e.to_string())?;
    }

    let file_content = std::fs::read_to_string(app_cfg_file).expect("Something went wrong reading the file");
    toml::from_str::<AppConfigs>(&file_content).map_err(|e| e.to_string())
}

fn init_logger(file_all: bool, file_errors: bool, level: log::LevelFilter) {
    let pattern = PatternEncoder::new("{h({d(%Y-%m-%d %H:%M:%S)} {({l}):5.5} {f}:{L})}: {m}{n}");
    let console: ConsoleAppender = ConsoleAppender::builder().encoder(Box::new(pattern.clone())).build();

    const FILE_SIZE: u64 = 10 * 1024 * 1024; // INFO: 10 MB
    const NUM_FILES: u32 = 3;

    let file_all_policy = CompoundPolicy::new(
        Box::new(SizeTrigger::new(FILE_SIZE)),
        Box::new(
            FixedWindowRoller::builder()
                .build("./logs/mondrian.{}.gz", NUM_FILES)
                .expect("Failed to create roller"),
        ),
    );
    let log_file_all: RollingFileAppender = RollingFileAppender::builder()
        .encoder(Box::new(pattern.clone()))
        .build("./logs/mondrian.log", Box::new(file_all_policy))
        .unwrap();

    let file_errors_policy = CompoundPolicy::new(
        Box::new(SizeTrigger::new(FILE_SIZE)),
        Box::new(
            FixedWindowRoller::builder()
                .build("./logs/errors.{}.gz", NUM_FILES)
                .expect("Failed to create roller"),
        ),
    );
    let log_file_errors: RollingFileAppender = RollingFileAppender::builder()
        .encoder(Box::new(pattern.clone()))
        .build("./logs/errors.log", Box::new(file_errors_policy))
        .unwrap();

    let mut root_builder = Root::builder().appender("console");

    if file_all {
        root_builder = root_builder.appender("file_all");
    }

    if file_errors {
        root_builder = root_builder.appender("file_errors");
    }

    let config = log4rs::config::Config::builder()
        .appender(Appender::builder().build("console", Box::new(console)))
        .appender(Appender::builder().build("file_all", Box::new(log_file_all)))
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(LevelFilter::Error)))
                .build("file_errors", Box::new(log_file_errors)),
        )
        .build(root_builder.build(level))
        .unwrap();

    let _log_config = log4rs::init_config(config).unwrap();

    log_panics::init();
}

fn log_info() {
    for m in enum_display_monitors() {
        log::info!(
            "Monitor detected {{ ID: {}, primary: {}, resolution: {} x {} }}",
            m.id,
            if m.primary { "Yes" } else { "No" },
            m.resolution.0,
            m.resolution.1
        );
    }
}

fn get_info_entries(cfg_file: &PathBuf) -> Vec<InfoEntry> {
    let configs = init_configs(cfg_file);
    let configs_info = configs
        .map(|_| "Ok".to_string())
        .unwrap_or_else(|e| format!("ERROR - {:?}", e));
    let infos = InfoEntry::simple("Configuration", configs_info).with_icon(InfoEntryIcon::Configs);
    vec![infos]
}
