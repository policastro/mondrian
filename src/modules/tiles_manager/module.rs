use super::configs::CoreModuleConfigs;
use super::lib::tm::command::TMCommand;
use super::lib::tm::configs::TilesManagerConfig;
use super::lib::tm::error::TilesManagerError;
use super::lib::tm::manager::TilesManager;
use super::lib::tm::manager::TilesManagerBase;
use super::lib::tm::public::TilesManagerOperations;
use crate::app::configs::AppConfigs;
use crate::app::mondrian_message::MondrianMessage;
use crate::app::mondrian_message::MoveSizeResult;
use crate::app::mondrian_message::SystemEvent;
use crate::app::mondrian_message::WindowEvent;
use crate::app::structs::area::Area;
use crate::app::structs::info_entry::InfoEntry;
use crate::app::structs::info_entry::InfoEntryIcon;
use crate::modules::module_impl::ModuleImpl;
use crate::modules::ConfigurableModule;
use crate::modules::Module;
use crate::win32::api::monitor::enum_display_monitors;
use crate::win32::window::window_obj::WindowObjInfo;
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
        let animations_enabled = self.configs.animations_enabled;
        let on_update_start = move |wins| {
            app_tx
                .send(MondrianMessage::CoreUpdateStart(wins, animations_enabled))
                .unwrap();
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
        let _ = tm.update_layout(true, None);

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
    let prev_wins = tm.get_managed_windows();
    tm.check_for_vd_changes()
        .inspect_err(|m| log::trace!("VD changes check error: {m:?}"))
        .ok();

    let res = match event {
        TMCommand::WindowEvent(window_event) => match window_event {
            WindowEvent::Maximized(winref) => tm.on_maximize(winref, true),
            WindowEvent::Unmaximized(winref) => tm.on_maximize(winref, false),
            WindowEvent::Opened(winref) => tm.on_open(winref),
            WindowEvent::Restored(winref) => tm.on_restore(winref),
            WindowEvent::Closed(winref) => tm.on_close(winref),
            WindowEvent::Minimized(winref) => tm.on_minimize(winref),
            WindowEvent::StartMoveSize(_) => {
                tm.pause_updates(true);
                tm.cancel_animation();
                Ok(())
            }
            WindowEvent::EndMoveSize(winref, res) => {
                tm.pause_updates(false);
                match res {
                    MoveSizeResult::Resized(p_area, c_area) => tm.on_resize(winref, c_area.get_shift(&p_area)),
                    MoveSizeResult::Moved(coords, intra, inter) => tm.on_move(winref, coords, intra, inter),
                    MoveSizeResult::None => Ok(()),
                }
            }
            WindowEvent::Focused(winref) => tm.on_focus(winref),
        },
        TMCommand::SystemEvent(evt) => match evt {
            SystemEvent::VirtualDesktopCreated { desktop } => tm.on_vd_created(desktop),
            SystemEvent::VirtualDesktopRemoved { destroyed, fallback } => tm.on_vd_destroyed(destroyed, fallback),
            SystemEvent::VirtualDesktopChanged { old, new } => tm.on_vd_changed(old, new),
            SystemEvent::WorkareaChanged => tm.on_workarea_changed(),
            _ => Ok(()),
        },
        TMCommand::Focus(direction) => tm.change_focus(direction, configs.move_cursor_on_focus),
        TMCommand::Minimize => tm.minimize_focused(),
        TMCommand::Insert(direction) => tm.move_focused(direction, configs.move_cursor_on_focus),
        TMCommand::Move(direction, insert_if_empty) => match tm.swap_focused(direction, configs.move_cursor_on_focus) {
            Err(TilesManagerError::NoWindow) if insert_if_empty => {
                tm.move_focused(direction, configs.move_cursor_on_focus)
            }
            res => res,
        },
        TMCommand::Resize(direction, size) => tm.resize_focused(direction, size),
        TMCommand::Invert => tm.invert_orientation(),
        TMCommand::Release(b) => tm.release_focused(b),
        TMCommand::Peek(direction, ratio) => tm.peek_current(direction, ratio),
        TMCommand::Focalize => tm.focalize_focused(),
        TMCommand::HalfFocalize => tm.half_focalize_focused(),
        TMCommand::Amplify => tm.amplify_focused(configs.move_cursor_on_focus),
        TMCommand::CycleFocalized(next) => tm.cycle_focalized_wins(next, None),
        TMCommand::ListManagedWindows => {
            let windows = tm.get_managed_windows();
            tx.send(MondrianMessage::UpdatedWindows(windows, event)).unwrap();
            Ok(())
        }
        TMCommand::QueryInfo => {
            tx.send(MondrianMessage::QueryInfoResponse {
                name: "Tiles Manager".to_string(),
                icon: InfoEntryIcon::TilesManager,
                infos: get_info_entries(tm),
            })
            .unwrap();
            Ok(())
        }
        TMCommand::Update(animate) => tm.update_layout(animate, None),
        TMCommand::Quit => return false,
    };

    match res {
        Err(error) if error.require_refresh() => {
            tm.update_layout(true, None).ok();
            log::warn!("TilesManager refreshed due to error: {:?}", error)
        }
        Err(error) => log::log!(error.get_log_level(), "{}", error.get_info()),
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

fn get_info_entries(tm: &TilesManager) -> Vec<InfoEntry> {
    let monitors = enum_display_monitors();
    let monitors_areas: Vec<(String, Area)> = monitors.iter().map(|m| (m.id.clone(), (*m).clone().into())).collect();
    let windows = tm.get_managed_windows();
    let windows_str = windows.iter().map(|w| (w.0.snapshot(), w.1)).map(|w| {
        let c = w.0.get_area().map(|a| a.get_center());
        let monitor = monitors_areas
            .iter()
            .find_map(|m| c.filter(|c| m.1.contains(*c)).map(|_| m.0.clone()));
        let monitor_info = InfoEntry::simple("Monitor", monitor.unwrap_or("/".to_string()));
        let state_info = InfoEntry::simple("State", format!("{:?}", w.1));

        InfoEntry::list(
            format!("Window {}", format!("{:?}", w.0).trim_start_matches("WindowSnapshot ")),
            [monitor_info, state_info],
        )
    });

    vec![
        InfoEntry::list("Monitors", monitors.iter().map(|m| format!("{m:?}").into())).with_icon(InfoEntryIcon::Monitor),
        InfoEntry::list("Currently managed windows", windows_str).with_icon(InfoEntryIcon::Window),
    ]
}
