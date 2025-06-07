use super::configs::CoreModuleConfigs;
use super::lib::tm::command::TMCommand;
use super::lib::tm::public::TilesManagerCommands;
use super::lib::tm::public::TilesManagerEvents;
use super::lib::tm::result::TilesManagerError;
use super::lib::tm::TilesManager;
use crate::app::configs::AppConfig;
use crate::app::mondrian_message::MondrianMessage;
use crate::app::mondrian_message::MoveSizeResult;
use crate::app::mondrian_message::SystemEvent;
use crate::app::mondrian_message::WindowEvent;
use crate::app::structs::area::Area;
use crate::app::structs::info_entry::InfoEntry;
use crate::app::structs::info_entry::InfoEntryIcon;
use crate::modules::module_impl::ModuleImpl;
use crate::modules::utils;
use crate::modules::ConfigurableModule;
use crate::modules::Module;
use crate::win32::api::monitor::enum_display_monitors;
use crate::win32::window::window_obj::WindowObjInfo;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
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
        let tm_configs = self.configs.tm_configs.clone();

        let app_tx = self.bus_tx.clone();
        let animations_enabled = tm_configs.animation.animation_type.is_some();
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

        let mut tm = match TilesManager::create(Some(tm_configs), on_update_start, on_update_error, on_update_complete)
        {
            Ok(tm) => tm,
            Err(error) => {
                log::error!("TilesManager creation error: {:?}", error);
                return;
            }
        };
        tm.add_open_windows().ok();
        tm.update_layout(true, None).ok();

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

        let (event_sender, event_receiver) = crossbeam_channel::unbounded();
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

    fn handle(&mut self, event: &MondrianMessage, app_configs: &AppConfig, _tx: &Sender<MondrianMessage>) {
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
            MondrianMessage::HealthCheckPing => utils::send_pong(&Module::name(self), &self.bus_tx),
            MondrianMessage::Quit => Module::stop(self),
            msg => {
                if let Ok(command) = msg.try_into() {
                    self.send_to_tm(command);
                }
            }
        }
    }

    fn name(&self) -> String {
        "tiles-manager".to_string()
    }
}

impl ConfigurableModule for TilesManagerModule {
    type Config = CoreModuleConfigs;
    fn configure(&mut self, config: Self::Config) {
        self.configs = config;
    }
}

fn handle_tm(tm: &mut TilesManager, tx: &Sender<MondrianMessage>, event: TMCommand) -> bool {
    let prev_wins = tm.get_visible_managed_windows();
    tm.check_for_vd_changes()
        .inspect_err(|m| log::trace!("VD changes check error: {m:?}"))
        .ok();

    let res = match event.clone() {
        TMCommand::WindowEvent(window_event) => match window_event {
            WindowEvent::Maximized(winref) => {
                // INFO: If Maximized event is fired after StartMoveSize
                // and before EndMoveSize
                tm.pause_updates(false);
                tm.on_maximize(winref, true)
            }
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
            WindowEvent::Repositioned(w) => tm.reposition_window(w),
            WindowEvent::Focused(winref) => tm.on_focus(winref),
        },
        TMCommand::SystemEvent(evt) => match evt {
            SystemEvent::VirtualDesktopRemoved { destroyed, fallback } => tm.on_vd_destroyed(destroyed, fallback),
            SystemEvent::VirtualDesktopChanged { old, new } => tm.on_vd_changed(old, new),
            SystemEvent::WorkareaChanged => tm.on_workarea_changed(),
            SystemEvent::DesktopFocused { at } => tm.on_desktop_focus(at),
            _ => Ok(()),
        },
        TMCommand::Focus(direction) => tm.change_focus(direction),
        TMCommand::FocusMonitor(direction) => tm.change_focus_monitor(direction),
        TMCommand::FocusWorkspace { id, monitor } => tm.focus_workspace(&id, monitor.as_deref()),
        TMCommand::MoveToWorkspace { id, focus, monitor } => {
            tm.move_focused_to_workspace(&id, focus, monitor.as_deref())
        }
        TMCommand::SwitchFocus => tm.switch_focus(),
        TMCommand::Minimize => tm.minimize_focused(),
        TMCommand::Close => tm.close_focused(),
        TMCommand::Topmost => tm.topmost_focused(None),
        TMCommand::Insert(direction) => tm.insert_focused(direction),
        TMCommand::Move(direction, insert_if_empty, floating_inc) => match tm.move_focused(direction, floating_inc) {
            Err(TilesManagerError::NoWindow) if insert_if_empty => tm.insert_focused(direction),
            res => res,
        },
        TMCommand::Resize(direction, inc, floating_inc) => tm.resize_focused(direction, inc, floating_inc),
        TMCommand::Invert => tm.invert_orientation(),
        TMCommand::Release(b) => tm.release_focused(b),
        TMCommand::Peek(direction, ratio) => tm.peek_current(direction, ratio),
        TMCommand::Focalize => tm.focalize_focused(),
        TMCommand::HalfFocalize => tm.half_focalize_focused(),
        TMCommand::Amplify => tm.amplify_focused(),
        TMCommand::CycleFocalized(next) => tm.cycle_focalized_wins(next, None),
        TMCommand::ListManagedWindows => {
            let windows = tm.get_visible_managed_windows();
            tx.send(MondrianMessage::UpdatedWindows(windows, event.clone())).ok();
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
        let windows = tm.get_visible_managed_windows();
        if windows != prev_wins {
            tx.send(MondrianMessage::UpdatedWindows(windows, event)).unwrap();
        }
    }

    true
}

fn get_info_entries(tm: &TilesManager) -> Vec<InfoEntry> {
    let monitors = enum_display_monitors();
    let monitors_areas: Vec<(String, Area)> = monitors.iter().map(|m| (m.id.clone(), m.get_workspace())).collect();
    let windows = tm.get_visible_managed_windows();
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
