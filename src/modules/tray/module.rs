use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    Icon, TrayIconBuilder,
};

use crate::{
    app::mondrian_app_event::MondrianAppEvent, modules::module::Module,
    win32::win_event_loop::next_win_event_loop_iteration,
};

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Arc,
    },
    thread,
};

pub struct TrayModule {
    bus: Sender<MondrianAppEvent>,
    main_thread: Option<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
    enabled: bool,
}

impl Module for TrayModule {
    fn start(&mut self) {
        if self.enabled {
            self.start();
        }
    }

    fn stop(&mut self) {
        if self.enabled {
            self.stop();
        }
    }

    fn restart(&mut self) {
        if self.enabled {
            self.restart();
        }
    }

    fn enable(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.stop();
        }
    }
}

impl TrayModule {
    pub fn new(bus: Sender<MondrianAppEvent>) -> Self {
        Self {
            bus,
            main_thread: None,
            running: Arc::new(AtomicBool::new(false)),
            enabled: true,
        }
    }

    fn start(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }
        let bus = self.bus.clone();
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();
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

            while next_win_event_loop_iteration(None) && running.load(Ordering::SeqCst) {
                let event_id = MenuEvent::receiver()
                    .try_recv()
                    .map_or(None, |e| Some(e.id.0.to_owned()));

                let app_event = match event_id.as_deref() {
                    Some("PAUSE") => Some(MondrianAppEvent::Pause(pause.is_checked())),
                    Some("RETILE") => Some(MondrianAppEvent::Retile),
                    Some("REFRESH_CONFIG") => Some(MondrianAppEvent::RefreshConfig),
                    Some("OPEN_CONFIG") => Some(MondrianAppEvent::OpenConfig),
                    Some("QUIT") => Some(MondrianAppEvent::Quit),
                    _ => None,
                };

                match app_event {
                    Some(e) if e == MondrianAppEvent::Quit => {
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
            let _ = main_thread.join();
        }
    }

    fn restart(&mut self) {
        self.stop();
        self.start();
    }
}
