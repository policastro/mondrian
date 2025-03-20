use std::time::SystemTime;

use crate::win32::{api::window::get_foreground_window, window::window_ref::WindowRef};

use super::tm::error::TilesManagerError;

pub(crate) fn get_foreground() -> Option<WindowRef> {
    get_foreground_window().map(WindowRef::new)
}

pub(crate) fn get_current_time_ms() -> Result<u128, TilesManagerError> {
    Ok(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|_| TilesManagerError::Generic)?
        .as_millis())
}
