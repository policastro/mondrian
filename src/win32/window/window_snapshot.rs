use std::fmt::{Debug, Display};

use windows::Win32::Foundation::HWND;

use crate::app::structs::area::Area;

use super::window_obj::WindowObjInfo;

#[derive(Clone)]
pub struct WindowSnapshot {
    pub hwnd: HWND,
    pub(crate) title: Option<String>,
    pub(crate) exe_name: Option<String>,
    pub(crate) class_name: Option<String>,
    pub(crate) style: u32,
    pub(crate) iconic: bool,
    pub(crate) visible: bool,
    pub(crate) viewarea: Option<Area>,
    pub(crate) cloaked: bool,
}

impl WindowObjInfo for WindowSnapshot {
    fn get_title(&self) -> Option<String> {
        self.title.clone()
    }

    fn get_exe_name(&self) -> Option<String> {
        self.exe_name.clone()
    }

    fn get_window_box(&self) -> Option<Area> {
        self.viewarea
    }

    fn get_class_name(&self) -> Option<String> {
        self.class_name.clone()
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn is_iconic(&self) -> bool {
        self.iconic
    }

    fn get_window_style(&self) -> u32 {
        self.style
    }

    fn is_cloaked(&self) -> bool {
        self.cloaked
    }
}

impl Debug for WindowSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowSnapshot")
            .field("hwnd", &self.hwnd.0)
            .field("title", &self.title.clone().unwrap_or("/".to_string()))
            .field("exe", &self.exe_name.clone().unwrap_or("/".to_string()))
            .field("class", &self.class_name.clone().unwrap_or("/".to_string()))
            .field("visible", &self.visible)
            .field("iconic", &self.iconic)
            .field("style", &self.style)
            .field("cloaked", &self.cloaked)
            .field(
                "viewarea",
                &self.viewarea.clone().map_or("/".to_string(), |a| format!("{:?}", a)),
            )
            .finish()
    }
}

impl Display for WindowSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let title = default_str(self.title.as_ref());
        let exe = default_str(self.exe_name.as_ref());
        let class = default_str(self.class_name.as_ref());
        let area = self.viewarea.clone().map_or("/".to_string(), |a| {
            format!("({}, {}, {}, {})", a.x, a.y, a.width, a.height)
        });
        let visible = if self.visible { "v" } else { "!v" };
        let iconic = if self.iconic { "i" } else { "!i" };
        let cloaked = if self.cloaked { "c" } else { "!c" };
        let style = format!("0x{:x}", self.style);
        write!(
            f,
            "[{:?}] {} ({}) -> (class: {}, style: {}, [{}, {}, {}], view: {})",
            self.hwnd.0, exe, title, class, style, visible, iconic, cloaked, area
        )
    }
}

fn default_str<T: Debug>(val: Option<T>) -> String {
    val.map_or("/".to_string(), |v| format!("{:?}", v))
}
