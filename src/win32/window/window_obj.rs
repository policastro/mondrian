use crate::app::structs::area::Area;

pub trait WindowObjInfo {
    fn get_title(&self) -> Option<String>;
    fn get_exe_name(&self) -> Option<String>;
    fn get_class_name(&self) -> Option<String>;
    fn get_window_box(&self) -> Option<Area>;
    fn is_visible(&self) -> bool;
    fn is_iconic(&self) -> bool;
    fn is_cloaked(&self) -> bool;
    fn get_window_style(&self) -> u32;
}

pub trait WindowObjHandler {
    fn focus(&self);
    fn resize_and_move(
        &self,
        coordinates: (i32, i32),
        size: (u16, u16),
        async_op: bool,
        redraw: bool,
    ) -> Result<(), ()>;
    fn redraw(&self) -> Result<(), ()>;
    fn minimize(&self) -> bool;
    fn restore(&self) -> bool;
}
