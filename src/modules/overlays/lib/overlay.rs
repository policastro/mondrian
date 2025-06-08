use super::color::Color;
use super::utils::overlay;
use super::utils::overlay::WM_USER_CONFIGURE;
use crate::win32::api::misc::post_empty_message;
use crate::win32::api::misc::post_message;
use crate::win32::api::window::destroy_window;
use crate::win32::api::window::show_window;
use crate::win32::win_event_loop::start_win_event_loop;
use crate::win32::window::window_ref::WindowRef;
use std::sync::atomic::AtomicIsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::thread;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNA;
use windows::Win32::UI::WindowsAndMessaging::WM_QUIT;

enum InitState {
    Idle,
    Creating,
    Created,
}

pub struct Overlay {
    target: WindowRef,
    main_thread: Option<thread::JoinHandle<()>>,
    overlay_handle: Arc<AtomicIsize>,
    current_state: Arc<(Mutex<InitState>, Condvar)>,
    params: OverlayParams,
    class_name: String,
}

impl Overlay {
    pub fn new(target: WindowRef, class_name: &str, params: OverlayParams) -> Overlay {
        Overlay {
            class_name: class_name.to_string(),
            target,
            current_state: Arc::new((Mutex::new(InitState::Idle), Condvar::new())),
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
        let current_state = self.current_state.clone();

        Self::set_init_state(&self.current_state, InitState::Creating);

        let main_thread = thread::spawn(move || {
            let hwnd = match overlay::create(params, target.into(), class_name.as_str()) {
                Some(hwnd) => hwnd,
                None => {
                    Self::set_init_state(&current_state, InitState::Idle);
                    return;
                }
            };

            overlay_handle.store(hwnd.0 as isize, Ordering::Release);
            Self::set_init_state(&current_state, InitState::Created);
            start_win_event_loop();
            overlay_handle.store(0, Ordering::Release);
            destroy_window(hwnd);
            Self::set_init_state(&current_state, InitState::Idle);
        });

        self.main_thread = Some(main_thread);
    }

    pub fn exists(&self) -> bool {
        self.overlay_handle.load(std::sync::atomic::Ordering::Acquire) != 0 || self.main_thread.is_some()
    }

    pub fn destroy(&mut self) {
        if let Some(th) = self.main_thread.take() {
            // INFO: it takes some time to create the overlay
            let (lock, cvar) = &*self.current_state;
            // INFO: scope to release the lock before th.join()
            {
                let mut state = lock.lock().unwrap();
                while matches!(*state, InitState::Creating) {
                    state = cvar.wait(state).unwrap();
                }
            }

            if let Some(handle) = self.get_overlay_handle() {
                post_empty_message(handle, WM_QUIT);
            }
            th.join().unwrap();
        }
    }

    pub fn configure(&mut self, params: OverlayParams) -> bool {
        if params == self.params {
            return false;
        }

        self.params = params;
        if let Some(o) = self.get_overlay_handle() {
            overlay::move_to_target(o, self.target.into(), &self.params);
            post_message(o, WM_USER_CONFIGURE, Some(params));
        };

        true
    }

    pub fn get_overlay_handle(&self) -> Option<HWND> {
        match self.overlay_handle.load(std::sync::atomic::Ordering::Acquire) {
            0 => None,
            o => Some(HWND(o as *mut core::ffi::c_void)),
        }
    }

    pub fn reposition(&mut self) {
        if let Some(o) = self.get_overlay_handle() {
            overlay::move_to_target(o, self.target.into(), &self.params);
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

    fn set_init_state(state_variable: &Arc<(Mutex<InitState>, Condvar)>, state: InitState) {
        let (lock, cvar) = &**state_variable;
        let mut guard = lock.lock().unwrap();
        *guard = state;
        cvar.notify_all();
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
