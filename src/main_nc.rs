#![windows_subsystem = "windows"]
mod win32 {
    pub mod api {
        pub mod accessibility;
        pub mod cursor;
        pub mod key;
        pub mod misc;
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
    pub mod mondrian_command;
    pub mod win_events_handlers {
        pub mod maximize_event_handler;
        pub mod minimize_event_handler;
        pub mod open_event_handler;
        pub mod position_event_handler;
    }
    pub mod tiles_manager {
        pub mod config;
        pub(super) mod container;
        pub(super) mod containers_manager;
        pub mod manager;
        pub mod monitor_layout;
        pub mod tm_command;
        pub mod window_animator;
    }
    pub mod structs {
        pub mod area_tree {
            pub mod layout_strategy;
            pub mod leaf;
            pub mod node;
            pub mod tree;
        }
        pub mod area;
        pub mod direction;
        pub mod orientation;
        pub mod point;
    }
    pub mod config {
        pub mod app_configs;
        pub mod cli_args;
        pub mod win_matcher;
    }
}

mod modules {
    pub(super) mod module;
    pub mod overlays {
        pub mod configs;
        pub(crate) mod lib;
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
        pub mod configs;
        pub mod module;
    }
}

mod app_main;

fn main() {
    app_main::main();
}
