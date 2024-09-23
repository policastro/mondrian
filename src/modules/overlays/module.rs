use crate::{
    app::{config::app_configs::AppConfigs, mondrian_command::MondrianMessage},
    modules::module::{module_impl::ModuleImpl, ConfigurableModule, Module},
    win32::{win_event_loop::next_win_event_loop_no_block, win_events_manager::WinEventManagerBuilder},
};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::Sender,
    Arc, Mutex,
};

use std::thread;

use super::{
    configs::OverlaysModuleConfigs,
    lib::{overlay_manager::OverlaysManager, overlays_event_handler::OverlayEventHandler},
};

pub struct OverlaysModule {
    bus: Sender<MondrianMessage>,
    running: Arc<AtomicBool>,
    configs: OverlaysModuleConfigs,
    enabled: bool,
    overlays: Option<Arc<Mutex<OverlaysManager>>>,
    main_thread: Option<thread::JoinHandle<()>>,
}

impl OverlaysModule {
    pub fn new(bus: Sender<MondrianMessage>) -> OverlaysModule {
        OverlaysModule {
            running: Arc::new(AtomicBool::new(false)),
            configs: OverlaysModuleConfigs::default(),
            enabled: true,
            overlays: None,
            main_thread: None,
            bus,
        }
    }
}

impl ModuleImpl for OverlaysModule {
    fn start(&mut self) {
        if self.running.load(Ordering::SeqCst) || !self.configs.is_enabled() {
            return;
        }

        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        let om = OverlaysManager::new(self.configs.get_active(), self.configs.get_inactive());
        self.overlays = Some(Arc::new(Mutex::new(om)));

        let overlay_manager = self.overlays.clone().unwrap();
        let follow_movements = self.configs.get_follow_movements();
        let main_thread = thread::spawn(move || {
            let handler = OverlayEventHandler::new(overlay_manager, follow_movements);
            let wem_builder = WinEventManagerBuilder::new().handler(handler);
            thread::spawn(move || {
                let mut wem = wem_builder.build();
                while running.load(Ordering::SeqCst) {
                    wem.check_for_events();
                    next_win_event_loop_no_block(None);
                    thread::sleep(std::time::Duration::from_millis(20));
                }
            });
        });

        self.main_thread = Some(main_thread);
        self.bus.send(MondrianMessage::ListManagedWindows).unwrap();
    }

    fn stop(&mut self) {
        if !self.running.load(Ordering::SeqCst) {
            return;
        }

        self.running.store(false, Ordering::SeqCst);
        if let Some(main_thread) = self.main_thread.take() {
            main_thread.join().unwrap();
        }
        let overlays = self.overlays.as_mut().expect("Overlays not initialized");
        overlays.lock().unwrap().destroy();
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
            MondrianMessage::Pause(pause) => Module::pause(self, pause.unwrap_or(self.running.load(Ordering::SeqCst))),
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
                if self.running.load(Ordering::SeqCst) {
                    let overlays = self.overlays.as_mut().expect("Overlays not initialized");
                    overlays.lock().unwrap().rebuild(windows);
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
