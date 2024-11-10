use super::configs::EventMonitorModuleConfigs;
use super::lib::display_change_detector;
use super::lib::maximize_event_handler::MaximizeEventHandler;
use super::lib::minimize_event_handler::MinimizeEventHandler;
use super::lib::open_event_handler::OpenCloseEventHandler;
use super::lib::position_event_handler::PositionEventHandler;
use crate::app::config::app_configs::AppConfigs;
use crate::app::mondrian_message::MondrianMessage;
use crate::modules::module_impl::ModuleImpl;
use crate::modules::ConfigurableModule;
use crate::modules::Module;
use crate::win32::api::misc::{get_current_thread_id, post_empty_thread_message};
use crate::win32::api::window::destroy_window;
use crate::win32::win_events_manager::WindowsEventManager;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;
use windows::Win32::UI::WindowsAndMessaging::WM_QUIT;

pub struct EventsMonitorModule {
    win_events_thread_id: Arc<AtomicU32>,
    win_events_thread: Option<thread::JoinHandle<()>>,
    configs: EventMonitorModuleConfigs,
    running: Arc<AtomicBool>,
    enabled: bool,
    bus_tx: Sender<MondrianMessage>,
}

impl EventsMonitorModule {
    pub fn new(bus_tx: Sender<MondrianMessage>) -> Self {
        EventsMonitorModule {
            configs: EventMonitorModuleConfigs::default(),
            win_events_thread: None,
            win_events_thread_id: Arc::new(AtomicU32::new(0)),
            running: Arc::new(AtomicBool::new(false)),
            enabled: true,
            bus_tx,
        }
    }

    fn start_win_events_loop(&mut self) {
        let detect_maximized_windows = self.configs.detect_maximized_windows;
        let win_events_thread_id = self.win_events_thread_id.clone();
        let bus_tx = self.bus_tx.clone();
        let filter = self.configs.filter.clone().unwrap();
        self.win_events_thread = Some(thread::spawn(move || {
            let hwnd = display_change_detector::create(bus_tx.clone());
            if hwnd.is_none() {
                log::warn!("Failure while trying to create the monitor events detector window");
            }

            win_events_thread_id.store(get_current_thread_id(), Ordering::SeqCst);
            let mut wem = WindowsEventManager::new();
            wem.hook(PositionEventHandler::new(bus_tx.clone(), filter.clone()));
            wem.hook(OpenCloseEventHandler::new(bus_tx.clone(), filter.clone()));
            wem.hook(MinimizeEventHandler::new(bus_tx.clone(), filter.clone()));
            if detect_maximized_windows {
                wem.hook(MaximizeEventHandler::new(bus_tx.clone(), filter.clone()));
            }

            wem.start_event_loop();

            if let Some(hwnd) = hwnd {
                destroy_window(hwnd);
            }
        }));
    }
}

impl ModuleImpl for EventsMonitorModule {
    fn start(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }

        self.running.store(true, Ordering::SeqCst);
        self.start_win_events_loop();

        log::trace!("EventsMonitorModule started");
    }

    fn stop(&mut self) {
        if !self.running.load(Ordering::SeqCst) {
            return;
        }

        self.running.store(false, Ordering::SeqCst);
        if let Some(thread) = self.win_events_thread.take() {
            post_empty_thread_message(self.win_events_thread_id.load(Ordering::SeqCst), WM_QUIT);
            thread.join().unwrap();
        };

        log::trace!("EventsMonitorModule stopped");
    }

    fn restart(&mut self) {
        Module::stop(self);
        Module::start(self);
    }

    fn pause(&mut self, is_paused: bool) {
        match is_paused {
            true => Module::stop(self),
            false => Module::start(self),
        }
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn enable(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn handle(&mut self, event: &MondrianMessage, app_configs: &AppConfigs) {
        match event {
            MondrianMessage::Pause(pause) => Module::pause(self, pause.unwrap_or(self.running.load(Ordering::SeqCst))),
            MondrianMessage::Configure => {
                self.configure(app_configs.into());
            }
            MondrianMessage::RefreshConfig => {
                self.configure(app_configs.into());
                Module::restart(self);
            }
            MondrianMessage::Quit => Module::stop(self),
            _ => {}
        }
    }

    fn name(&self) -> String {
        "events-monitor".to_string()
    }
}

impl ConfigurableModule for EventsMonitorModule {
    type Config = EventMonitorModuleConfigs;
    fn configure(&mut self, config: Self::Config) {
        self.configs = config;
    }
}