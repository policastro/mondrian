use crate::app::structs::area::Area;


pub trait WindowObj {
    fn get_title(&self) -> Option<String>;
    fn get_exe_name(&self) -> Option<String>;
    fn get_class_name(&self) -> Option<String>;
    fn get_window_box(&self) -> Option<Area>;
    fn focus(&self);
    fn resize_and_move(&self, coordinates: (u32, u32), size: (u32, u32));
}
