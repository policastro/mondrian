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
    pub(crate) area: Option<Area>,
    pub(crate) visible_area: Option<Area>,
    pub(crate) borders: Option<(i32, i32, i32, i32)>,
    pub(crate) cloaked: bool,
    pub(crate) topmost: bool,
    pub(crate) fullscreen: bool,
    pub(crate) maximized: bool,
}

impl WindowObjInfo for WindowSnapshot {
    fn get_title(&self) -> Option<String> {
        self.title.clone()
    }

    fn get_exe_name(&self) -> Option<String> {
        self.exe_name.clone()
    }

    fn get_class_name(&self) -> Option<String> {
        self.class_name.clone()
    }

    fn get_area(&self) -> Option<Area> {
        self.area
    }

    fn get_visible_area(&self) -> Option<Area> {
        self.area
    }

    fn get_borders(&self) -> Option<(i32, i32, i32, i32)> {
        self.borders
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn is_iconic(&self) -> bool {
        self.iconic
    }

    fn is_cloaked(&self) -> bool {
        self.cloaked
    }

    fn is_topmost(&self) -> bool {
        self.topmost
    }

    fn is_fullscreen(&self) -> bool {
        self.fullscreen
    }

    fn is_maximized(&self) -> bool {
        self.maximized
    }

    fn get_window_style(&self) -> u32 {
        self.style
    }
}

impl Debug for WindowSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_struct("WindowSnapshot");
        dbg.field("hwnd", &self.hwnd.0);

        if cfg!(debug_assertions) {
            dbg.field("title", &self.title.clone().unwrap_or("/".to_string()));
        }

        dbg.field("exe", &self.exe_name.clone().unwrap_or("/".to_string()))
            .field("class", &self.class_name.clone().unwrap_or("/".to_string()))
            .field("visible", &self.visible)
            .field("iconic", &self.iconic)
            .field("style", &format!("{:x}", self.style))
            .field("cloaked", &self.cloaked)
            .field("area", &self.area.map_or("/".to_string(), |a| format!("{:?}", a)))
            .field(
                "visible_area",
                &self.visible_area.map_or("/".to_string(), |a| format!("{:?}", a)),
            )
            .field("borders", &self.borders.map_or("/".to_string(), |b| format!("{:?}", b)))
            .finish()
    }
}

impl Display for WindowSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let title = default_str(self.title.as_ref());
        let exe = default_str(self.exe_name.as_ref());
        let class = default_str(self.class_name.as_ref());
        let area = self.area.map_or("/".to_string(), |a| {
            format!("({}, {}, {}, {})", a.x, a.y, a.width, a.height)
        });
        let visible = if self.visible { "v" } else { "!v" };
        let iconic = if self.iconic { "i" } else { "!i" };
        let cloaked = if self.cloaked { "c" } else { "!c" };
        let style = format!("{:x}", self.style);
        if cfg!(debug_assertions) {
            write!(
                f,
                "[{:?}] {} ({}) -> (class: {}, style: {}, [{}, {}, {}], view: {})",
                self.hwnd.0, exe, title, class, style, visible, iconic, cloaked, area
            )
        } else {
            write!(
                f,
                "[{:?}] {} -> (class: {}, style: {}, [{}, {}, {}], view: {})",
                self.hwnd.0, exe, class, style, visible, iconic, cloaked, area
            )
        }
    }
}

fn default_str<T: Debug>(val: Option<T>) -> String {
    val.map_or("/".to_string(), |v| format!("{:?}", v))
}
