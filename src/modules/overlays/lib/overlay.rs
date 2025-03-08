use super::color::Color;
use super::utils;
use super::utils::overlay::OverlayBase;
use crate::win32::api::misc::post_empyt_message;
use crate::win32::api::misc::post_message;
use crate::win32::api::window::destroy_window;
use crate::win32::api::window::show_window;
use crate::win32::win_event_loop::start_win_event_loop;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;
use std::sync::atomic::AtomicIsize;
use std::sync::Arc;
use std::thread;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::SetWindowPos;
use windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE;
use windows::Win32::UI::WindowsAndMessaging::SWP_NOSENDCHANGING;
use windows::Win32::UI::WindowsAndMessaging::SWP_SHOWWINDOW;
use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNA;
use windows::Win32::UI::WindowsAndMessaging::WM_QUIT;

pub struct Overlay<P: OverlayBase + Clone + PartialEq + Copy + Send + 'static> {
    target_handle: HWND,
    main_thread: Option<thread::JoinHandle<()>>,
    overlay_handle: Arc<AtomicIsize>,
    current_params: Option<P>,
    class_name: String,
}

impl<P: OverlayBase + Clone + PartialEq + Send + Copy> Overlay<P> {
    pub fn new(target_handle: HWND, class_name: &str) -> Overlay<P> {
        Overlay::<P> {
            class_name: class_name.to_string(),
            target_handle,
            overlay_handle: Arc::new(AtomicIsize::new(0)),
            main_thread: None,
            current_params: None,
        }
    }

    pub fn create(&mut self, params: P) {
        if self.exists() {
            return;
        }

        self.current_params = Some(params);

        let class_name = self.class_name.clone();
        let overlay_handle = self.overlay_handle.clone();
        let target = self.target_handle;
        let main_thread = thread::spawn(move || {
            let hwnd = utils::overlay::create(params, Some(target), class_name.as_str());
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
            post_empyt_message(handle, WM_QUIT);
        }

        if let Some(th) = self.main_thread.take() {
            th.join().unwrap();
        }
    }

    pub fn configure(&mut self, params: P) {
        if self.current_params.filter(|p| *p == params).is_none() {
            self.current_params = Some(params);

            if let Some(o) = self.get_overlay_handle() {
                post_message(o, utils::overlay::WM_USER_CONFIGURE, Some(params));
                Self::move_overlay_to_target(o, self.target_handle, &params);
            }
        }
    }

    fn get_overlay_handle(&self) -> Option<HWND> {
        match self.overlay_handle.load(std::sync::atomic::Ordering::Acquire) {
            0 => None,
            o => Some(HWND(o)),
        }
    }

    pub fn reposition(&mut self, params: Option<P>) {
        let overlay_handle = self.get_overlay_handle();
        if self.current_params.zip(params).is_some_and(|(curr_p, p)| p != curr_p) {
            self.current_params = params;
            if let Some(o) = overlay_handle {
                post_message(o, utils::overlay::WM_USER_CONFIGURE, self.current_params);
            }
        }

        if let Some((o, p)) = overlay_handle.zip(self.current_params) {
            Self::move_overlay_to_target(o, self.target_handle, &p);
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

    fn move_overlay_to_target(overlay: HWND, target: HWND, params: &P) {
        let target_area = utils::overlay::get_box_from_target(target, params.get_thickness(), params.get_padding());
        let (x, y, cx, cy) = match target_area {
            Some(b) => b.into(),
            None => return,
        };

        let overlay_area = WindowRef::new(overlay).get_area();
        if target_area.zip(overlay_area).is_some_and(|(ta, oa)| ta == oa) {
            return;
        }
        let flags = SWP_NOSENDCHANGING | SWP_SHOWWINDOW | SWP_NOACTIVATE;
        let _ = unsafe { SetWindowPos(overlay, target, x, y, cx, cy, flags) };
    }
}

impl<P: OverlayBase + Clone + PartialEq + Send + Copy> Drop for Overlay<P> {
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
        OverlayParams::new(false, Color::new(0, 0, 0), 0, 0, 0)
    }
}
