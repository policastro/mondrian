use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    Icon, TrayIconBuilder,
};
use windows::Win32::{
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::{PostThreadMessageW, WM_QUIT},
};

use crate::{
    app::{config::app_configs::AppConfigs, mondrian_command::MondrianCommand},
    modules::module::{module_impl::ModuleImpl, Module},
    win32::win_event_loop::next_win_event_loop_iteration,
};

use std::{
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        mpsc::Sender,
        Arc,
    },
    thread,
};

pub struct TrayModule {
    bus: Sender<MondrianCommand>,
    main_thread: Option<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
    enabled: bool,
    main_thread_id: Arc<AtomicU32>,
}

impl TrayModule {
    pub fn new(bus: Sender<MondrianCommand>) -> Self {
        Self {
            bus,
            main_thread: None,
            running: Arc::new(AtomicBool::new(false)),
            enabled: true,
            main_thread_id: Arc::new(AtomicU32::new(0)),
        }
    }
}

impl ModuleImpl for TrayModule {
    fn start(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }
        let bus = self.bus.clone();
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        let main_thread_id = self.main_thread_id.clone();
        let main_thread = thread::spawn(move || {
            let icon = Icon::from_resource_name("APP_ICON", Some((256, 256))).unwrap();
            let tray_menu = Menu::new();
            let retile = MenuItem::with_id("RETILE", "Retile", true, None);
            let open_config = MenuItem::with_id("OPEN_CONFIG", "Open config file", true, None);
            let with_id = MenuItem::with_id("REFRESH_CONFIG", "Refresh config", true, None);
            let refresh_config = with_id;
            let pause = tray_icon::menu::CheckMenuItem::with_id("PAUSE", "Pause", true, false, None);
            let separator = PredefinedMenuItem::separator();
            let quit = MenuItem::with_id("QUIT", "Quit", true, None);

            tray_menu
                .append_items(&[&retile, &open_config, &refresh_config, &pause, &separator, &quit])
                .expect("Failed to append items");

            let _tray_icon = TrayIconBuilder::new()
                .with_menu(Box::new(tray_menu))
                .with_tooltip("Mondrian")
                .with_icon(icon)
                .build()
                .unwrap();

            let thread_id = unsafe { GetCurrentThreadId() };
            main_thread_id.store(thread_id, Ordering::SeqCst);

            while next_win_event_loop_iteration(None) && running.load(Ordering::SeqCst) {
                let event_id = MenuEvent::receiver()
                    .try_recv()
                    .map_or(None, |e| Some(e.id.0.to_owned()));

                let app_event = match event_id.as_deref() {
                    Some("PAUSE") => Some(MondrianCommand::Pause(pause.is_checked())),
                    Some("RETILE") => Some(MondrianCommand::Retile),
                    Some("REFRESH_CONFIG") => Some(MondrianCommand::RefreshConfig),
                    Some("OPEN_CONFIG") => Some(MondrianCommand::OpenConfig),
                    Some("QUIT") => Some(MondrianCommand::Quit),
                    _ => None,
                };

                match app_event {
                    Some(e) if e == MondrianCommand::Quit => {
                        bus.send(e).expect("TrayModule: Failed to send event");
                        break;
                    }
                    Some(e) => bus.send(e).expect("TrayModule: Failed to send event"),
                    _ => {}
                }
            }
        });
        self.main_thread = Some(main_thread);
    }

    fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(main_thread) = self.main_thread.take() {
            let _ = unsafe { PostThreadMessageW(self.main_thread_id.load(Ordering::SeqCst), WM_QUIT, None, None) };
            let _ = main_thread.join();
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

    fn handle(&mut self, event: &MondrianCommand, _app_configs: &AppConfigs) {
        if let MondrianCommand::Quit = event {
            Module::stop(self)
        }
    }
}
