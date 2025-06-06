use super::configs::OverlaysModuleConfigs;
use super::lib::overlay_manager::MonoOverlaysManager;
use super::lib::overlay_manager::MultiOverlaysManager;
use super::lib::overlay_manager::OverlaysManagerEnum;
use super::lib::overlay_manager::OverlaysManagerTrait;
use super::lib::overlays_event_handler::OverlayEventHandler;
use super::lib::utils::overlay::overlay_win_proc;
use crate::app::configs::AppConfig;
use crate::app::mondrian_message::MondrianMessage;
use crate::app::mondrian_message::WindowEvent;
use crate::app::mondrian_message::WindowTileState;
use crate::modules::module_impl::ModuleImpl;
use crate::modules::ConfigurableModule;
use crate::modules::Module;
use crate::win32::api::misc::get_current_thread_id;
use crate::win32::api::misc::post_empty_thread_message;
use crate::win32::api::window::register_class;
use crate::win32::win_events_manager::WindowsEventManager;
use crossbeam_channel::Sender;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use windows::Win32::UI::WindowsAndMessaging::WM_QUIT;

const OVERLAY_CLASS_NAME: &str = "mondrian:overlay";

pub struct Overlays {
    bus: Sender<MondrianMessage>,
    configs: OverlaysModuleConfigs,
    enabled: bool,
    overlays: Option<Arc<Mutex<OverlaysManagerEnum>>>,
    main_thread: Option<thread::JoinHandle<()>>,
    main_thread_id: Arc<AtomicU32>,
}

impl Overlays {
    pub fn new(bus: Sender<MondrianMessage>) -> Overlays {
        register_class(OVERLAY_CLASS_NAME, Some(overlay_win_proc));

        Overlays {
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

impl ModuleImpl for Overlays {
    fn start(&mut self) {
        if self.is_running() || !self.configs.is_enabled() {
            return;
        }

        let (active, inactive) = (
            self.configs.get_active().unwrap_or(self.configs.get_hidden()),
            self.configs.get_inactive(),
        );
        let om = match inactive {
            Some(inactive) => {
                OverlaysManagerEnum::from(MultiOverlaysManager::new(active, inactive, OVERLAY_CLASS_NAME))
            }
            None => OverlaysManagerEnum::from(MonoOverlaysManager::new(active, OVERLAY_CLASS_NAME)),
        };
        self.overlays = Some(Arc::new(Mutex::new(om)));

        let overlay_manager = self.overlays.clone().unwrap();
        let main_thread_id = self.main_thread_id.clone();
        let main_thread = thread::spawn(move || {
            main_thread_id.store(get_current_thread_id(), Ordering::SeqCst);
            let mut wem = WindowsEventManager::new();
            wem.hook(OverlayEventHandler::new(overlay_manager));
            wem.start_event_loop();
        });

        self.main_thread = Some(main_thread);
        self.bus.send(MondrianMessage::ListManagedWindows).unwrap();
    }

    fn stop(&mut self) {
        if let Some(main_thread) = self.main_thread.take() {
            post_empty_thread_message(self.main_thread_id.load(Ordering::SeqCst), WM_QUIT);
            main_thread.join().unwrap();
            self.overlays = None;
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

    fn handle(&mut self, event: &MondrianMessage, app_configs: &AppConfig) {
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
                if !self.is_running() {
                    return;
                }

                let wins = windows
                    .iter()
                    .filter(|w| !matches!(*w.1, WindowTileState::Maximized))
                    .map(|w| (*w.0, self.configs.get_by_tile_state(w.1)))
                    .collect();

                let overlays = self.overlays.as_mut().expect("Overlays not initialized");
                overlays.lock().unwrap().rebuild(&wins);
            }
            MondrianMessage::WindowEvent(WindowEvent::StartMoveSize(win)) => {
                if !self.is_running() || self.configs.update_while_dragging {
                    return;
                }

                let overlays = self.overlays.as_mut().expect("Overlays not initialized");
                overlays.lock().unwrap().suspend(*win);
            }
            MondrianMessage::WindowEvent(WindowEvent::EndMoveSize(win, _)) => {
                if !self.is_running() || self.configs.update_while_dragging {
                    return;
                }

                let overlays = self.overlays.as_mut().expect("Overlays not initialized");
                overlays.lock().unwrap().resume(*win);
            }
            MondrianMessage::CoreUpdateStart(wins, animations_enabled) => {
                if !self.is_running() || !animations_enabled || self.configs.update_while_animating {
                    return;
                }

                let overlays = self.overlays.as_mut().expect("Overlays not initialized");
                wins.iter().for_each(|w| overlays.lock().unwrap().suspend(*w))
            }
            MondrianMessage::CoreUpdateError | MondrianMessage::CoreUpdateComplete => {
                if !self.is_running() || self.configs.update_while_animating {
                    return;
                }

                let overlays = self.overlays.as_mut().expect("Overlays not initialized");
                overlays.lock().unwrap().resume_all();
            }
            MondrianMessage::SystemEvent(evt) => match evt {
                evt if evt.session_is_active() => Module::start(self),
                evt if evt.session_is_inactive() => Module::stop(self),
                _ => {}
            },
            MondrianMessage::Quit => Module::stop(self),
            _ => {}
        }
    }

    fn name(&self) -> String {
        "overlays".to_string()
    }
}

impl ConfigurableModule for Overlays {
    type Config = OverlaysModuleConfigs;
    fn configure(&mut self, config: Self::Config) {
        self.configs = config;
    }
}
