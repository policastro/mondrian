use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{SetForegroundWindow, SetWindowPos, HWND_TOP, SWP_NOSENDCHANGING, SWP_SHOWWINDOW},
};

use crate::{
    app::structs::area::Area,
    win32utils::api::window::{get_class_name, get_executable_name, get_window_box, get_window_title},
};

use super::{window_obj::WindowObj, window_snapshot::WindowSnapshot};

#[derive(Debug, Clone, Copy)]
pub struct WindowRef {
    pub hwnd: HWND,
}

impl WindowRef {
    pub fn new(hwnd: HWND) -> WindowRef {
        WindowRef { hwnd }
    }

    pub fn snapshot(&self) -> Option<WindowSnapshot> {
        Some(WindowSnapshot {
            hwnd: self.hwnd,
            exe_name: self.get_exe_name()?,
            title: self.get_title()?,
            viewarea: self.get_window_box()?,
            class_name: self.get_class_name()?,
        })
    }
}

impl PartialEq for WindowRef {
    fn eq(&self, other: &Self) -> bool {
        self.hwnd.0 == other.hwnd.0
    }
}

impl From<isize> for WindowRef {
    fn from(hwnd: isize) -> Self {
        WindowRef { hwnd: HWND(hwnd) }
    }
}

impl Into<WindowSnapshot> for WindowRef {
    fn into(self) -> WindowSnapshot {
        self.snapshot().unwrap()
    }
}

impl WindowObj for WindowRef {
    fn get_title(&self) -> Option<String> {
        get_window_title(self.hwnd)
    }

    fn get_exe_name(&self) -> Option<String> {
        get_executable_name(self.hwnd)
    }

    fn get_window_box(&self) -> Option<Area> {
        get_window_box(self.hwnd).map(Area::from)
    }

    fn focus(&self) {
        unsafe { let _ = SetForegroundWindow(self.hwnd); };
    }

    fn resize_and_move(&self, coordinates: (u32, u32), size: (u32, u32)) {
        unsafe {
            match SetWindowPos(
                self.hwnd,
                HWND_TOP,
                coordinates.0 as i32,
                coordinates.1 as i32,
                size.0 as i32,
                size.1 as i32,
                SWP_NOSENDCHANGING | SWP_SHOWWINDOW,
            ) {
                Ok(_) => (),
                Err(err) => {
                    log::error!("SetWindowPos failed for window {:?}: {}", self.hwnd, err);
                }
            }
        };
    }

    fn get_class_name(&self) -> Option<String> {
        Some(get_class_name(self.hwnd))
    }
}
