//#![windows_subsystem = "windows"]
mod win32 {
    pub mod api {
        pub mod accessibility;
        pub mod cursor;
        pub mod key;
        pub mod monitor;
        pub mod window;
    }
    pub mod callbacks {
        pub mod enum_monitors;
        pub mod enum_windows;
        pub mod win_event_hook;
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
    pub mod globals;
    pub mod mondrian_app_event;
    pub mod win32_event;
    pub mod win_events_handlers {
        pub mod minimize_event_handler;
        pub mod open_event_handler;
        pub mod position_event_handler;
    }
    pub mod tiles_manager {
        pub mod config;
        pub(super) mod container;
        pub(super) mod containers_manager;
        pub(super) mod managed_monitor;
        pub(super) mod managed_window;
        pub mod manager;
        pub mod monitor_layout;
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
        pub mod orientation;
    }
    pub mod config {
        pub mod app_configs;
        pub mod cli_args;
        pub(super) mod ext_configs;
        pub mod filters {
            pub mod window_filter;
            pub mod window_filter_type;
            pub mod window_match_filter;
        }
    }
}

mod modules {
    pub(super) mod module;
    pub mod overlay {
        pub mod lib {
            pub mod color;
            pub mod focus_event_handler;
            pub mod utils;
            pub mod window_overlay;
        }
        pub mod configs;
        pub mod module;
    }
    pub mod tray {
        pub mod module;
    }
    pub mod core {
        pub mod config;
        pub mod module;
    }
    pub mod keybindings {
        pub mod module;
    }
}

mod app_main;

fn main() {
    app_main::main();
}