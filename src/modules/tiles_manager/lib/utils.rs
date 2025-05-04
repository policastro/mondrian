use super::tm::result::TilesManagerError;
use crate::app::configs::floating::{FloatingWinsConfig, FloatingWinsSizeStrategy};
use crate::app::structs::area::Area;
use crate::win32::api::window::get_foreground_window;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;
use winvd::is_window_on_current_desktop;

pub(crate) fn is_on_current_vd(window: &WindowRef) -> Result<bool, TilesManagerError> {
    is_window_on_current_desktop((*window).into()).map_err(TilesManagerError::VDError)
}

pub(crate) fn get_foreground() -> Option<WindowRef> {
    get_foreground_window().map(WindowRef::new)
}

pub(crate) fn get_floating_win_area(
    monitor_area: &Area,
    window: &WindowRef,
    config: &FloatingWinsConfig,
) -> Result<Area, TilesManagerError> {
    let (monitor_x, monitor_y) = monitor_area.get_center();
    let (monitor_w, monitor_h) = monitor_area.get_size();
    let a = window.get_area().ok_or(TilesManagerError::NoWindowsInfo)?;
    let new_area = match config.strategy {
        FloatingWinsSizeStrategy::Preserve => {
            let (x, y) = match config.centered {
                true => (monitor_x - (a.width as i32 / 2), monitor_y - (a.height as i32 / 2)),
                false => a.get_origin(),
            };
            Area::new(x, y, a.width, a.height)
        }
        FloatingWinsSizeStrategy::Fixed { w, h } => {
            let (w, h) = (w.min(monitor_w), h.min(monitor_h));
            let (x, y) = match config.centered {
                true => (monitor_x - (w as i32 / 2), monitor_y - (h as i32 / 2)),
                false => a.get_origin(),
            };
            Area::new(x, y, w, h)
        }
        FloatingWinsSizeStrategy::Relative { w: rw, h: rh } => {
            let (w, h) = (
                (monitor_w as f32 * rw).round() as u16,
                (monitor_h as f32 * rh).round() as u16,
            );
            let (x, y) = match config.centered {
                true => (monitor_x - (w as i32 / 2), monitor_y - (h as i32 / 2)),
                false => a.get_origin(),
            };
            Area::new(x, y, w, h)
        }
    };

    Ok(new_area)
}
