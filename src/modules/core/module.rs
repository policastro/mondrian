use windows::Win32::UI::WindowsAndMessaging::WM_QUIT;

use crate::app::config::app_configs::AppConfigs;
use crate::app::config::win_matcher;
use crate::app::mondrian_command::MondrianMessage;
use crate::app::tiles_manager::config::TilesManagerConfig;
use crate::app::tiles_manager::manager::TilesManager;
use crate::app::tiles_manager::monitor_layout::MonitorLayout;
use crate::app::tiles_manager::tm_command::TMCommand;
use crate::app::win_events_handlers::maximize_event_handler::MaximizeEventHandler;
use crate::modules::module::module_impl::ModuleImpl;
use crate::modules::module::{ConfigurableModule, Module};
use crate::win32::api::misc::{get_current_thread_id, post_empty_thread_message};

use crate::app::win_events_handlers::position_event_handler::PositionEventHandler;
use crate::app::win_events_handlers::{
    minimize_event_handler::MinimizeEventHandler, open_event_handler::OpenCloseEventHandler,
};
use crate::win32::api::monitor::enum_display_monitors;
use crate::win32::win_events_manager::WinEventManager;
use crate::win32::window::window_ref::WindowRef;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::mpsc::{RecvError, Sender};
use std::sync::Arc;
use std::thread::{self};

use super::config::CoreModuleConfigs;

pub struct CoreModule {
    tm_command_tx: Option<Sender<TMCommand>>,
    win_events_thread_id: Arc<AtomicU32>,
    win_events_thread: Option<thread::JoinHandle<()>>,
    tiles_manager_thread: Option<thread::JoinHandle<()>>,
    configs: CoreModuleConfigs,
    running: Arc<AtomicBool>,
    enabled: bool,
    bus_tx: Sender<MondrianMessage>,
}

impl CoreModule {
    pub fn new(bus_tx: Sender<MondrianMessage>) -> Self {
        CoreModule {
            configs: CoreModuleConfigs::default(),
            tm_command_tx: None,
            win_events_thread: None,
            win_events_thread_id: Arc::new(AtomicU32::new(0)),
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

    fn run_win_events_loop(&mut self, event_sender: Sender<TMCommand>) {
        let detect_maximized_windows = self.configs.detect_maximized_windows;
        let win_events_thread_id = self.win_events_thread_id.clone();
        self.win_events_thread = Some(thread::spawn(move || {
            win_events_thread_id.store(get_current_thread_id(), Ordering::SeqCst);
            let mut wem = WinEventManager::new();
            wem.hook(PositionEventHandler::new(event_sender.clone()));
            wem.hook(OpenCloseEventHandler::new(event_sender.clone()));
            wem.hook(MinimizeEventHandler::new(event_sender.clone()));
            if detect_maximized_windows {
                wem.hook(MaximizeEventHandler::new(event_sender.clone()));
            }
            wem.start_event_loop();
        }));
    }

    fn run_tiles_manager(&mut self, event_receiver: Receiver<TMCommand>) {
        let filter = self.configs.filter.clone().unwrap();

        let monitors = enum_display_monitors()
            .into_iter()
            .map(|monitor| MonitorLayout::new(monitor.clone(), self.configs.get_layout(Some(&monitor.name))))
            .collect();

        let tm_configs = TilesManagerConfig::from(&self.configs);
        let mut tm = TilesManager::new(monitors, Some(tm_configs));
        let tx = self.bus_tx.clone();
        self.tiles_manager_thread = Some(thread::spawn(move || loop {
            match filter_events(event_receiver.recv(), &filter) {
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

impl ModuleImpl for CoreModule {
    fn start(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }

        log::trace!("Start App::run()");

        self.running.store(true, Ordering::SeqCst);
        let (event_sender, event_receiver) = channel();
        self.run_win_events_loop(event_sender.clone());
        self.tm_command_tx = Some(event_sender.clone());
        self.run_tiles_manager(event_receiver);

        log::trace!("App::run() done");
    }

    fn stop(&mut self) {
        if !self.running.load(Ordering::SeqCst) {
            return;
        }

        self.running.store(false, Ordering::SeqCst);

        log::trace!("Start App::stop()");
        self.tm_command_tx.as_ref().unwrap().send(TMCommand::Quit).unwrap();

        if let Some(thread) = self.win_events_thread.take() {
            post_empty_thread_message(self.win_events_thread_id.load(Ordering::SeqCst), WM_QUIT);
            thread.join().unwrap();
        };

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
            msg if TMCommand::from(msg).is_op() => self.send_to_tm(msg.into()),
            MondrianMessage::Quit => Module::stop(self),
            _ => {}
        }
    }

    fn name(&self) -> String {
        "core".to_string()
    }
}

impl ConfigurableModule for CoreModule {
    type Config = CoreModuleConfigs;
    fn configure(&mut self, config: Self::Config) {
        self.configs = config;
    }
}

fn handle_tm(tm: &mut TilesManager, tx: &Sender<MondrianMessage>, event: TMCommand) -> bool {
    let res = match event {
        TMCommand::WindowOpened(hwnd) | TMCommand::WindowRestored(hwnd) | TMCommand::WindowUnmaximized(hwnd) => {
            tm.add(WindowRef::new(hwnd), true)
        }
        TMCommand::WindowClosed(hwnd) | TMCommand::WindowMinimized(hwnd) | TMCommand::WindowMaximized(hwnd) => {
            let minimized = matches!(event, TMCommand::WindowMinimized(_));
            tm.remove(hwnd, minimized, true)
        }
        TMCommand::WindowStartMoveSize(_) => {
            tm.cancel_animation();
            Ok(())
        }
        TMCommand::WindowMoved(hwnd, coords, invert, switch) => tm.on_move(hwnd, coords, invert, switch),
        TMCommand::WindowResized(hwnd, p_area, c_area) => tm.on_resize(hwnd, c_area.get_shift(&p_area), true),
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
        TMCommand::Noop => return true,
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

fn filter_events(
    result: Result<TMCommand, RecvError>,
    win_match: &win_matcher::WinMatcher,
) -> Result<TMCommand, RecvError> {
    let event = match result {
        Ok(event) => event,
        _ => return Ok(TMCommand::Noop),
    };

    let cmd = match event {
        TMCommand::WindowClosed(hwnd) => {
            let snap = WindowRef::new(hwnd).snapshot();
            log::info!("[{:?}]: {}", event, snap.map_or("/".to_string(), |w| format!("{}", w)));
            event
        }
        command if command.can_be_filtered() => {
            let hwnd = command.get_hwnd().expect("Can't get the window handle");
            let info = match WindowRef::new(hwnd).snapshot() {
                Some(win_info) => win_info,
                None => return Ok(TMCommand::Noop),
            };

            if win_match.matches(&info) {
                log::trace!("[excluded][{:?}]: {}", event, info);
                TMCommand::Noop
            } else {
                log::info!("[{:?}]: {}", event, info);
                event
            }
        }
        _ => event,
    };

    Ok(cmd)
}
