use crate::app::config::app_configs::AppConfigs;
use crate::app::config::filters::window_match_filter::WinMatchAnyFilters;
use crate::app::mondrian_command::MondrianCommand;
use crate::app::tiles_manager::config::TilesManagerConfig;
use crate::app::tiles_manager::manager::TilesManager;
use crate::app::tiles_manager::monitor_layout::MonitorLayout;
use crate::app::tiles_manager::tm_command::TMCommand;
use crate::modules::module::module_impl::ModuleImpl;
use crate::modules::module::{ConfigurableModule, Module};
use crate::win32::win_event_loop::next_win_event_loop_no_block;

use crate::app::config::filters::window_filter::WindowFilter;
use crate::app::win_events_handlers::position_event_handler::PositionEventHandler;
use crate::app::win_events_handlers::{
    minimize_event_handler::MinimizeEventHandler, open_event_handler::OpenCloseEventHandler,
};
use crate::win32::api::monitor::enum_display_monitors;
use crate::win32::win_events_manager::WinEventManager;
use crate::win32::window::window_ref::WindowRef;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::mpsc::{RecvError, Sender};
use std::sync::Arc;
use std::thread::{self, sleep};
use std::time::Duration;

use super::config::CoreConfigs;

pub struct CoreModule {
    tm_command_tx: Option<Sender<TMCommand>>,
    win_events_thread: Option<thread::JoinHandle<()>>,
    tiles_manager_thread: Option<thread::JoinHandle<()>>,
    configs: CoreConfigs,
    running: Arc<AtomicBool>,
    enabled: bool,
}

impl CoreModule {
    pub fn new() -> Self {
        CoreModule {
            configs: CoreConfigs::default(),
            tm_command_tx: None,
            win_events_thread: None,
            tiles_manager_thread: None,
            running: Arc::new(AtomicBool::new(false)),
            enabled: true,
        }
    }

    pub fn send_to_tiles_manager(&self, event: TMCommand) {
        if !self.enabled || !self.running.load(Ordering::SeqCst) {
            return;
        }
        if let Some(tx) = self.tm_command_tx.as_ref() {
            tx.send(event).expect("Failed to send event to tiles manager");
        }
    }

    fn run_win_events_loop(&mut self, event_sender: Sender<TMCommand>) {
        let wem_builder = WinEventManager::builder()
            .handler(PositionEventHandler::new(event_sender.clone()))
            .handler(OpenCloseEventHandler::new(event_sender.clone()))
            .handler(MinimizeEventHandler::new(event_sender.clone()));

        let refresh_time = self.configs.refresh_time;
        let running = self.running.clone();
        self.win_events_thread = Some(thread::spawn(move || {
            let mut wem = wem_builder.build();
            while running.load(Ordering::SeqCst) {
                next_win_event_loop_no_block(None);
                wem.check_for_events();
                sleep(Duration::from_millis(refresh_time));
            }
            log::trace!("WinEventsLoop exit!");
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
        self.tiles_manager_thread = Some(thread::spawn(move || loop {
            match filter_events(event_receiver.recv(), &filter) {
                Ok(app_event) => {
                    if !handle_tiles_manager(&mut tm, app_event) {
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

    fn handle(&mut self, event: &MondrianCommand, app_configs: &AppConfigs) {
        match event {
            MondrianCommand::Pause(pause) => Module::pause(self, *pause),
            MondrianCommand::Configure => {
                self.configure(app_configs.into());
            }
            MondrianCommand::Retile => Module::restart(self),
            MondrianCommand::RefreshConfig => {
                self.configure(app_configs.into());
                Module::restart(self);
            }
            MondrianCommand::Focus(dir) => self.send_to_tiles_manager(TMCommand::Focus(*dir)),
            MondrianCommand::Quit => Module::stop(self),
            _ => {}
        }
    }
}

impl ConfigurableModule for CoreModule {
    type Config = CoreConfigs;
    fn configure(&mut self, config: Self::Config) {
        self.configs = config;
    }
}

fn handle_tiles_manager(tm: &mut TilesManager, event: TMCommand) -> bool {
    match event {
        TMCommand::WindowOpened(hwnd) | TMCommand::WindowRestored(hwnd) => drop(tm.add(WindowRef::new(hwnd), true)),
        TMCommand::WindowClosed(hwnd) | TMCommand::WindowMinimized(hwnd) => tm.remove(hwnd),
        TMCommand::WindowMoved(hwnd, coords, invert, switch) => tm.move_window(hwnd, coords, invert, switch),
        TMCommand::WindowResized(hwnd) => tm.refresh_window_size(hwnd),
        TMCommand::Focus(direction) => tm.focus_at(direction),
        TMCommand::Noop => return true,
        TMCommand::Quit => return false,
    };
    true
}

fn filter_events(
    result: Result<TMCommand, RecvError>,
    win_filter: &WinMatchAnyFilters,
) -> Result<TMCommand, RecvError> {
    let event = match result {
        Ok(event) => event,
        _ => return Ok(TMCommand::Noop),
    };

    Ok(match event {
        TMCommand::WindowClosed(hwnd) => {
            let snap = WindowRef::new(hwnd).snapshot();
            log::info!("[{:?}]: {}", event, snap.map_or("/".to_string(), |w| format!("{}", w)));
            event
        }
        TMCommand::WindowOpened(hwnd)
        | TMCommand::WindowMinimized(hwnd)
        | TMCommand::WindowRestored(hwnd)
        | TMCommand::WindowResized(hwnd)
        | TMCommand::WindowMoved(hwnd, _, _, _) => {
            let win_info = match WindowRef::new(hwnd).snapshot() {
                Some(win_info) => win_info,
                None => return Ok(TMCommand::Noop),
            };

            match !win_filter.filter(&win_info) {
                true => {
                    log::info!("[{:?}]: {}", event, win_info);
                    event
                }
                false => {
                    log::trace!("[excluded][{:?}]: {}", event, win_info);
                    TMCommand::Noop
                }
            }
        }
        _ => event,
    })
}
