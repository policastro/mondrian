use super::configs::EventMonitorModuleConfigs;
use super::lib::focus_event_handler::FocusEventHandler;
use super::lib::maximize_event_handler::MaximizeEventHandler;
use super::lib::minimize_event_handler::MinimizeEventHandler;
use super::lib::open_event_handler::OpenCloseEventHandler;
use super::lib::position_event_handler::PositionEventHandler;
use super::lib::system_events_detector;
use crate::app::configs::AppConfig;
use crate::app::mondrian_message::MondrianMessage;
use crate::app::mondrian_message::SystemEvent;
use crate::modules::module_impl::ModuleImpl;
use crate::modules::ConfigurableModule;
use crate::modules::Module;
use crate::win32::api::misc::{get_current_thread_id, post_empty_thread_message};
use crate::win32::api::window::destroy_window;
use crate::win32::win_event_loop::start_win_event_loop;
use crate::win32::win_events_manager::WindowsEventManager;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;
use windows::Win32::UI::WindowsAndMessaging::WM_QUIT;
use winvd::listen_desktop_events;
use winvd::DesktopEvent;
use winvd::DesktopEventThread;

pub struct EventsMonitor {
    win_events_thread: Option<thread::JoinHandle<()>>,
    system_events_thread: Option<thread::JoinHandle<()>>,
    vd_listener_thread: Option<DesktopEventThread>,
    vd_events_thread: Option<thread::JoinHandle<()>>,
    win_events_thread_id: Arc<AtomicU32>,
    system_events_thread_id: Arc<AtomicU32>,
    configs: EventMonitorModuleConfigs,
    running: Arc<AtomicBool>,
    enabled: bool,
    bus_tx: Sender<MondrianMessage>,
}

impl EventsMonitor {
    pub fn new(bus_tx: Sender<MondrianMessage>) -> Self {
        EventsMonitor {
            win_events_thread: None,
            system_events_thread: None,
            vd_listener_thread: None,
            vd_events_thread: None,
            win_events_thread_id: Arc::new(AtomicU32::new(0)),
            system_events_thread_id: Arc::new(AtomicU32::new(0)),
            configs: EventMonitorModuleConfigs::default(),
            running: Arc::new(AtomicBool::new(false)),
            enabled: true,
            bus_tx,
        }
    }

    fn start_vd_events_loop(&mut self) {
        if self.vd_listener_thread.is_some() {
            return;
        }

        log::trace!("Starting Virtual Desktops events loop");

        let (winvd_tx, winvd_rx) = std::sync::mpsc::channel::<DesktopEvent>();
        let bus_tx = self.bus_tx.clone();
        self.vd_listener_thread = listen_desktop_events(winvd_tx).ok();
        self.vd_events_thread = Some(thread::spawn(move || {
            for event in winvd_rx {
                match event {
                    DesktopEvent::DesktopChanged { new, old } => {
                        let _ = bus_tx
                            .send(SystemEvent::VirtualDesktopChanged { old, new }.into())
                            .inspect_err(|_| log::warn!("Error sending VirtualDesktopChanged message"));
                    }
                    DesktopEvent::DesktopCreated(desktop) => {
                        let _ = bus_tx
                            .send(SystemEvent::VirtualDesktopCreated { desktop }.into())
                            .inspect_err(|_| log::warn!("Error sending VirtualDesktopChanged message"));
                    }
                    DesktopEvent::DesktopDestroyed { destroyed, fallback } => {
                        let _ = bus_tx
                            .send(SystemEvent::VirtualDesktopRemoved { destroyed, fallback }.into())
                            .inspect_err(|_| log::warn!("Error sending VirtualDesktopRemoved message"));
                    }
                    other => {
                        log::trace!("Virtual Desktops event: {other:?}");
                    }
                }
            }
        }));
    }

    fn start_system_events_loop(&mut self) {
        if self.system_events_thread.is_some() {
            return;
        }

        log::trace!("Starting System Events loop");

        let thread_id = self.system_events_thread_id.clone();
        let bus_tx = self.bus_tx.clone();
        self.system_events_thread = Some(thread::spawn(move || {
            let hwnd = system_events_detector::create(bus_tx.clone());
            if hwnd.is_none() {
                log::warn!("Failure while trying to create the system events detector window");
                return;
            }

            thread_id.store(get_current_thread_id(), Ordering::SeqCst);

            if let Some(hwnd) = hwnd {
                start_win_event_loop();
                destroy_window(hwnd);
            }
        }));
    }

    fn start_win_events_loop(&mut self) {
        if self.win_events_thread.is_some() {
            return;
        }

        log::trace!("Starting Win Events loop");

        let (detect_maximized, default_insert, default_free_move) = (
            self.configs.detect_maximized_windows,
            self.configs.default_insert_in_monitor,
            self.configs.default_free_move_in_monitor,
        );

        let thread_id = self.win_events_thread_id.clone();
        let bus_tx = self.bus_tx.clone();
        let filter = self.configs.filter.clone().unwrap();
        self.win_events_thread = Some(thread::spawn(move || {
            thread_id.store(get_current_thread_id(), Ordering::SeqCst);

            let mut wem = WindowsEventManager::new();
            wem.hook(OpenCloseEventHandler::new(bus_tx.clone(), filter.clone()));
            wem.hook(MinimizeEventHandler::new(bus_tx.clone(), filter.clone()));
            wem.hook(FocusEventHandler::new(bus_tx.clone(), filter.clone()));
            wem.hook(PositionEventHandler::new(
                bus_tx.clone(),
                filter.clone(),
                default_insert,
                default_free_move,
            ));

            if detect_maximized {
                wem.hook(MaximizeEventHandler::new(bus_tx.clone(), filter.clone()));
            }

            wem.start_event_loop();
        }));
    }

    fn stop_win_events_loop(&mut self) {
        if let Some(thread) = self.win_events_thread.take() {
            post_empty_thread_message(self.win_events_thread_id.load(Ordering::SeqCst), WM_QUIT);
            thread.join().unwrap();
            log::trace!("Win Events loop stopped");
        };
    }

    fn stop_system_events_loop(&mut self) {
        if let Some(thread) = self.system_events_thread.take() {
            post_empty_thread_message(self.system_events_thread_id.load(Ordering::SeqCst), WM_QUIT);
            thread.join().unwrap();
            log::trace!("System Events loop stopped");
        };
    }

    fn stop_vd_events_loop(&mut self) {
        self.vd_listener_thread.take();
        if let Some(thread) = self.vd_events_thread.take() {
            thread.join().unwrap();
            log::trace!("Virtual Desktops Events loop stopped");
        };
    }
}

impl ModuleImpl for EventsMonitor {
    fn start(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }

        self.start_vd_events_loop();
        self.start_system_events_loop();
        self.start_win_events_loop();

        self.running.store(true, Ordering::SeqCst);
        log::trace!("EventsMonitorModule started");
    }

    fn stop(&mut self) {
        if !self.running.load(Ordering::SeqCst) {
            return;
        }

        self.stop_vd_events_loop();
        self.stop_system_events_loop();
        self.stop_win_events_loop();

        self.running.store(false, Ordering::SeqCst);
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

    fn handle(&mut self, event: &MondrianMessage, app_configs: &AppConfig) {
        match event {
            MondrianMessage::Pause(pause) => Module::pause(self, pause.unwrap_or(self.running.load(Ordering::SeqCst))),
            MondrianMessage::Retile => Module::restart(self),
            MondrianMessage::Configure => {
                self.configure(app_configs.into());
            }
            MondrianMessage::RefreshConfig => {
                self.configure(app_configs.into());
                Module::restart(self);
            }
            MondrianMessage::SystemEvent(evt) => match evt {
                evt if evt.session_is_active() => {
                    if self.enabled {
                        self.start_vd_events_loop();
                        self.start_win_events_loop();
                    }
                }
                evt if evt.session_is_inactive() => {
                    self.stop_vd_events_loop();
                    self.stop_win_events_loop();
                }
                _ => {}
            },
            MondrianMessage::Quit => Module::stop(self),
            _ => {}
        }
    }

    fn name(&self) -> String {
        "events-monitor".to_string()
    }
}

impl ConfigurableModule for EventsMonitor {
    type Config = EventMonitorModuleConfigs;
    fn configure(&mut self, config: Self::Config) {
        self.configs = config;
    }
}
