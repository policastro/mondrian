use crate::app;
use crate::app::config::app_configs::AppConfigs;

use crate::app::config::filters::window_filter::WindowFilter;
use crate::app::win_events_handlers::minimize_event_handler::MinimizeEventHandler;
use crate::app::win_events_handlers::open_event_handler::OpenCloseEventHandler;
use crate::app::win_events_handlers::position_event_handler::PositionEventHandler;
use crate::win32utils::api::monitor::enum_display_monitors;
use crate::win32utils::api::window::enum_user_manageable_windows;
use crate::win32utils::win_event_loop::run_win_event_loop;
use crate::win32utils::win_events_manager::WinEventManager;
use crate::win32utils::window::window_ref::WindowRef;
use log::info;
use std::sync::mpsc::{self, Sender, TryRecvError};
use std::sync::mpsc::{channel, Receiver};
use std::thread::{self};

use super::app_event::AppEvent;
use super::config::filters::window_match_filter::WinMatchAnyFilters;
use super::structs::area_tree::layout::golden_ration_layout::GoldenRatioLayoutStrategy;
use super::structs::area_tree::layout::layout_strategy::AreaTreeLayoutStrategy;
use super::structs::direction::Direction;
use super::structs::monitors_layout::MonitorsLayout;
use super::tiles_manager::TilesManager;

pub struct MondrianApp {
    win_events_loop_tx: Option<Sender<i32>>,
    tiles_manager_loop_tx: Option<Sender<i32>>,
    win_events_thread: Option<thread::JoinHandle<()>>,
    tiles_manager_thread: Option<thread::JoinHandle<()>>,
    app_configs: AppConfigs,
}

impl MondrianApp {
    pub fn new(app_configs: AppConfigs) -> Self {
        MondrianApp {
            win_events_loop_tx: None,
            tiles_manager_loop_tx: None,
            win_events_thread: None,
            tiles_manager_thread: None,
            app_configs,
        }
    }

    pub fn run(&mut self, enable_input: bool) {
        info!("App::run()");

        let (event_sender, event_receiver) = channel();

        if enable_input {
            app::input_binder::add_bindings(event_sender.clone());
        }

        self.run_win_events_loop(event_sender.clone());
        self.run_tiles_manager(event_receiver);
    }

    pub fn stop(&mut self) {
        self.win_events_loop_tx.as_ref().unwrap().send(0).unwrap();
        self.tiles_manager_loop_tx.as_ref().unwrap().send(0).unwrap();

        let _ = self.win_events_thread.take().unwrap().join();
        let _ = self.tiles_manager_thread.take().unwrap().join();

        info!("App::stop()");
    }

    fn run_win_events_loop(&mut self, event_sender: Sender<AppEvent>) {
        let (app_tx, app_rx) = mpsc::channel();
        self.win_events_loop_tx = Some(app_tx);

        let mut win_events_manager = WinEventManager::new();

        win_events_manager.add_handler(PositionEventHandler::new(event_sender.clone()));
        win_events_manager.add_handler(OpenCloseEventHandler::new(event_sender.clone()));
        win_events_manager.add_handler(MinimizeEventHandler::new(event_sender.clone()));

        self.win_events_thread = Some(thread::spawn(move || loop {
            let thread_event = app_rx.try_recv();
            if thread_event.is_ok_and(|v| v == 0) {
                print!("Good bye from WinEventsLoop!\n");
                break;
            }
            win_events_manager.check_for_events();
        }));
    }

    fn run_tiles_manager(&mut self, event_receiver: Receiver<AppEvent>) {
        let (app_tx, app_rx) = mpsc::channel();
        self.tiles_manager_loop_tx = Some(app_tx);

        let filter = self.app_configs.filter.clone().unwrap();
        let windows = enum_user_manageable_windows()
            .iter()
            .filter(|w| w.snapshot().is_some_and(|wi| !filter.filter(&wi)))
            .cloned()
            .collect();

        let mut tm = TilesManager::new(
            MonitorsLayout::new(enum_display_monitors()),
            GoldenRatioLayoutStrategy::new(),
        );

        tm.add_all(windows);
        Self::print_all_windows(&tm.get_windows());

        self.tiles_manager_thread = Some(thread::spawn(move || loop {
            let thread_event = app_rx.try_recv();
            if thread_event.is_ok_and(|v| v == 0) {
                print!("Good bye from TilesManager!\n");
                break;
            }

            match Self::filter_events(event_receiver.try_recv(), &filter) {
                Ok(app_event) => Self::handle_tiles_manager(&mut tm, app_event),
                Err(error) => {
                    info!("Error: {:?}", error);
                    break;
                }
            }
        }));
    }

    fn handle_tiles_manager<L: AreaTreeLayoutStrategy + Copy>(tm: &mut TilesManager<L>, event: AppEvent) {
        match event {
            AppEvent::Left => tm.select_near(Direction::Left),
            AppEvent::Right => tm.select_near(Direction::Right),
            AppEvent::ListAll => Self::print_all_windows(&tm.get_windows()),
            AppEvent::UpdateLayout => tm.update(),
            AppEvent::WindowOpened(hwnd) | AppEvent::WindowRestored(hwnd) => tm.add_then_update(WindowRef::new(hwnd)),
            AppEvent::WindowClosed(hwnd) | AppEvent::WindowMinimized(hwnd) => tm.remove(hwnd),
            AppEvent::WindowMoved(hwnd, coords, orientation) => {
                tm.change_window_position(hwnd, (coords.0 as u32, coords.1 as u32), orientation)
            }
            AppEvent::WindowResized(hwnd) => tm.refresh_window_size(hwnd),
            _ => (),
        }
    }

    fn filter_events(
        result: Result<AppEvent, TryRecvError>,
        win_filter: &WinMatchAnyFilters,
    ) -> Result<AppEvent, TryRecvError> {
        let event = match result {
            Ok(event) => event,
            _ => return Ok(AppEvent::Noop),
        };
        info!("Received: {:?}", event);
        Ok(match event {
            AppEvent::Left | AppEvent::Right | AppEvent::ListAll | AppEvent::UpdateLayout => event,
            AppEvent::WindowClosed(_) => event,
            AppEvent::WindowOpened(hwnd)
            | AppEvent::WindowMinimized(hwnd)
            | AppEvent::WindowRestored(hwnd)
            | AppEvent::WindowResized(hwnd)
            | AppEvent::WindowMoved(hwnd, _, _) => {
                let win_info = match WindowRef::new(hwnd).snapshot() {
                    Some(win_info) => win_info,
                    None => return Ok(AppEvent::Noop),
                };
                info!("\t[Filtered: {}] {:?}", win_filter.filter(&win_info), win_info);
                match !win_filter.filter(&win_info) {
                    true => event,
                    false => AppEvent::Noop,
                }
            }
            _ => event,
        })
    }

    fn print_all_windows(windows: &[WindowRef]) {
        windows.iter().for_each(|w| info!("\t{:?}", w.snapshot()));
    }
}
