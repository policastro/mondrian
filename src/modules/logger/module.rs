use crate::app::config::app_configs::AppConfigs;
use crate::app::mondrian_message::MondrianMessage;
use crate::modules::module_impl::ModuleImpl;

pub struct LoggerModule;

impl ModuleImpl for LoggerModule {
    fn start(&mut self) {}
    fn stop(&mut self) {}
    fn restart(&mut self) {}
    fn pause(&mut self, _is_paused: bool) {}
    fn enable(&mut self, _enabled: bool) {}
    fn enabled(&self) -> bool {
        true
    }

    fn handle(&mut self, event: &MondrianMessage, _app_configs: &AppConfigs) {
        log::trace!("{:?}", event);
    }

    fn name(&self) -> String {
        "logger".to_string()
    }
}
