use clap::Parser;
use log4rs::config::RawConfig;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use crate::app::config::app_configs::AppConfigs;
use crate::app::config::cli_args::CliArgs;
use crate::app::mondrian_app_event::MondrianAppEvent;
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
    let app_configs = &init_configs(cfg_file);

    let (bus_tx, bus_rx) = std::sync::mpsc::channel();

    let mut core = CoreModule::new();
    let mut overlay = OverlayModule::new();
    let mut keybindings = KeybindingsModule::new();
    let mut tray = TrayModule::new(bus_tx.clone());

    overlay.enable(app_configs.overlay_enabled);
    keybindings.enable(app_configs.keybinds_enabled);

    core.configure(app_configs.into());
    overlay.configure(app_configs.into());

    core.start();
    tray.start();
    keybindings.start();
    thread::sleep(Duration::from_millis(50)); // TODO Magic number, find a better way to start the overlay only after the core module is ready (i.e. has placed the initial windows on the screen)
    overlay.start();

    log::info!("Application started!");
    loop {
        let event = match bus_rx.recv() {
            Ok(e) => e,
            Err(_) => continue,
        };

        match event {
            MondrianAppEvent::Pause(pause) => {
                core.pause(pause);
                overlay.pause(pause);
                keybindings.pause(pause);
            }
            MondrianAppEvent::Retile => core.restart(),
            MondrianAppEvent::RefreshConfig => {
                let app_configs = &init_configs(cfg_file);
                core.configure(app_configs.into());
                core.restart();

                overlay.enable(app_configs.overlay_enabled);
                overlay.configure(app_configs.into());
                overlay.restart();
            }
            MondrianAppEvent::OpenConfig => drop(open::that(cfg_file.clone())),
            MondrianAppEvent::Quit => break,
        }
    }

    core.stop();
    overlay.stop();
    tray.stop();
    keybindings.stop();

    log::info!("Application stopped!");
}

fn init_configs(app_cfg_file: &PathBuf) -> AppConfigs {
    if !app_cfg_file.parent().unwrap().exists() {
        std::fs::create_dir_all(app_cfg_file.parent().unwrap()).unwrap();
    }

    if !app_cfg_file.exists() {
        std::fs::write(app_cfg_file, include_bytes!("../assets/configs/mondrian.toml")).unwrap();
    }

    return AppConfigs::from_file(app_cfg_file);
}

fn init_logger() {
    let config: RawConfig = serde_yaml::from_str(include_str!("../assets/configs/mondrian.log.yml")).unwrap();
    log4rs::init_raw_config(config).unwrap();
}