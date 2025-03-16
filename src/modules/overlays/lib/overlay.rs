use super::color::Color;
use super::utils::overlay;
use super::utils::overlay::WM_USER_CONFIGURE;
use crate::win32::api::misc::post_empty_message;
use crate::win32::api::misc::post_message;
use crate::win32::api::window::destroy_window;
use crate::win32::api::window::show_window;
use crate::win32::win_event_loop::start_win_event_loop;
use std::sync::atomic::AtomicIsize;
use std::sync::Arc;
use std::thread;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNA;
use windows::Win32::UI::WindowsAndMessaging::WM_QUIT;

pub struct Overlay {
    target: HWND,
    main_thread: Option<thread::JoinHandle<()>>,
    overlay_handle: Arc<AtomicIsize>,
    params: OverlayParams,
    class_name: String,
}

impl Overlay {
    pub fn new(target_handle: HWND, class_name: &str, params: OverlayParams) -> Overlay {
        Overlay {
            class_name: class_name.to_string(),
            target: target_handle,
            overlay_handle: Arc::new(AtomicIsize::new(0)),
            params,
            main_thread: None,
        }
    }

    pub fn create(&mut self, params: Option<OverlayParams>) {
        if self.exists() {
            return;
        }

        self.params = params.unwrap_or(self.params);

        let class_name = self.class_name.clone();
        let overlay_handle = self.overlay_handle.clone();
        let target = self.target;
        let params = self.params;
        let main_thread = thread::spawn(move || {
            let hwnd = overlay::create(params, Some(target), class_name.as_str());
            if hwnd.0 == 0 {
                return;
            }

            overlay_handle.store(hwnd.0, std::sync::atomic::Ordering::Release);
            start_win_event_loop();
            overlay_handle.store(0, std::sync::atomic::Ordering::Release);
            destroy_window(hwnd);
        });

        self.main_thread = Some(main_thread);
    }

    pub fn exists(&self) -> bool {
        self.overlay_handle.load(std::sync::atomic::Ordering::Acquire) != 0 || self.main_thread.is_some()
    }

    pub fn destroy(&mut self) {
        if let Some(handle) = self.get_overlay_handle() {
            post_empty_message(handle, WM_QUIT);
        }

        if let Some(th) = self.main_thread.take() {
            th.join().unwrap();
        }
    }

    pub fn configure(&mut self, params: OverlayParams) -> bool {
        if params == self.params {
            return false;
        }

        self.params = params;
        if let Some(o) = self.get_overlay_handle() {
            overlay::move_to_target(o, self.target, &self.params);
            post_message(o, WM_USER_CONFIGURE, Some(params));
        };

        true
    }

    pub fn get_overlay_handle(&self) -> Option<HWND> {
        match self.overlay_handle.load(std::sync::atomic::Ordering::Acquire) {
            0 => None,
            o => Some(HWND(o)),
        }
    }

    pub fn reposition(&mut self) {
        if let Some(o) = self.get_overlay_handle() {
            overlay::move_to_target(o, self.target, &self.params);
        }
    }

    pub fn hide(&self) {
        if let Some(o) = self.get_overlay_handle() {
            show_window(o, SW_HIDE);
        }
    }

    pub fn show(&self) {
        if let Some(o) = self.get_overlay_handle() {
            show_window(o, SW_SHOWNA);
        }
    }
}

impl Drop for Overlay {
    fn drop(&mut self) {
        self.destroy();
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverlayParams {
    pub enabled: bool,
    pub color: Color,
    pub thickness: u8,
    pub border_radius: u8,
    pub padding: u8,
}

impl Default for OverlayParams {
    fn default() -> Self {
        OverlayParams::empty()
    }
}

impl OverlayParams {
    pub fn new(enabled: bool, color: Color, thickness: u8, border_radius: u8, padding: u8) -> OverlayParams {
        OverlayParams {
            enabled,
            color,
            thickness,
            border_radius,
            padding,
        }
    }

    pub fn empty() -> OverlayParams {
        OverlayParams::new(false, Color::solid(0, 0, 0), 0, 0, 0)
    }
}
