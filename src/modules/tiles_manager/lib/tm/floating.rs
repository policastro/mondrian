use super::manager::TilesManager;
use super::result::TilesManagerError;
use super::result::TilesManagerSuccess;
use crate::app::mondrian_message::WindowTileState;
use crate::app::structs::direction::Direction;
use crate::app::structs::orientation::Orientation;
use crate::app::structs::point::Point;
use crate::win32::api::cursor::set_cursor_pos;
use crate::win32::window::window_obj::WindowObjHandler;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;
use std::collections::HashMap;
use utils::*;

type Success = TilesManagerSuccess;
type Error = TilesManagerError;
type TMResult = Result<Success, Error>;

const MIN_FLOATING_DIM: u16 = 250; // INFO: seems reasonable
const CROSS_MONITOR_THRESHOLD: i16 = 15; // INFO: seems reasonable

pub trait TilesManagerFloating {
    fn move_window(&mut self, window: WindowRef, direction: Direction, step: u16) -> TMResult;
    fn resize(&mut self, window: WindowRef, axis: Orientation, increment: i16) -> TMResult;
    fn change_focus(&mut self, window: WindowRef, direction: Direction, center_cursor: bool) -> TMResult;
}

impl TilesManagerFloating for TilesManager {
    fn move_window(&mut self, window: WindowRef, direction: Direction, step: u16) -> TMResult {
        if !matches!(self.get_window_state(window)?, WindowTileState::Floating) {
            return Ok(Success::NoChange);
        }

        let area = window.get_area().ok_or(Error::NoWindowsInfo)?;
        let point_to_check = area
            .pad_full(-CROSS_MONITOR_THRESHOLD)
            .get_center_in_direction(direction);

        let src_monitor = find_containing_monitor(&self.managed_monitors, area.get_center());
        let trg_monitor = find_containing_monitor(&self.managed_monitors, point_to_check);
        let monitor = match (src_monitor, trg_monitor) {
            (Some(src), None) => src,
            (None, Some(trg)) => trg,
            (Some(src), Some(trg)) => match src.id == trg.id {
                true => src,
                false => trg,
            },
            _ => return Ok(Success::NoChange),
        };
        let monitor_area = monitor.get_area();

        let step = step.clamp(0, i16::MAX as u16) as i16;
        let mut area = match direction {
            Direction::Left => area.shift((-step, 0, 0, 0)),
            Direction::Right => area.shift((step, 0, 0, 0)),
            Direction::Up => area.shift((0, -step, 0, 0)),
            Direction::Down => area.shift((0, step, 0, 0)),
        };

        area.x = area.x.max(monitor_area.x);
        area.y = area.y.max(monitor_area.y);
        area = area.shift((
            0.min(monitor_area.get_right_edge() - area.get_right_edge()) as i16,
            0.min(monitor_area.get_bottom_edge() - area.get_bottom_edge()) as i16,
            0,
            0,
        ));

        let area = area.clamp(&monitor_area);

        Ok(Success::queue(window, area, None))
    }

    fn resize(&mut self, window: WindowRef, axis: Orientation, increment: i16) -> TMResult {
        if !matches!(self.get_window_state(window)?, WindowTileState::Floating) {
            return Ok(Success::NoChange);
        }

        let area = window.get_area().ok_or(Error::NoWindowsInfo)?;
        let increment = increment.clamp(-500, 500);
        if increment < 0
            && ((matches!(axis, Orientation::Horizontal) && area.width <= MIN_FLOATING_DIM)
                || (matches!(axis, Orientation::Vertical) && area.height <= MIN_FLOATING_DIM))
        {
            return Ok(Success::NoChange);
        }

        let monitor = find_containing_monitor(&self.managed_monitors, area.get_center());
        let monitor = monitor.ok_or(Error::Generic)?;
        let monitor_area = monitor.get_area();

        let area = match axis {
            Orientation::Horizontal => area.pad_xy((-increment, 0)),
            Orientation::Vertical => area.pad_xy((0, -increment)),
        }
        .clamp(&monitor_area);

        let pad_x = get_corrective_pad(MIN_FLOATING_DIM as i16, area.width as i16);
        let pad_y = get_corrective_pad(MIN_FLOATING_DIM as i16, area.height as i16);
        let area = area.pad(pad_x, pad_y);

        Ok(Success::queue(window, area, None))
    }

    fn change_focus(&mut self, window: WindowRef, direction: Direction, center_cursor: bool) -> TMResult {
        if !matches!(self.get_window_state(window)?, WindowTileState::Floating) {
            return Ok(Success::NoChange);
        }

        let curr_area = window.get_area().ok_or(Error::NoWindowsInfo)?;
        let curr_center = curr_area.get_center();
        let curr_edge = curr_area.get_edge(direction);
        let floatings_centers: Vec<(WindowRef, (i32, i32))> = self
            .floating_wins
            .enabled_keys()
            .filter(|w| *w != window)
            .filter_map(|w| w.get_area().map(|a| (w, a.get_center())))
            .collect();

        let nearest = floatings_centers
            .iter()
            .filter(|f| match direction {
                Direction::Left => f.1 .0 < curr_edge,
                Direction::Right => f.1 .0 > curr_edge,
                Direction::Up => f.1 .1 < curr_edge,
                Direction::Down => f.1 .1 > curr_edge,
            })
            .min_by(|a, b| curr_center.distance(a.1).cmp(&curr_center.distance(b.1)));

        let nearest = match nearest {
            Some(n) => Some(n),
            None => floatings_centers
                .iter()
                .min_by(|a, b| curr_center.distance(a.1).cmp(&curr_center.distance(b.1))),
        };

        if let Some(nearest) = nearest {
            nearest.0.focus();
            if center_cursor {
                let (x, y) = nearest.1;
                set_cursor_pos(x, y);
            }
        }

        Ok(Success::NoChange)
    }
}

#[derive(Debug)]
pub struct FloatingProperties {
    pub minimized: bool,
    pub locked: bool,
}

impl FloatingProperties {
    pub fn new() -> Self {
        FloatingProperties {
            minimized: false,
            locked: false,
        }
    }
}

pub trait FloatingWindows {
    fn enabled_keys(&self) -> impl Iterator<Item = WindowRef>;
    fn locked(&self, window: &WindowRef) -> Option<bool>;
    fn set_properties(&mut self, window: &WindowRef, minimized: bool, locked: bool);
    fn set_all_locked(&mut self, locked: bool);
    fn set_locked(&mut self, window: &WindowRef, locked: bool);
    fn set_minimized(&mut self, window: &WindowRef, minimized: bool);
    fn can_be_closed(&self, window: &WindowRef) -> bool;
}

impl FloatingWindows for HashMap<WindowRef, FloatingProperties> {
    fn enabled_keys(&self) -> impl Iterator<Item = WindowRef> {
        self.iter().filter(|(_, props)| !props.minimized).map(|(key, _)| *key)
    }

    fn set_all_locked(&mut self, locked: bool) {
        for (_, props) in self.iter_mut() {
            props.locked = locked;
        }
    }

    fn set_locked(&mut self, window: &WindowRef, locked: bool) {
        if let Some(props) = self.get_mut(window) {
            props.locked = locked;
        }
    }

    fn set_minimized(&mut self, window: &WindowRef, minimized: bool) {
        if let Some(props) = self.get_mut(window) {
            props.minimized = minimized;
        }
    }

    fn set_properties(&mut self, window: &WindowRef, minimized: bool, locked: bool) {
        if let Some(props) = self.get_mut(window) {
            props.minimized = minimized;
            props.locked = locked;
        }
    }

    fn can_be_closed(&self, window: &WindowRef) -> bool {
        self.get(window)
            .map(|props| !props.minimized && !props.locked)
            .unwrap_or(true)
    }

    fn locked(&self, window: &WindowRef) -> Option<bool> {
        self.get(window).map(|props| props.locked)
    }
}

mod utils {
    use std::collections::HashMap;

    use crate::win32::api::monitor::Monitor;

    pub fn get_corrective_pad(min_dim: i16, dim: i16) -> (i16, i16) {
        if dim >= min_dim {
            return (0, 0);
        }
        let offset = min_dim.saturating_sub(dim);
        let new_dim = (offset as f32 / 2.0).ceil() as i16;
        (-new_dim, new_dim - offset)
    }

    pub fn find_containing_monitor(monitors: &HashMap<String, Monitor>, point: (i32, i32)) -> Option<&Monitor> {
        monitors.values().find(|m| m.get_area().contains(point))
    }
}
