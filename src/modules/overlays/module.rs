use super::configs::OverlaysModuleConfigs;
use super::lib::overlay_manager::OverlaysManager;
use super::lib::overlays_event_handler::OverlayEventHandler;
use super::lib::utils::overlay::overlay_win_proc;
use crate::app::configs::AppConfigs;
use crate::app::mondrian_message::MondrianMessage;
use crate::app::mondrian_message::WindowTileState;
use crate::modules::module_impl::ModuleImpl;
use crate::modules::ConfigurableModule;
use crate::modules::Module;
use crate::win32::api::misc::get_current_thread_id;
use crate::win32::api::misc::post_empty_thread_message;
use crate::win32::api::window::register_class;
use crate::win32::win_events_manager::WindowsEventManager;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use windows::Win32::UI::WindowsAndMessaging::WM_QUIT;

const OVERLAY_CLASS_NAME: &str = "mondrian:overlay";

pub struct OverlaysModule {
    bus: Sender<MondrianMessage>,
    configs: OverlaysModuleConfigs,
    enabled: bool,
    overlays: Option<Arc<Mutex<OverlaysManager>>>,
    main_thread: Option<thread::JoinHandle<()>>,
    main_thread_id: Arc<AtomicU32>,
}

impl OverlaysModule {
    pub fn new(bus: Sender<MondrianMessage>) -> OverlaysModule {
        register_class(OVERLAY_CLASS_NAME, Some(overlay_win_proc)); // Overlays class

        OverlaysModule {
            configs: OverlaysModuleConfigs::default(),
            enabled: true,
            overlays: None,
            main_thread: None,
            bus,
            main_thread_id: Arc::new(AtomicU32::new(0)),
        }
    }

    fn is_running(&self) -> bool {
        self.main_thread.is_some()
    }
}

impl ModuleImpl for OverlaysModule {
    fn start(&mut self) {
        if self.is_running() || !self.configs.is_enabled() {
            return;
        }

        let (active, inactive) = (self.configs.get_active(), self.configs.get_inactive());
        let om = OverlaysManager::new(active, inactive, OVERLAY_CLASS_NAME);
        self.overlays = Some(Arc::new(Mutex::new(om)));

        let overlay_manager = self.overlays.clone().unwrap();
        let main_thread_id = self.main_thread_id.clone();
        let update_while_resizing = self.configs.update_while_resizing;
        let main_thread = thread::spawn(move || {
            main_thread_id.store(get_current_thread_id(), Ordering::SeqCst);
            let mut wem = WindowsEventManager::new();
            wem.hook(OverlayEventHandler::new(overlay_manager, update_while_resizing));
            wem.start_event_loop();
        });

        self.main_thread = Some(main_thread);
        self.bus.send(MondrianMessage::ListManagedWindows).unwrap();
    }

    fn stop(&mut self) {
        if let Some(main_thread) = self.main_thread.take() {
            post_empty_thread_message(self.main_thread_id.load(Ordering::SeqCst), WM_QUIT);
            main_thread.join().unwrap();
            let overlays = self.overlays.as_mut().expect("Overlays not initialized");
            overlays.lock().unwrap().destroy();
            self.main_thread_id.store(0, Ordering::SeqCst);
        }
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

    fn enable(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn handle(&mut self, event: &MondrianMessage, app_configs: &AppConfigs) {
        match event {
            MondrianMessage::Pause(pause) => Module::pause(self, pause.unwrap_or(self.is_running())),
            MondrianMessage::Configure => {
                Module::enable(self, app_configs.modules.overlays.enabled);
                self.configure(app_configs.into());
            }
            MondrianMessage::RefreshConfig => {
                Module::enable(self, app_configs.modules.overlays.enabled);
                self.configure(app_configs.into());
                Module::restart(self);
            }
            MondrianMessage::UpdatedWindows(windows, _) => {
                if self.is_running() {
                    let overlays = self.overlays.as_mut().expect("Overlays not initialized");
                    let win_set = windows
                        .iter()
                        .filter(|w| !matches!(*w.1, WindowTileState::Maximized))
                        .map(|w| *(w.0))
                        .collect();
                    overlays.lock().unwrap().rebuild(&win_set);
                }
            }
            MondrianMessage::CoreUpdateStart => {
                if self.is_running() {
                    let overlays = self.overlays.as_mut().expect("Overlays not initialized");
                    overlays.lock().unwrap().suspend();
                }
            }
            MondrianMessage::CoreUpdateError | MondrianMessage::CoreUpdateComplete => {
                if self.is_running() {
                    let overlays = self.overlays.as_mut().expect("Overlays not initialized");
                    overlays.lock().unwrap().resume();
                }
            }
            MondrianMessage::Quit => Module::stop(self),
            _ => {}
        }
    }

    fn name(&self) -> String {
        "overlays".to_string()
    }
}

impl ConfigurableModule for OverlaysModule {
    type Config = OverlaysModuleConfigs;
    fn configure(&mut self, config: Self::Config) {
        self.configs = config;
    }
}
