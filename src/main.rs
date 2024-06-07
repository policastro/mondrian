//#![windows_subsystem = "windows"]

mod win32utils {
    pub mod api {
        pub mod cursor;
        pub mod key;
        pub mod monitor;
        pub mod window;
        pub mod callbacks {
            pub mod enum_windows;
            pub mod win_event_hook;
        }
    }
    pub mod window {
        pub mod window_obj;
        pub mod window_ref;
        pub mod window_snapshot;
    }
    pub mod win_event_loop;
    pub mod win_events_manager;
}

mod app {
    pub mod app_event;
    pub mod globals;
    pub mod input_binder;
    pub mod mondrian_app;
    pub mod tiles_manager;
    pub mod win_events_handlers {
        pub mod minimize_event_handler;
        pub mod open_event_handler;
        pub mod position_event_handler;
    }
    pub mod structs {
        pub mod area_tree {
            pub mod layout {
                pub mod golden_ration_layout;
                pub mod layout_strategy;
                pub mod mono_axis_layout;
            }
            pub mod leaf;
            pub mod node;
            pub mod tree;
        }
        pub mod area;
        pub mod direction;
        pub mod monitors_layout;
        pub mod orientation;
    }
    pub mod config {
        pub mod app_configs;
        pub mod cli_args;
        pub mod filters {
            pub mod window_filter;
            pub mod window_filter_type;
            pub mod window_match_filter;
        }
    }
}

use std::path::PathBuf;

use crate::app::config::app_configs::AppConfigs;

use app::{config::cli_args::CliArgs, mondrian_app::MondrianApp};
use clap::Parser;
use log::info;
use tray_icon::{
    menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};
use win32utils::win_event_loop::next_win_event_loop_iteration;

fn main() {
    let args = CliArgs::parse();

    let log_config_file = args.config_path.join("mondrian.log.yml");
    let app_config_file = args.config_path.join("mondrian.toml");

    create_configurations_file(&app_config_file, &log_config_file);

    log4rs::init_file(log_config_file, Default::default()).unwrap();
    let app_config = AppConfigs::from_file(&app_config_file);

    let (_tray, pause_menu_item) = create_tray_icon();

    let mut app = MondrianApp::new(app_config);
    app.run(true);

    info!("Application started!");

    while next_win_event_loop_iteration(None) {
        let event_id = MenuEvent::receiver()
            .try_recv()
            .map_or(None, |e| Some(e.id.0.to_owned()));

        match event_id.as_ref().map(|id| id.as_str()) {
            Some("PAUSE") => app.pause(pause_menu_item.is_checked(), true),
            Some("RETILE") => app.restart(true, None),
            Some("REFRESH_CONFIG") => app.restart(true, AppConfigs::from_file(&app_config_file).into()),
            Some("OPEN_CONFIG") => {
                let _ = open::that(app_config_file.clone());
            }
            Some("QUIT") => {
                app.stop();
                break;
            }
            Some(_) | None => {}
        }
    }

    //inputbot::handle_input_events();
    info!("Application stopped!");
}

fn create_configurations_file(app_config_file: &PathBuf, log_config_file: &PathBuf) {
    if !app_config_file.parent().unwrap().exists() {
        std::fs::create_dir_all(app_config_file.parent().unwrap()).unwrap();
    }

    if !log_config_file.parent().unwrap().exists() {
        std::fs::create_dir_all(log_config_file.parent().unwrap()).unwrap();
    }

    if !app_config_file.exists() {
        let bytes = include_bytes!("../assets/example_config/mondrian.toml");
        std::fs::write(app_config_file, bytes).unwrap();
    }

    if !log_config_file.exists() {
        let bytes = include_bytes!("../assets/example_config/mondrian.log.yml");
        std::fs::write(log_config_file, bytes).unwrap();
    }
}

fn create_tray_icon() -> (TrayIcon, CheckMenuItem) {
    let icon = Icon::from_resource_name("APP_ICON", Some((256, 256))).unwrap();

    let tray_menu = Menu::new();
    let retile = MenuItem::with_id("RETILE", "Retile", true, None);
    let open_config = MenuItem::with_id("OPEN_CONFIG", "Open config file", true, None);
    let refresh_config = MenuItem::with_id("REFRESH_CONFIG", "Refresh config", true, None);
    let pause = tray_icon::menu::CheckMenuItem::with_id("PAUSE", "Pause", true, false, None);
    let separator = PredefinedMenuItem::separator();
    let quit = MenuItem::with_id("QUIT", "Quit", true, None);

    let _ = tray_menu.append_items(&[&retile, &open_config, &refresh_config, &pause, &separator, &quit]);

    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Mondrian")
        .with_icon(icon)
        .build()
        .unwrap();

    (tray_icon, pause)
}
