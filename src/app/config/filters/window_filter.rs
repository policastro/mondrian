use crate::win32::window::window_obj::WindowObjInfo;

pub trait WindowFilter {
    fn filter(&self, window: &impl WindowObjInfo) -> bool;
}
