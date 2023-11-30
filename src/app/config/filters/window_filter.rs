use crate::win32utils::window::window_obj::WindowObj;

pub trait WindowFilter {
    fn filter(&self, window: &impl WindowObj) -> bool;
}
