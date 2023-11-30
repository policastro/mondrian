use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::SetForegroundWindow};

use crate::app::structs::area::Area;

use super::window_obj::WindowObj;

#[derive(Debug, Clone)]
pub struct WindowSnapshot {
    pub(crate) hwnd: HWND,
    pub(crate) title: String,
    pub(crate) exe_name: String,
    pub viewarea: Area,
    pub(crate) class_name: String,
}

impl WindowObj for WindowSnapshot {
    fn get_title(&self) -> Option<String> {
        Some(self.title.clone())
    }

    fn get_exe_name(&self) -> Option<String> {
        Some(self.exe_name.clone())
    }

    fn get_window_box(&self) -> Option<Area> {
        Some(self.viewarea.clone())
    }

    fn focus(&self) {
        unsafe {
            let _ = SetForegroundWindow(self.hwnd);
        };
    }

    fn resize_and_move(&self, _coordinates: (u32, u32), _sizee: (u32, u32)) {
        // TODO
    }

    fn get_class_name(&self) -> Option<String> {
        Some(self.class_name.clone())
    }
}
