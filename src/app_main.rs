use clap::Parser;
use log4rs::config::RawConfig;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use crate::app::config::app_configs::AppConfigs;
use crate::app::config::cli_args::CliArgs;
use crate::app::mondrian_command::MondrianCommand;
use crate::modules::core::module::CoreModule;
use crate::modules::keybindings::module::KeybindingsModule;
use crate::modules::module::Module;
use crate::modules::overlay::module::OverlayModule;
use crate::modules::tray::module::TrayModule;

pub fn main() {
    init_logger();

    let args = CliArgs::parse();
    let cfg_file = args.config_path.join("mondrian.toml");

    start_app(&cfg_file);
}

fn start_app(cfg_file: &PathBuf) {
    let mut app_configs = match init_configs(cfg_file) {
        Ok(c) => c,
        Err(e) => panic!("Failed to initialize configs: {}", e),
    };

    let (bus_tx, bus_rx) = std::sync::mpsc::channel();

    let mut modules: Vec<Box<dyn Module>> = vec![
        Box::new(CoreModule::new()),
        Box::new(TrayModule::new(bus_tx.clone())),
        Box::new(KeybindingsModule::new(bus_tx.clone())),
        Box::new(OverlayModule::new()),
    ];
    modules.iter_mut().for_each(|m| {
        m.handle(&MondrianCommand::Configure, &app_configs);
        m.start();
        thread::sleep(Duration::from_millis(20)); // TODO Magic number, find a better way to start the overlay only after the core module is ready (i.e. has placed the initial windows on the screen)
    });

    log::info!("Application started!");
    loop {
        let event = if let Ok(e) = bus_rx.recv() { e } else { continue };

        match event {
            MondrianCommand::OpenConfig => drop(open::that(cfg_file.clone())),
            MondrianCommand::RefreshConfig => {
                app_configs = init_configs(cfg_file).unwrap_or_else(|e| {
                    log::error!("Can't read config file: {}", e);
                    app_configs.clone()
                });
                modules.iter_mut().for_each(|m| m.handle(&event, &app_configs));
            }
            event => modules.iter_mut().for_each(|m| m.handle(&event, &app_configs)),
        }

        if event == MondrianCommand::Quit {
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

    match AppConfigs::from_file(app_cfg_file) {
        Ok(c) => Ok(c),
        Err(e) => Err(e.to_string()),
    }
}

fn init_logger() {
    let config: RawConfig = serde_yaml::from_str(include_str!("../assets/configs/mondrian.log.yml")).unwrap();
    log4rs::init_raw_config(config).unwrap();
    log_panics::init();
}
