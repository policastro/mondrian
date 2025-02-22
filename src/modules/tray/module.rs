use crate::app::assets::Asset;
use crate::app::configs::AppConfigs;
use crate::app::mondrian_message::MondrianMessage;
use crate::modules::module_impl::ModuleImpl;
use crate::modules::Module;
use crate::win32::api::misc::get_current_thread_id;
use crate::win32::api::misc::post_empty_thread_message;
use crate::win32::win_event_loop::next_win_event_loop_iteration;
use std::io::Cursor;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;
use tray_icon::menu::Menu;
use tray_icon::menu::MenuEvent;
use tray_icon::menu::MenuItem;
use tray_icon::menu::PredefinedMenuItem;
use tray_icon::Icon;
use tray_icon::TrayIconBuilder;
use windows::Win32::UI::WindowsAndMessaging::WM_NULL;
use windows::Win32::UI::WindowsAndMessaging::WM_QUIT;

pub struct TrayModule {
    bus: Sender<MondrianMessage>,
    main_thread: Option<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
    enabled: bool,
    main_thread_id: Arc<AtomicU32>,
    pause_flag: Arc<AtomicU8>,
}

impl TrayModule {
    const PAUSE_UNSET: u8 = 0;
    const PAUSE_DISABLED: u8 = 1;
    const PAUSE_ENABLED: u8 = 2;
    const PAUSE_TOGGLED: u8 = 3;
    pub fn new(bus: Sender<MondrianMessage>) -> Self {
        Self {
            bus,
            main_thread: None,
            running: Arc::new(AtomicBool::new(false)),
            enabled: true,
            main_thread_id: Arc::new(AtomicU32::new(0)),
            pause_flag: Arc::new(AtomicU8::new(Self::PAUSE_UNSET)),
        }
    }

    pub fn refresh_tray(&self) {
        if self.running.load(Ordering::SeqCst) {
            post_empty_thread_message(self.main_thread_id.load(Ordering::SeqCst), WM_NULL);
        }
    }

    pub fn get_icon(asset_path: &str) -> Option<Icon> {
        let icon_asset = Asset::get(asset_path)?;
        let icon_dir = ico::IconDir::read(Cursor::new(icon_asset.data)).ok()?;
        let icon_rgba = icon_dir.entries()[0].decode().ok()?.rgba_data().to_vec();
        Icon::from_rgba(icon_rgba, 256, 256).ok()
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
        let pause_flag = self.pause_flag.clone();

        let main_thread_id = self.main_thread_id.clone();
        let main_thread = thread::spawn(move || {
            let icon_normal = Self::get_icon("mondrian.ico").unwrap();
            let icon_gray = Self::get_icon("mondrian_gray.ico").unwrap();
            let tray_menu = Menu::new();
            let retile = MenuItem::with_id("RETILE", "⊞ Retile", true, None);
            let open_config = MenuItem::with_id("OPEN_CONFIG", "⚙️ Open config file", true, None);
            let refresh_config = MenuItem::with_id("REFRESH_CONFIG", "⟳ Refresh config", true, None);
            let pause = tray_icon::menu::CheckMenuItem::with_id("PAUSE", "⏯ Pause", true, false, None);
            let separator_1 = PredefinedMenuItem::separator();
            let open_log_folder = MenuItem::with_id("OPEN_LOG_FOLDER", "📂 Open logs folder", true, None);
            let about = MenuItem::with_id("ABOUT", "ⓘ About", true, None);
            let separator_2 = PredefinedMenuItem::separator();
            let quit = MenuItem::with_id("QUIT", "✖ Quit", true, None);

            tray_menu
                .append_items(&[
                    &retile,
                    &open_config,
                    &refresh_config,
                    &pause,
                    &separator_1,
                    &open_log_folder,
                    &about,
                    &separator_2,
                    &quit,
                ])
                .expect("Failed to append items");

            let tray_icon = TrayIconBuilder::new()
                .with_menu(Box::new(tray_menu.clone()))
                .with_tooltip("Mondrian")
                .with_icon(icon_normal.clone())
                .build()
                .unwrap();

            main_thread_id.store(get_current_thread_id(), Ordering::SeqCst);

            while next_win_event_loop_iteration(None) && running.load(Ordering::SeqCst) {
                let is_paused = pause_flag.load(Ordering::Relaxed);
                match is_paused {
                    Self::PAUSE_ENABLED => pause.set_checked(true),
                    Self::PAUSE_DISABLED => pause.set_checked(false),
                    Self::PAUSE_TOGGLED => pause.set_checked(!pause.is_checked()),
                    _ => {}
                }

                if !matches!(is_paused, Self::PAUSE_UNSET) {
                    let (icon, tooltip) = match pause.is_checked() {
                        true => (Some(icon_gray.clone()), "Mondrian (paused)"),
                        false => (Some(icon_normal.clone()), "Mondrian"),
                    };

                    let _ = tray_icon.set_icon(icon);
                    let _ = tray_icon.set_tooltip(Some(tooltip));
                    pause_flag.store(Self::PAUSE_UNSET, Ordering::Relaxed);
                }

                let event_id = MenuEvent::receiver()
                    .try_recv()
                    .map_or(None, |e| Some(e.id.0.to_owned()));

                let app_event = match event_id.as_deref() {
                    Some("PAUSE") => Some(MondrianMessage::Pause(Some(pause.is_checked()))),
                    Some("RETILE") => Some(MondrianMessage::Retile),
                    Some("REFRESH_CONFIG") => Some(MondrianMessage::RefreshConfig),
                    Some("OPEN_LOG_FOLDER") => Some(MondrianMessage::OpenLogFolder),
                    Some("OPEN_CONFIG") => Some(MondrianMessage::OpenConfig),
                    Some("ABOUT") => Some(MondrianMessage::About),
                    Some("QUIT") => Some(MondrianMessage::Quit),
                    _ => None,
                };

                match app_event {
                    Some(e) if e == MondrianMessage::Quit => {
                        bus.send(e).expect("TrayModule: Failed to send event");
                        break;
                    }
                    Some(e) => bus.send(e).expect("TrayModule: Failed to send event"),
                    _ => {}
                }
                thread::sleep(std::time::Duration::from_millis(200));
            }
        });
        self.main_thread = Some(main_thread);
    }

    fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(main_thread) = self.main_thread.take() {
            post_empty_thread_message(self.main_thread_id.load(Ordering::SeqCst), WM_QUIT);
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

    fn handle(&mut self, event: &MondrianMessage, _app_configs: &AppConfigs) {
        match event {
            MondrianMessage::Pause(pause) => {
                let pause = match pause {
                    Some(p) => match *p {
                        true => Self::PAUSE_ENABLED,
                        false => Self::PAUSE_DISABLED,
                    },
                    None => Self::PAUSE_TOGGLED,
                };
                self.pause_flag.store(pause, Ordering::Relaxed);
                self.refresh_tray();
            }
            MondrianMessage::Quit => Module::stop(self),
            _ => {}
        }
    }

    fn name(&self) -> String {
        "tray".to_string()
    }
}
