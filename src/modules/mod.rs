pub mod events_monitor {
    pub mod configs;
    pub mod module;
    pub(crate) mod lib {
        pub mod filter;
        pub mod focus_event_handler;
        pub mod maximize_event_handler;
        pub mod minimize_event_handler;
        pub mod open_event_handler;
        pub mod position_event_handler;
        pub mod system_events_detector;
    }
}

pub mod tiles_manager {
    pub mod configs;
    pub mod module;
    pub(crate) mod lib {
        pub mod containers;
        pub mod structs;
        pub mod tm;
        pub mod utils;
        pub mod window_animation_player;
    }
}

pub mod keybindings {
    pub mod configs;
    pub mod module;
}

pub mod overlays {
    pub mod configs;
    pub mod module;
    pub(crate) mod lib {
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

pub mod logger {
    pub mod module;
}

pub mod file_watcher {
    pub mod module;
}

use enum_dispatch::enum_dispatch;
use events_monitor::module::EventsMonitor;
use file_watcher::module::FileWatcher;
use keybindings::module::Keybindings;
use logger::module::Logger;
use overlays::module::Overlays;
use tiles_manager::module::TilesManagerModule;
use tray::module::Tray;

use crate::app::{configs::AppConfig, mondrian_message::MondrianMessage};

#[enum_dispatch(ModuleEnum)]
pub trait Module {
    fn start(&mut self);
    fn stop(&mut self);
    fn restart(&mut self);
    fn enable(&mut self, enabled: bool);
    fn handle(&mut self, event: &MondrianMessage, app_configs: &AppConfig);
    fn pause(&mut self, is_paused: bool);
    fn name(&self) -> String;
}

pub trait ConfigurableModule: Module {
    type Config;
    fn configure(&mut self, config: Self::Config);
}

pub(in crate::modules) mod module_impl {
    use crate::app::configs::AppConfig;
    use crate::app::mondrian_message::MondrianMessage;

    use super::Module;

    pub trait ModuleImpl {
        fn start(&mut self);
        fn stop(&mut self);
        fn restart(&mut self);
        fn handle(&mut self, event: &MondrianMessage, app_configs: &AppConfig);
        fn enabled(&self) -> bool;
        fn enable(&mut self, enabled: bool);
        fn pause(&mut self, is_paused: bool);
        fn name(&self) -> String;
    }

    impl<T: ModuleImpl> Module for T {
        fn start(&mut self) {
            if self.enabled() {
                ModuleImpl::start(self);
            }
        }

        fn stop(&mut self) {
            if self.enabled() {
                ModuleImpl::stop(self);
            }
        }

        fn restart(&mut self) {
            if self.enabled() {
                ModuleImpl::restart(self);
            }
        }

        fn enable(&mut self, enabled: bool) {
            ModuleImpl::enable(self, enabled);
            if !enabled {
                ModuleImpl::stop(self);
            }
        }

        fn handle(&mut self, event: &MondrianMessage, app_configs: &AppConfig) {
            ModuleImpl::handle(self, event, app_configs);
        }

        fn pause(&mut self, is_paused: bool) {
            ModuleImpl::pause(self, is_paused);
        }

        fn name(&self) -> String {
            ModuleImpl::name(self)
        }
    }
}

#[enum_dispatch]
pub enum ModuleEnum {
    EventsMonitor,
    TilesManagerModule,
    Overlays,
    Tray,
    Keybindings,
    Logger,
    FileWatcher,
}
