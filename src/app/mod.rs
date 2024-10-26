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
    pub mod window_animation_player;
}

pub mod area_tree {
    pub mod layout_strategy;
    pub mod leaf;
    pub mod node;
    pub mod tree;
}

pub mod structs {
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
