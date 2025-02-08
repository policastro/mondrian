use std::hash::{Hash, Hasher};

use windows::Win32::{
    Foundation::HWND,
    Graphics::Gdi::{RedrawWindow, RDW_ALLCHILDREN, RDW_FRAME, RDW_INTERNALPAINT, RDW_INVALIDATE},
    UI::WindowsAndMessaging::{
        IsIconic, IsWindowVisible, SetWindowPos, HWND_NOTOPMOST, HWND_TOPMOST, SET_WINDOW_POS_FLAGS, SWP_NOMOVE,
        SWP_NOSIZE, SW_MINIMIZE, SW_RESTORE, SW_SHOWNOACTIVATE, SW_SHOWNORMAL,
    },
};

use super::{
    window_obj::{WindowObjHandler, WindowObjInfo},
    window_snapshot::WindowSnapshot,
};
use crate::win32::api::window::{get_dpi_for_window, get_dwmwa_extended_frame_bounds};
use crate::{
    app::structs::area::Area,
    win32::api::window::{
        focus, get_class_name, get_executable_name, get_window_box, get_window_rect, get_window_style,
        get_window_title, is_window_cloaked, show_window,
    },
};

#[derive(Debug, Clone, Copy, Eq)]
pub struct WindowRef {
    pub hwnd: HWND,
}

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

impl From<HWND> for WindowRef {
    fn from(hwnd: HWND) -> Self {
        WindowRef { hwnd }
    }
}

impl From<isize> for WindowRef {
    fn from(hwnd: isize) -> Self {
        HWND(hwnd).into()
    }
}

impl From<WindowRef> for WindowSnapshot {
    fn from(val: WindowRef) -> Self {
        val.snapshot()
    }
}

impl WindowRef {
    pub fn new(hwnd: HWND) -> WindowRef {
        WindowRef { hwnd }
    }

    pub fn snapshot(&self) -> WindowSnapshot {
        WindowSnapshot {
            hwnd: self.hwnd,
            exe_name: self.get_exe_name(),
            title: self.get_title(),
            viewarea: self.get_window_box(),
            class_name: self.get_class_name(),
            style: self.get_window_style(),
            visible: self.is_visible(),
            iconic: self.is_iconic(),
            cloaked: self.is_cloaked(),
        }
    }

    pub fn adjust_area(&self, area: Area) -> Area {
        let w = match get_window_rect(self.hwnd) {
            Some(w) => w,
            _ => return area,
        };

        // INFO: used to find windows invisible borders
        let frame = get_dwmwa_extended_frame_bounds(self.hwnd).unwrap_or([0, 0, 0, 0]);
        let dpi: f32 = get_dpi_for_window(self.hwnd) as f32 / 96.0;
        let th = (
            ((frame[0] as f32 / dpi) - w[0] as f32).round() as i32,
            ((frame[1] as f32 / dpi) - w[1] as f32).round() as i32,
            ((frame[2] as f32 / dpi) - w[2] as f32).round() as i32,
            ((frame[3] as f32 / dpi) - w[3] as f32).round() as i32,
        );
        let th = (th.0, th.1, th.0 - th.2, th.1 - th.3);

        Area {
            x: area.x - th.0,
            y: area.y - th.1,
            width: (area.width as i32 + th.2).max(0) as u16,
            height: (area.height as i32 + th.3).max(0) as u16,
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

    fn get_window_box(&self) -> Option<Area> {
        // INFO: some apps (like Windows settings) have negative values (-8)
        get_window_box(self.hwnd)
            .map(|b| Area::new(b[0], b[1], b[2].try_into().unwrap_or(0), b[3].try_into().unwrap_or(0)))
    }

    fn get_class_name(&self) -> Option<String> {
        Some(get_class_name(self.hwnd))
    }

    fn get_window_style(&self) -> u32 {
        get_window_style(self.hwnd)
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
                show_window(self.hwnd, SW_SHOWNORMAL); // INFO: remove maximized state
            }

            match SetWindowPos(self.hwnd, None, pos.0, pos.1, size.0, size.1, flags) {
                Ok(_) => Ok(()),
                Err(_) => Err(()),
            }
        }
    }

    fn redraw(&self) -> Result<(), ()> {
        unsafe {
            let _ = RedrawWindow(
                self.hwnd,
                None,
                None,
                RDW_INTERNALPAINT | RDW_FRAME | RDW_INVALIDATE | RDW_ALLCHILDREN,
            );
        }
        Ok(())
    }

    fn minimize(&self) -> bool {
        show_window(self.hwnd, SW_MINIMIZE)
    }

    fn restore(&self, activate: bool) -> bool {
        show_window(self.hwnd, if activate { SW_RESTORE } else { SW_SHOWNOACTIVATE })
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
}
