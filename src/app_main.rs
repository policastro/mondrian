use crate::app::app_lock::AppLock;
use crate::app::assets::Asset;
use crate::app::cli_args::CliArgs;
use crate::app::configs::AppConfig;
use crate::app::mondrian_message::MondrianMessage;
use crate::app::structs::info_entry::{InfoEntry, InfoEntryIcon};
use crate::modules::events_monitor::module::EventsMonitor;
use crate::modules::file_watcher::module::FileWatcher;
use crate::modules::keybindings::module::Keybindings;
use crate::modules::logger::module::Logger;
use crate::modules::overlays::module::Overlays;
use crate::modules::tiles_manager::module::TilesManagerModule;
use crate::modules::tray::module::Tray;
use crate::modules::{Module, ModuleEnum};
use crate::win32::api::gdiplus::{init_gdiplus, shutdown_gdiplus};
use crate::win32::api::monitor::enum_display_monitors;
use clap::Parser;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
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
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use windows::Win32::UI::HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2};

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
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
    init_gdiplus();

    let config = load_configs(cfg_file)
        .inspect_err(|e| log::error!("Can't read config file: {}", e))
        .unwrap_or_default();
    let shared_config = Arc::new(RwLock::new(config));

    let (bus_tx, bus_rx) = crossbeam_channel::unbounded();
    let modules: Vec<ModuleEnum> = vec![
        Logger::new().into(),
        EventsMonitor::new(bus_tx.clone()).into(),
        TilesManagerModule::new(bus_tx.clone()).into(),
        Overlays::new(bus_tx.clone()).into(),
        Tray::new(bus_tx.clone()).into(),
        Keybindings::new(bus_tx.clone()).into(),
        FileWatcher::new(bus_tx.clone(), cfg_file).into(),
    ];

    let mut modules_map: HashMap<String, (Sender<MondrianMessage>, JoinHandle<()>)> = HashMap::new();

    for mut m in modules.into_iter() {
        let module_name = m.name().to_lowercase();
        if modules_map.contains_key(&module_name) {
            panic!("Module '{module_name}' is already registered")
        }

        let local_shared_config = shared_config.clone();
        let (tx, rx) = crossbeam_channel::unbounded();
        let th = thread::spawn(move || module_handler(&mut m, local_shared_config, rx));

        modules_map.insert(module_name, (tx, th));
    }

    if dump_info {
        bus_tx.send(MondrianMessage::QueryInfo).ok();
    }

    event_dispatcher(bus_rx, bus_tx, modules_map, shared_config, cfg_file);
    shutdown_gdiplus();

    log::info!("Application stopped!");
}

fn load_configs(app_cfg_file: &PathBuf) -> Result<AppConfig, String> {
    if !app_cfg_file.parent().unwrap().exists() {
        std::fs::create_dir_all(app_cfg_file.parent().unwrap()).unwrap();
    }

    if !app_cfg_file.exists() {
        let default_cfg = Asset::get_string("configs/mondrian.toml").map_err(|e| e.to_string())?;
        std::fs::write(app_cfg_file, default_cfg).map_err(|e| e.to_string())?;
    }

    let file_content = std::fs::read_to_string(app_cfg_file).expect("Something went wrong reading the file");
    toml::from_str::<AppConfig>(&file_content).map_err(|e| e.to_string())
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
    let configs = load_configs(cfg_file);
    let configs_info = configs
        .map(|_| "Ok".to_string())
        .unwrap_or_else(|e| format!("ERROR - {:?}", e));
    let infos = InfoEntry::simple("Configuration", configs_info).with_icon(InfoEntryIcon::Configs);
    vec![infos]
}

fn module_handler(m: &mut ModuleEnum, shared_config: Arc<RwLock<AppConfig>>, rx: Receiver<MondrianMessage>) {
    let mut config = shared_config.read().unwrap().clone();

    m.handle(&MondrianMessage::Configure, &config);
    m.start();
    log::info!("Module '{}' started", m.name());

    loop {
        let event = if let Ok(e) = rx.recv() { e } else { continue };
        if matches!(event, MondrianMessage::RefreshConfig) {
            config = shared_config.read().unwrap().clone();
        }
        m.handle(&event, &config);
        if matches!(event, MondrianMessage::Quit) {
            log::info!("Module '{}' stopped", m.name());
            break;
        }
    }
}

fn event_dispatcher(
    bus_rx: Receiver<MondrianMessage>,
    bus_tx: Sender<MondrianMessage>,
    mut modules_map: HashMap<String, (Sender<MondrianMessage>, JoinHandle<()>)>,
    shared_config: Arc<RwLock<AppConfig>>,
    cfg_file: &PathBuf,
) {
    loop {
        let event = if let Ok(e) = bus_rx.recv() { e } else { continue };

        match &event {
            MondrianMessage::QueryInfo => {
                bus_tx
                    .send(MondrianMessage::QueryInfoResponse {
                        name: "General".to_string(),
                        icon: InfoEntryIcon::General,
                        infos: get_info_entries(cfg_file),
                    })
                    .ok();
            }
            MondrianMessage::RefreshConfig => {
                if let Ok(c) = load_configs(cfg_file).inspect_err(|e| log::error!("Can't read config file: {}", e)) {
                    shared_config.write().unwrap().clone_from(&c);
                }
            }
            _ => (),
        }

        match &event {
            MondrianMessage::OpenConfig => drop(open::that(cfg_file.clone())),
            MondrianMessage::OpenLogFolder => drop(open::that("logs")),
            MondrianMessage::About => drop(open::that("https://github.com/policastro/mondrian")),
            MondrianMessage::PauseModule(name, pause) => {
                if let Some((tx, _)) = modules_map.get_mut(name) {
                    tx.send(MondrianMessage::Pause(*pause)).ok();
                };
            }
            event => modules_map.values_mut().for_each(|(tx, _)| {
                tx.send(event.clone()).ok();
            }),
        }

        if matches!(event, MondrianMessage::Quit) {
            for (_, m) in modules_map.into_iter() {
                m.1.join().ok();
            }
            break;
        }
    }
}
