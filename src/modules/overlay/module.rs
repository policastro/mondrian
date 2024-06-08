use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_QUIT};

use crate::{
    modules::module::Module,
    win32::{
        win_event_loop::{next_win_event_loop_no_block, start_mono_win_event_loop},
        win_events_manager::WinEventManagerBuilder,
    },
};

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use super::{
    configs::OverlayConfig,
    lib::{focus_event_handler::FocusEventHandler, window_overlay::WindowOverlay},
};

pub struct OverlayModule {
    main_thread: Option<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
    configs: OverlayConfig,
    enabled: bool,
}

impl Module for OverlayModule {
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

impl OverlayModule {
    pub fn new() -> OverlayModule {
        OverlayModule {
            main_thread: None,
            running: Arc::new(AtomicBool::new(false)),
            configs: OverlayConfig::default(),
            enabled: true,
        }
    }

    pub fn configure(&mut self, config: OverlayConfig) {
        self.configs = config;
    }

    pub fn pause(&mut self, is_paused: bool) {
        if is_paused {
            self.stop();
        } else {
            self.start();
        }
    }

    fn start(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        let thickness = self.configs.thickness;
        let border_color = self.configs.color;
        let padding = self.configs.padding;
        let main_thread = thread::spawn(move || {
            let overlay = WindowOverlay::new(thickness, border_color, padding);
            let overlay_hwnd = overlay.hwnd.clone();

            let wem_builder = WinEventManagerBuilder::new().handler(FocusEventHandler::new(overlay));
            thread::spawn(move || {
                let mut wem = wem_builder.build();
                while running.load(Ordering::SeqCst) {
                    wem.check_for_events();
                    next_win_event_loop_no_block(None);
                    thread::sleep(std::time::Duration::from_millis(20));
                }
                let _ = unsafe { PostMessageW(overlay_hwnd, WM_QUIT, None, None) };
            });
            start_mono_win_event_loop(overlay_hwnd);
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
