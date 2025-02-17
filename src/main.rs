#![cfg_attr(feature = "noconsole", windows_subsystem = "windows")]

mod app;
mod app_main;
mod modules;
mod win32;

fn main() {
    app_main::main();
}
