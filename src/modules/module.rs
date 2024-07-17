use crate::app::{config::app_configs::AppConfigs, mondrian_command::MondrianCommand};

pub trait Module {
    fn start(&mut self);
    fn stop(&mut self);
    fn restart(&mut self);
    fn enable(&mut self, enabled: bool);
    fn handle(&mut self, event: &MondrianCommand, app_configs: &AppConfigs);
    fn pause(&mut self, is_paused: bool);
}
pub trait ConfigurableModule: Module {
    type Config;
    fn configure(&mut self, config: Self::Config);
}

pub(in crate::modules) mod module_impl {
    use crate::app::{config::app_configs::AppConfigs, mondrian_command::MondrianCommand};

    use super::Module;

    pub trait ModuleImpl {
        fn start(&mut self);
        fn stop(&mut self);
        fn restart(&mut self);
        fn handle(&mut self, event: &MondrianCommand, app_configs: &AppConfigs);
        fn enabled(&self) -> bool;
        fn enable(&mut self, enabled: bool);
        fn pause(&mut self, is_paused: bool);
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

        fn handle(&mut self, event: &MondrianCommand, app_configs: &AppConfigs) {
            ModuleImpl::handle(self, event, app_configs);
        }

        fn pause(&mut self, is_paused: bool) {
            ModuleImpl::pause(self, is_paused);
        }
    }
}
