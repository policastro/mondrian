use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_QUIT};

use crate::{
    app::{config::app_configs::AppConfigs, mondrian_command::MondrianCommand},
    modules::module::{module_impl::ModuleImpl, ConfigurableModule, Module},
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

impl OverlayModule {
    pub fn new() -> OverlayModule {
        OverlayModule {
            main_thread: None,
            running: Arc::new(AtomicBool::new(false)),
            configs: OverlayConfig::default(),
            enabled: true,
        }
    }
}

impl ModuleImpl for OverlayModule {
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
            let overlay_hwnd = overlay.hwnd;

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

    fn handle(&mut self, event: &MondrianCommand, app_configs: &AppConfigs) {
        match event {
            MondrianCommand::Pause(pause) => Module::pause(self, *pause),
            MondrianCommand::Configure => {
                Module::enable(self, app_configs.overlay_enabled);
                self.configure(app_configs.into());
            }
            MondrianCommand::RefreshConfig => {
                Module::enable(self, app_configs.overlay_enabled);
                self.configure(app_configs.into());
                Module::restart(self);
            }
            MondrianCommand::Quit => Module::stop(self),
            _ => {}
        }
    }
}

impl ConfigurableModule for OverlayModule {
    type Config = OverlayConfig;
    fn configure(&mut self, config: Self::Config) {
        self.configs = config;
    }
}
