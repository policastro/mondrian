use windows::Win32::UI::WindowsAndMessaging::SET_WINDOW_POS_FLAGS;

use crate::app::structs::area::Area;

pub trait WindowObjInfo {
    fn get_title(&self) -> Option<String>;
    fn get_exe_name(&self) -> Option<String>;
    fn get_class_name(&self) -> Option<String>;
    fn get_area(&self) -> Option<Area>;
    fn get_visible_area(&self) -> Option<Area>;
    fn get_borders(&self) -> Option<(i32, i32, i32, i32)>;
    fn is_visible(&self) -> bool;
    fn is_iconic(&self) -> bool;
    fn is_cloaked(&self) -> bool;
    fn get_window_style(&self) -> u32;
}

pub trait WindowObjHandler {
    fn focus(&self);
    fn resize_and_move(
        &self,
        pos: (i32, i32),
        size: (u16, u16),
        force_normal: bool,
        flags: SET_WINDOW_POS_FLAGS,
    ) -> Result<(), ()>;
    fn redraw(&self) -> Result<(), ()>;
    fn minimize(&self) -> bool;
    fn restore(&self, activate: bool) -> bool;
    fn set_topmost(&self, topmost: bool) -> Result<(), ()>;
}
