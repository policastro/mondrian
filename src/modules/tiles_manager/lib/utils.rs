use crate::win32::{api::window::get_foreground_window, window::window_ref::WindowRef};

pub fn get_foreground() -> Option<WindowRef> {
    get_foreground_window().map(WindowRef::new)
}
