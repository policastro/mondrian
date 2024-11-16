use super::configs::CoreModuleConfigs;
use super::lib::monitor_layout::MonitorLayout;
use super::lib::tm::TilesManager;
use super::lib::tm_command::TMCommand;
use super::lib::tm_configs::TilesManagerConfig;
use crate::app::config::app_configs::AppConfigs;
use crate::app::mondrian_message::{MondrianMessage, WindowEvent};
use crate::modules::module_impl::ModuleImpl;
use crate::modules::{ConfigurableModule, Module};
use crate::win32::api::monitor::enum_display_monitors;
use crate::win32::window::window_ref::WindowRef;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;

pub struct TilesManagerModule {
    tm_command_tx: Option<Sender<TMCommand>>,
    tiles_manager_thread: Option<thread::JoinHandle<()>>,
    configs: CoreModuleConfigs,
    running: Arc<AtomicBool>,
    enabled: bool,
    bus_tx: Sender<MondrianMessage>,
}

impl TilesManagerModule {
    pub fn new(bus_tx: Sender<MondrianMessage>) -> Self {
        TilesManagerModule {
            configs: CoreModuleConfigs::default(),
            tm_command_tx: None,
            tiles_manager_thread: None,
            running: Arc::new(AtomicBool::new(false)),
            enabled: true,
            bus_tx,
        }
    }

    pub fn send_to_tm(&self, event: TMCommand) {
        if !self.enabled || !self.running.load(Ordering::SeqCst) {
            return;
        }
        if let Some(tx) = self.tm_command_tx.as_ref() {
            tx.send(event).expect("Failed to send event to tiles manager");
        }
    }

    fn start_tiles_manager(&mut self, event_receiver: Receiver<TMCommand>) {
        let monitors = enum_display_monitors()
            .into_iter()
            .map(|monitor| MonitorLayout::new(monitor.clone(), self.configs.get_layout(Some(&monitor.name))))
            .collect();

        let tm_configs = TilesManagerConfig::from(&self.configs);

        let app_tx = self.bus_tx.clone();
        let on_update_start = move || {
            app_tx.send(MondrianMessage::CoreUpdateStart).unwrap();
        };

        let app_tx = self.bus_tx.clone();
        let tm_tx = self.tm_command_tx.clone();
        let on_update_error = move || {
            app_tx.send(MondrianMessage::CoreUpdateError).unwrap();
            if let Some(tx) = tm_tx.as_ref() {
                tx.send(TMCommand::Update(false)).unwrap();
            }
        };

        let app_tx = self.bus_tx.clone();
        let on_update_complete = move || {
            app_tx.send(MondrianMessage::CoreUpdateComplete).unwrap();
        };

        let mut tm = TilesManager::new(
            monitors,
            Some(tm_configs),
            on_update_start,
            on_update_error,
            on_update_complete,
        );
        let tx = self.bus_tx.clone();

        self.tiles_manager_thread = Some(thread::spawn(move || loop {
            match event_receiver.recv() {
                Ok(app_event) => {
                    if !handle_tm(&mut tm, &tx, app_event) {
                        log::trace!("TilesManager exit!");
                        break;
                    }
                }
                Err(error) => {
                    log::error!("Error: {:?}", error);
                    break;
                }
            }
        }));
    }
}

impl ModuleImpl for TilesManagerModule {
    fn start(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }

        log::trace!("Start App::run()");

        self.running.store(true, Ordering::SeqCst);

        let (event_sender, event_receiver) = channel();
        self.tm_command_tx = Some(event_sender.clone());
        self.start_tiles_manager(event_receiver);

        log::trace!("App::run() done");
    }

    fn stop(&mut self) {
        if !self.running.load(Ordering::SeqCst) {
            return;
        }

        self.running.store(false, Ordering::SeqCst);

        log::trace!("Start App::stop()");

        self.tm_command_tx.as_ref().unwrap().send(TMCommand::Quit).unwrap();
        if let Some(thread) = self.tiles_manager_thread.take() {
            thread.join().unwrap();
        };

        log::trace!("App::stop() done");
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
            MondrianMessage::Retile => Module::restart(self),
            MondrianMessage::RefreshConfig => {
                self.configure(app_configs.into());
                Module::restart(self);
            }
            MondrianMessage::MonitorsLayoutChanged => {
                Module::restart(self);
            }
            MondrianMessage::Quit => Module::stop(self),
            msg => {
                if let Ok(command) = msg.try_into() {
                    self.send_to_tm(command);
                }
            }
        }
    }

    fn name(&self) -> String {
        "core".to_string()
    }
}

impl ConfigurableModule for TilesManagerModule {
    type Config = CoreModuleConfigs;
    fn configure(&mut self, config: Self::Config) {
        self.configs = config;
    }
}

fn handle_tm(tm: &mut TilesManager, tx: &Sender<MondrianMessage>, event: TMCommand) -> bool {
    let res = match event {
        TMCommand::WindowEvent(window_event) => match window_event {
            WindowEvent::Opened(hwnd) | WindowEvent::Restored(hwnd) | WindowEvent::Unmaximized(hwnd) => {
                tm.add(WindowRef::new(hwnd), true)
            }
            WindowEvent::Closed(hwnd) | WindowEvent::Minimized(hwnd) | WindowEvent::Maximized(hwnd) => {
                let minimized = matches!(window_event, WindowEvent::Minimized(_));
                tm.remove(hwnd, minimized, true)
            }
            WindowEvent::StartMoveSize(_) => {
                tm.cancel_animation();
                Ok(())
            }
            WindowEvent::Moved(hwnd, coords, intra, inter) => tm.on_move(hwnd, coords, intra, inter),
            WindowEvent::Resized(hwnd, p_area, c_area) => tm.on_resize(hwnd, c_area.get_shift(&p_area), true),
        },
        TMCommand::Focus(direction) => tm.focus_at(direction),
        TMCommand::Minimize => tm.minimize_focused(),
        TMCommand::Move(direction) => tm.move_focused(direction),
        TMCommand::Resize(direction, size) => tm.resize_focused(direction, size),
        TMCommand::Invert => tm.invert_orientation(),
        TMCommand::Release(b) => tm.release(b, None),
        TMCommand::Focalize => tm.focalize_focused(),
        TMCommand::ListManagedWindows => {
            let windows = tm.get_managed_windows();
            tx.send(MondrianMessage::UpdatedWindows(windows, event)).unwrap();
            Ok(())
        }
        TMCommand::Update(animate) => {
            tm.update(animate);
            Ok(())
        }
        TMCommand::Quit => return false,
    };

    match res {
        Err(error) if error.require_refresh() => {
            tm.update(true);
            log::error!("{:?}", error)
        }
        Err(error) if error.is_warn() => log::warn!("{:?}", error),
        Err(error) => log::error!("{:?}", error),
        Ok(_) => {}
    }

    if event.require_update() {
        let windows = tm.get_managed_windows();
        tx.send(MondrianMessage::UpdatedWindows(windows, event)).unwrap();
    }

    true
}
