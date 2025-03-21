use super::tm::error::TilesManagerError;
use crate::app::configs::general::FloatingWinsConfigs;
use crate::app::configs::general::FloatingWinsSizeStrategy;
use crate::app::structs::area::Area;
use crate::win32::api::window::get_foreground_window;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;
use std::time::SystemTime;

pub(crate) fn get_foreground() -> Option<WindowRef> {
    get_foreground_window().map(WindowRef::new)
}

pub(crate) fn get_current_time_ms() -> Result<u128, TilesManagerError> {
    Ok(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|_| TilesManagerError::Generic)?
        .as_millis())
}

pub(crate) fn get_floating_win_area(
    monitor_area: &Area,
    window: &WindowRef,
    floating_wins: &FloatingWinsConfigs,
) -> Result<Area, TilesManagerError> {
    let (monitor_x, monitor_y) = monitor_area.get_center();
    let (monitor_w, monitor_h) = monitor_area.get_size();
    let area = match floating_wins.size {
        FloatingWinsSizeStrategy::Preserve => {
            let a = window.get_area().ok_or(TilesManagerError::NoWindowsInfo)?;
            let (x, y) = (monitor_x - (a.width as i32 / 2), monitor_y - (a.height as i32 / 2));
            Area::new(x, y, a.width, a.height)
        }
        FloatingWinsSizeStrategy::Fixed => {
            let (w, h) = floating_wins.size_fixed;
            let (w, h) = (w.min(monitor_w), h.min(monitor_h));
            let (x, y) = (monitor_x - (w as i32 / 2), monitor_y - (h as i32 / 2));
            Area::new(x, y, w, h)
        }
        FloatingWinsSizeStrategy::Relative => {
            let (rw, rh) = floating_wins.size_ratio;
            let (w, h) = (
                (monitor_w as f32 * rw).round() as u16,
                (monitor_h as f32 * rh).round() as u16,
            );
            let (x, y) = (monitor_x - (w as i32 / 2), monitor_y - (h as i32 / 2));
            Area::new(x, y, w, h)
        }
    };

    Ok(area)
}
