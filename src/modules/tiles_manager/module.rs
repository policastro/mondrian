use super::configs::CoreModuleConfigs;
use super::lib::tm::command::TMCommand;
use super::lib::tm::configs::TilesManagerConfig;
use super::lib::tm::error::TilesManagerError;
use super::lib::tm::manager::TilesManager;
use super::lib::tm::manager::TilesManagerBase;
use super::lib::tm::public::TilesManagerOperations;
use crate::app::configs::AppConfigs;
use crate::app::mondrian_message::MondrianMessage;
use crate::app::mondrian_message::SystemEvent;
use crate::app::mondrian_message::WindowEvent;
use crate::modules::module_impl::ModuleImpl;
use crate::modules::ConfigurableModule;
use crate::modules::Module;
use crate::win32::window::window_ref::WindowRef;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
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

        let mut tm = TilesManager::new(Some(tm_configs), on_update_start, on_update_error, on_update_complete);
        let _ = tm.init();
        let _ = tm.add_open_windows();
        let _ = tm.update_layout(true);

        let configs = self.configs.clone();
        let tx = self.bus_tx.clone();

        self.tiles_manager_thread = Some(thread::spawn(move || loop {
            match event_receiver.recv() {
                Ok(app_event) => {
                    if !handle_tm(&mut tm, &tx, &configs, app_event) {
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
                let config_changed = self.configs != app_configs.into();
                self.configure(app_configs.into());
                if config_changed {
                    Module::restart(self);
                }
            }
            MondrianMessage::SystemEvent(evt) if *evt == SystemEvent::MonitorsLayoutChanged => {
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

fn handle_tm(
    tm: &mut TilesManager,
    tx: &Sender<MondrianMessage>,
    configs: &CoreModuleConfigs,
    event: TMCommand,
) -> bool {
    let _ = tm.check_for_vd_changes();
    let prev_wins = tm.get_managed_windows();
    let res = match event {
        TMCommand::WindowEvent(window_event) => match window_event {
            WindowEvent::Maximized(hwnd) => tm.on_maximize(hwnd.into(), true),
            WindowEvent::Unmaximized(hwnd) => tm.on_maximize(hwnd.into(), false),
            WindowEvent::Opened(hwnd) | WindowEvent::Restored(hwnd) => tm.add(WindowRef::new(hwnd)),
            WindowEvent::Closed(hwnd) | WindowEvent::Minimized(hwnd) => {
                let minimized = matches!(window_event, WindowEvent::Minimized(_));
                tm.remove(hwnd.into(), minimized)
            }
            WindowEvent::StartMoveSize(_) => {
                tm.pause_updates(true);
                tm.cancel_animation();
                Ok(())
            }
            WindowEvent::NoMoveSize(_) => {
                tm.pause_updates(false);
                Ok(())
            }
            WindowEvent::Moved(hwnd, coords, intra, inter) => {
                tm.pause_updates(false);
                tm.on_move(hwnd.into(), coords, intra, inter)
            }
            WindowEvent::Resized(hwnd, p_area, c_area) => {
                tm.pause_updates(false);
                tm.on_resize(hwnd.into(), c_area.get_shift(&p_area))
            }
            WindowEvent::Focused(hwnd) => tm.on_focus(hwnd.into()),
        },
        TMCommand::SystemEvent(evt) => match evt {
            SystemEvent::VirtualDesktopCreated { desktop } => tm.on_vd_created(desktop),
            SystemEvent::VirtualDesktopRemoved { destroyed, fallback } => tm.on_vd_destroyed(destroyed, fallback),
            SystemEvent::VirtualDesktopChanged { old, new } => tm.on_vd_changed(old, new),
            _ => Ok(()),
        },
        TMCommand::Focus(direction) => tm.change_focus(direction, configs.move_cursor_on_focus),
        TMCommand::Minimize => tm.minimize_focused(),
        TMCommand::Insert(direction) => tm.move_focused(direction),
        TMCommand::Move(direction, insert_if_empty) => match tm.swap_focused(direction) {
            Err(TilesManagerError::NoWindow) if insert_if_empty => tm.move_focused(direction),
            res => res,
        },
        TMCommand::Resize(direction, size) => tm.resize_focused(direction, size),
        TMCommand::Invert => tm.invert_orientation(),
        TMCommand::Release(b) => tm.release_focused(b),
        TMCommand::Focalize => tm.focalize_focused(),
        TMCommand::Amplify => tm.amplify_focused(),
        TMCommand::ListManagedWindows => {
            let windows = tm.get_managed_windows();
            tx.send(MondrianMessage::UpdatedWindows(windows, event)).unwrap();
            Ok(())
        }
        TMCommand::Update(animate) => tm.update_layout(animate),
        TMCommand::Quit => return false,
    };

    match res {
        Err(error) if error.require_refresh() => {
            let _ = tm.update_layout(true);
            log::error!("{:?}", error)
        }
        Err(error) if error.is_warn() => log::warn!("{:?}", error),
        Err(error) => log::error!("{:?}", error),
        Ok(_) => {}
    }

    if event.can_change_layout() {
        let windows = tm.get_managed_windows();
        if windows != prev_wins {
            tx.send(MondrianMessage::UpdatedWindows(windows, event)).unwrap();
        }
    }

    true
}
