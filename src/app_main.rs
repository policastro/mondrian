use crate::app::config::app_configs::AppConfigs;
use crate::app::config::cli_args::CliArgs;
use crate::app::mondrian_message::MondrianMessage;
use crate::modules::events_monitor::module::EventsMonitorModule;
use crate::modules::keybindings::module::KeybindingsModule;
use crate::modules::logger::module::LoggerModule;
use crate::modules::overlays::module::OverlaysModule;
use crate::modules::tiles_manager::module::TilesManagerModule;
use crate::modules::tray::module::TrayModule;
use crate::modules::Module;
use clap::Parser;
use log4rs::config::RawConfig;
use std::collections::HashMap;
use std::path::PathBuf;

pub fn main() {
    let args = CliArgs::parse();

    init_logger(args.log);

    let cfg_file = dirs::home_dir()
        .expect("Failed to get home dir")
        .join(".config/mondrian/mondrian.toml");

    start_app(&cfg_file);
}

fn start_app(cfg_file: &PathBuf) {
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
        Box::new(LoggerModule {}),
        Box::new(EventsMonitorModule::new(bus_tx.clone())),
        Box::new(TilesManagerModule::new(bus_tx.clone())),
        Box::new(OverlaysModule::new(bus_tx.clone())),
        Box::new(TrayModule::new(bus_tx.clone())),
        Box::new(KeybindingsModule::new(bus_tx.clone())),
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
    });

    log::info!("Application started!");
    loop {
        let event = if let Ok(e) = bus_rx.recv() { e } else { continue };

        match event.clone() {
            MondrianMessage::OpenConfig => drop(open::that(cfg_file.clone())),
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

    log::info!("Application stopped!");
}

fn init_configs(app_cfg_file: &PathBuf) -> Result<AppConfigs, String> {
    if !app_cfg_file.parent().unwrap().exists() {
        std::fs::create_dir_all(app_cfg_file.parent().unwrap()).unwrap();
    }

    if !app_cfg_file.exists() {
        std::fs::write(app_cfg_file, include_bytes!("../assets/configs/mondrian.toml")).unwrap();
    }
    let file_content = std::fs::read_to_string(app_cfg_file).expect("Something went wrong reading the file");
    toml::from_str::<AppConfigs>(&file_content).map_err(|e| e.to_string())
}

fn init_logger(enable_file: bool) {
    let config: RawConfig = match enable_file {
        true => serde_yaml::from_str(include_str!("../assets/configs/mondrian.log.yml")).unwrap(),
        false => serde_yaml::from_str(include_str!("../assets/configs/mondrian_nofile.log.yml")).unwrap(),
    };
    log4rs::init_raw_config(config).unwrap();
    log_panics::init();
}
