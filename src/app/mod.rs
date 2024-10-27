pub mod mondrian_command;

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
