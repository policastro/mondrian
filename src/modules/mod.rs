pub mod module;

pub mod core {
    pub mod config;
    pub mod module;
}

pub mod keybindings {
    pub mod configs;
    pub mod module;
}

pub mod overlays {
    pub mod configs;
    pub mod module;
    pub mod lib {
        pub mod color;
        pub mod overlay;
        pub mod overlay_manager;
        pub mod overlays_event_handler;
        pub mod utils;
    }
}

pub mod tray {
    pub mod module;
}
