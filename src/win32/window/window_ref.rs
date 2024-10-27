use windows::Win32::{
    Foundation::HWND,
    Graphics::Gdi::{RedrawWindow, RDW_ALLCHILDREN, RDW_FRAME, RDW_INTERNALPAINT, RDW_INVALIDATE},
    UI::WindowsAndMessaging::{
        DeferWindowPos, IsIconic, IsWindowVisible, SetWindowPos, HDWP, SWP_ASYNCWINDOWPOS, SWP_FRAMECHANGED,
        SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOREDRAW, SWP_NOSENDCHANGING, SWP_NOZORDER, SWP_SHOWWINDOW, SW_MINIMIZE,
        SW_RESTORE, SW_SHOWNOACTIVATE, SW_SHOWNORMAL,
    },
};

use super::{
    window_obj::{WindowObjHandler, WindowObjInfo},
    window_snapshot::WindowSnapshot,
};
use crate::{
    app::structs::area::Area,
    win32::api::window::{
        focus, get_class_name, get_client_rect, get_executable_name, get_window_box, get_window_rect, get_window_style,
        get_window_title, is_window_cloaked, show_window,
    },
};

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
            exe_name: self.get_exe_name(),
            title: self.get_title(),
            viewarea: self.get_window_box(),
            class_name: self.get_class_name(),
            style: self.get_window_style(),
            visible: self.is_visible(),
            iconic: self.is_iconic(),
            cloaked: self.is_cloaked(),
        })
    }

    pub fn adjust_area(&self, area: Area) -> Area {
        let c = get_client_rect(self.hwnd);
        let w = get_window_rect(self.hwnd);
        let (c, w) = match (c, w) {
            (Some(c), Some(w)) => (c, w),
            _ => return area,
        };
        let thickness_x = ((w[2] - w[0]) - c[2]) / 2;
        let thickness_y = ((w[3] - w[1]) - c[3]) / 2;

        Area {
            x: area.x - thickness_x,
            y: area.y - thickness_y,
            width: (area.width as i32 + (thickness_x * 2)).max(0) as u16,
            height: (area.height as i32 + (thickness_y * 2)).max(0) as u16,
        }
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

impl From<WindowRef> for WindowSnapshot {
    fn from(val: WindowRef) -> Self {
        val.snapshot().unwrap()
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
        coordinates: (i32, i32),
        size: (u16, u16),
        force_normal: bool,
        async_op: bool,
        redraw: bool,
    ) -> Result<(), ()> {
        unsafe {
            let coord = (coordinates.0, coordinates.1);
            let size = (i32::from(size.0), i32::from(size.1));
            let mut flags = SWP_NOSENDCHANGING | SWP_SHOWWINDOW | SWP_NOZORDER;

            if !redraw {
                flags |= SWP_NOREDRAW;
            }

            if async_op {
                flags |= SWP_ASYNCWINDOWPOS;
            }

            if force_normal {
                show_window(self.hwnd, SW_SHOWNORMAL); // INFO: remove maximized state
            }

            match SetWindowPos(self.hwnd, None, coord.0, coord.1, size.0, size.1, flags) {
                Ok(_) => Ok(()),
                Err(_) => Err(()),
            }
        }
    }

    fn defer_resize_and_move(
        &self,
        hdwp: HDWP,
        coordinates: (i32, i32),
        size: (u16, u16),
        force_normal: bool,
        redraw: bool,
    ) -> Result<HDWP, ()> {
        unsafe {
            let coord = (coordinates.0, coordinates.1);
            let size = (i32::from(size.0), i32::from(size.1));
            let mut flags = SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOCOPYBITS | SWP_FRAMECHANGED;

            if !redraw {
                flags |= SWP_NOREDRAW;
            };

            if force_normal {
                show_window(self.hwnd, SW_SHOWNORMAL); // INFO: remove maximized state
            }
            match DeferWindowPos(hdwp, self.hwnd, None, coord.0, coord.1, size.0, size.1, flags) {
                Ok(hdwp) => Ok(hdwp),
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
}
