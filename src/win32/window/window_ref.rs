use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

use windows::Win32::UI::WindowsAndMessaging::HWND_TOP;
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        IsIconic, IsWindowVisible, SetWindowPos, HWND_NOTOPMOST, HWND_TOPMOST, SET_WINDOW_POS_FLAGS, SWP_NOMOVE,
        SWP_NOSIZE, SW_MINIMIZE, SW_RESTORE, SW_SHOWMINNOACTIVE, SW_SHOWNOACTIVATE, SW_SHOWNORMAL, WM_CLOSE,
    },
};

use super::{
    window_obj::{WindowObjHandler, WindowObjInfo},
    window_snapshot::WindowSnapshot,
};
use crate::{
    app::area_tree::leaf::AreaLeaf,
    win32::api::{
        misc::post_empty_message,
        window::{get_dwmwa_extended_frame_bounds, is_fullscreen, is_maximized, is_window_topmost},
    },
};
use crate::{
    app::structs::area::Area,
    win32::api::window::{
        focus, get_class_name, get_executable_name, get_window_box, get_window_style, get_window_title,
        is_window_cloaked, show_window,
    },
};

#[derive(Debug, Clone, Copy, Eq)]
pub struct WindowRef {
    pub hwnd: HWND,
}

// NOTE: makes HWND thread safe
unsafe impl Send for WindowRef {}
unsafe impl Sync for WindowRef {}

impl Hash for WindowRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hwnd.0.hash(state);
    }
}

impl PartialEq for WindowRef {
    fn eq(&self, other: &Self) -> bool {
        self.hwnd.0 == other.hwnd.0
    }
}

impl Display for WindowRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.hwnd)
    }
}

impl From<HWND> for WindowRef {
    fn from(hwnd: HWND) -> Self {
        WindowRef { hwnd }
    }
}

impl From<WindowRef> for HWND {
    fn from(val: WindowRef) -> Self {
        val.hwnd
    }
}

impl From<WindowRef> for isize {
    fn from(val: WindowRef) -> Self {
        val.hwnd.0 as isize
    }
}

impl From<isize> for WindowRef {
    fn from(hwnd: isize) -> Self {
        HWND(hwnd as *mut core::ffi::c_void).into()
    }
}

impl From<WindowRef> for WindowSnapshot {
    fn from(val: WindowRef) -> Self {
        val.snapshot()
    }
}

impl From<AreaLeaf<WindowRef>> for WindowRef {
    fn from(val: AreaLeaf<WindowRef>) -> Self {
        val.id
    }
}
impl From<&AreaLeaf<WindowRef>> for WindowRef {
    fn from(val: &AreaLeaf<WindowRef>) -> Self {
        val.id
    }
}

impl WindowRef {
    pub fn new(hwnd: HWND) -> WindowRef {
        WindowRef { hwnd }
    }

    pub fn snapshot(&self) -> WindowSnapshot {
        WindowSnapshot {
            hwnd: self.hwnd,
            title: self.get_title(),
            exe_name: self.get_exe_name(),
            class_name: self.get_class_name(),
            style: self.get_window_style(),
            iconic: self.is_iconic(),
            visible: self.is_visible(),
            area: self.get_area(),
            visible_area: self.get_visible_area(),
            borders: self.get_borders(),
            cloaked: self.is_cloaked(),
            topmost: self.is_topmost(),
            fullscreen: self.is_fullscreen(),
            maximized: self.is_maximized(),
        }
    }
}

impl WindowObjInfo for WindowRef {
    fn get_title(&self) -> Option<String> {
        get_window_title(self.hwnd)
    }

    fn get_exe_name(&self) -> Option<String> {
        get_executable_name(self.hwnd)
    }

    fn get_class_name(&self) -> Option<String> {
        Some(get_class_name(self.hwnd))
    }

    fn get_area(&self) -> Option<Area> {
        let b = get_window_box(self.hwnd)?;
        Some(Area::new(
            b[0],
            b[1],
            b[2].clamp(0, u16::MAX as i32) as u16,
            b[3].clamp(0, u16::MAX as i32) as u16,
        ))
    }

    fn get_visible_area(&self) -> Option<Area> {
        let frame = get_dwmwa_extended_frame_bounds(self.hwnd)?;
        Some(Area::new(
            frame[0] as i32,
            frame[1] as i32,
            (frame[2] - frame[0]).clamp(0, u16::MAX as i32) as u16,
            (frame[3] - frame[1]).clamp(0, u16::MAX as i32) as u16,
        ))
    }

    fn get_borders(&self) -> Option<(i32, i32, i32, i32)> {
        let area = self.get_area()?;
        let visible = self.get_visible_area()?;
        let (l, t) = (visible.x - area.x, visible.y - area.y);
        Some((
            l,
            t,
            (area.width as i32 - visible.width as i32) - l,
            (area.height as i32 - visible.height as i32) - t,
        ))
    }

    fn is_visible(&self) -> bool {
        unsafe { IsWindowVisible(self.hwnd) }.as_bool()
    }

    fn is_iconic(&self) -> bool {
        unsafe { IsIconic(self.hwnd) }.as_bool()
    }

    fn is_cloaked(&self) -> bool {
        is_window_cloaked(self.hwnd)
    }

    fn is_topmost(&self) -> bool {
        is_window_topmost(self.hwnd)
    }

    fn is_fullscreen(&self) -> bool {
        is_fullscreen(self.hwnd)
    }

    fn is_maximized(&self) -> bool {
        is_maximized(self.hwnd)
    }

    fn get_window_style(&self) -> u32 {
        get_window_style(self.hwnd)
    }
}

impl WindowObjHandler for WindowRef {
    fn focus(&self) {
        focus(self.hwnd);
    }

    fn resize_and_move(
        &self,
        pos: (i32, i32),
        size: (u16, u16),
        force_normal: bool,
        flags: SET_WINDOW_POS_FLAGS,
    ) -> Result<(), ()> {
        unsafe {
            let size = (i32::from(size.0), i32::from(size.1));

            if force_normal {
                self.set_normal();
            }

            match SetWindowPos(self.hwnd, None, pos.0, pos.1, size.0, size.1, flags) {
                Ok(_) => Ok(()),
                Err(_) => Err(()),
            }
        }
    }

    fn set_normal(&self) {
        show_window(self.hwnd, SW_SHOWNORMAL);
    }

    fn minimize(&self, move_focus: bool) -> bool {
        match move_focus {
            true => show_window(self.hwnd, SW_MINIMIZE),
            false => show_window(self.hwnd, SW_SHOWMINNOACTIVE),
        }
    }

    fn restore(&self, activate: bool) -> bool {
        show_window(self.hwnd, if activate { SW_RESTORE } else { SW_SHOWNOACTIVATE })
    }

    fn close(&self) {
        post_empty_message(self.hwnd, WM_CLOSE);
    }

    fn set_topmost(&self, topmost: bool) -> Result<(), ()> {
        let hwnd_flag = if topmost { HWND_TOPMOST } else { HWND_NOTOPMOST };
        unsafe {
            match SetWindowPos(self.hwnd, hwnd_flag, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE) {
                Ok(_) => Ok(()),
                Err(_) => Err(()),
            }
        }
    }

    fn to_front(&self) {
        unsafe { SetWindowPos(self.hwnd, HWND_TOP, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE).ok() };
    }
}
