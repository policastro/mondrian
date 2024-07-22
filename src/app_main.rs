use clap::Parser;
use log4rs::config::RawConfig;
use std::path::PathBuf;

use crate::app::config::cli_args::CliArgs;
use crate::app::config::app_configs::AppConfigs;
use crate::app::mondrian_command::MondrianMessage;
use crate::modules::core::module::CoreModule;
use crate::modules::keybindings::module::KeybindingsModule;
use crate::modules::module::Module;
use crate::modules::overlays::module::OverlaysModule;
use crate::modules::tray::module::TrayModule;

pub fn main() {
    init_logger();

    let args = CliArgs::parse();
    let cfg_file = args.config_path.join("mondrian.toml");

    start_app(&cfg_file);
}

fn start_app(cfg_file: &PathBuf) {
    let mut configs = match init_configs(cfg_file) {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to initialize configs: {}", e);
            log::warn!("Using default configs ...");
            AppConfigs::default()
        },
    };

    let (bus_tx, bus_rx) = std::sync::mpsc::channel();

    let mut modules: Vec<Box<dyn Module>> = vec![
        Box::new(OverlaysModule::new()),
        Box::new(CoreModule::new(bus_tx.clone())),
        Box::new(TrayModule::new(bus_tx.clone())),
        Box::new(KeybindingsModule::new(bus_tx.clone())),
    ];

    log::info!("Starting modules ...");

    modules.iter_mut().for_each(|m| {
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
                modules.iter_mut().for_each(|m| m.handle(&event, &configs));
            }
            event => modules.iter_mut().for_each(|m| m.handle(&event, &configs)),
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

fn init_logger() {
    let config: RawConfig = serde_yaml::from_str(include_str!("../assets/configs/mondrian.log.yml")).unwrap();
    log4rs::init_raw_config(config).unwrap();
    log_panics::init();
}
