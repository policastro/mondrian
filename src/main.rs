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

use crate::app::config::app_configs::AppConfigs;

use app::{config::cli_args::CliArgs, mondrian_app::MondrianApp};
use clap::Parser;
use log::info;
use win32utils::win_event_loop::run_win_event_loop;

fn main() {
    info!("Application started!");

    let app_config = get_app_configs();
    let mut app = MondrianApp::new(app_config);

    app.run(true);

    run_win_event_loop();
    //inputbot::handle_input_events();
    info!("Application stopped!");
}

fn get_app_configs() -> AppConfigs {
    let args = CliArgs::parse();

    let config_path = args.config_path.map_or(
        dirs::home_dir()
            .expect("Failed to get home dir")
            .join(".config")
            .join("mondrian"),
        |c| c,
    );
    let config_file = config_path.join("mondrian.json");
    let config_log_file = config_path.join("mondrian.log.yml");

    info!("Application started!");

    log4rs::init_file(config_log_file, Default::default()).unwrap();
    let app_configs = AppConfigs::from_file(config_file.to_str().expect("Failed to get config path"));

    app_configs
}

//sleep(std::time::Duration::from_millis(2000));
//
//sleep(std::time::Duration::from_millis(10000));
//app.stop();

//let mut tray_menu = Menu::new();
//
//let quit_i = MenuItem::new("Quit", true, None);
//let _ = tray_menu.append_items(&[&PredefinedMenuItem::separator(), &quit_i]);
//
//let mut tray_icon = Some(
//    TrayIconBuilder::new()
//        .with_menu(Box::new(tray_menu))
//        .with_tooltip("tao - awesome windowing lib")
//        //.with_icon(icon)
//        .build()
//        .unwrap(),
//);
