use super::tm::result::TilesManagerError;
use crate::app::area_tree::leaf::AreaLeaf;
use crate::app::configs::floating::{FloatingWinsConfig, FloatingWinsSizeStrategy};
use crate::app::structs::area::Area;
use crate::app::structs::direction::Direction;
use crate::app::structs::orientation::Orientation;
use crate::app::structs::point::Point;
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

pub fn find_nearest_candidate<K: Clone>(
    src_area: &Area,
    direction: Direction,
    candidates: &[(K, Area)],
) -> Option<(K, Area)> {
    let center = src_area.get_center();
    let edge = src_area.get_edge(direction);

    let centers_candidates = candidates.iter().map(|(k, c)| (k, c, c.get_center()));
    let nearest = centers_candidates
        .filter(|(_, _, c)| match direction {
            Direction::Left => c.0 < edge,
            Direction::Right => c.0 > edge,
            Direction::Up => c.1 < edge,
            Direction::Down => c.1 > edge,
        })
        .min_by(|(_, _, c1), (_, _, c2)| center.distance(*c1).cmp(&center.distance(*c2)));

    let nearest = nearest.or(candidates
        .iter()
        .map(|(k, c)| (k, c, c.get_center()))
        .min_by(|(_, _, c1), (_, _, c2)| center.distance(*c1).cmp(&center.distance(*c2))));

    nearest.map(|(k, a, _)| (k.clone(), *a))
}

pub fn leaves_limited_by_edge(
    windows: &[AreaLeaf<WindowRef>],
    direction: Direction,
    edge: [(i32, i32); 2],
) -> Vec<AreaLeaf<WindowRef>> {
    let dir_axis = direction.axis();
    let (lim1, lim2, axis_value) = match dir_axis {
        Orientation::Vertical => (edge[0].0, edge[1].0, edge[0].1),
        Orientation::Horizontal => (edge[0].1, edge[1].1, edge[0].0),
    };

    let mut leaves: Vec<_> = windows
        .iter()
        .filter(|l| match dir_axis {
            Orientation::Horizontal => l.viewbox.overlaps_y(lim1, lim2) && l.viewbox.contains_x(axis_value),
            Orientation::Vertical => l.viewbox.overlaps_x(lim1, lim2) && l.viewbox.contains_y(axis_value),
        })
        .copied()
        .collect();

    leaves.sort_by_key(|l| match dir_axis {
        Orientation::Horizontal => l.viewbox.y,
        Orientation::Vertical => l.viewbox.x,
    });

    leaves
}
