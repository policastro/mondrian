use crate::{
    app::{
        mondrian_message::WindowTileState,
        structs::{direction::Direction, orientation::Orientation, point::Point},
    },
    win32::{
        api::{cursor::set_cursor_pos, monitor::Monitor},
        window::{
            window_obj::{WindowObjHandler, WindowObjInfo},
            window_ref::WindowRef,
        },
    },
};

use super::{
    manager::TilesManager,
    result::{TilesManagerError, TilesManagerSuccess},
};

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
        let monitor_area = match (src_monitor, trg_monitor) {
            (Some(src), None) => src,
            (None, Some(trg)) => trg,
            (Some(src), Some(trg)) => match src.id == trg.id {
                true => src,
                false => trg,
            },
            _ => return Ok(Success::NoChange),
        }
        .workspace_area;

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
        let monitor = find_containing_monitor(&self.managed_monitors, area.get_center()).ok_or(Error::Generic)?;
        let monitor_area = monitor.workspace_area;

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
        let floatings_centers = self
            .floating_wins
            .iter()
            .filter(|w| **w != window)
            .filter_map(|w| w.get_area().map(|a| (w, a.get_center())));

        let nearest = floatings_centers
            .clone()
            .filter(|f| match direction {
                Direction::Left => f.1 .0 < curr_edge,
                Direction::Right => f.1 .0 > curr_edge,
                Direction::Up => f.1 .1 < curr_edge,
                Direction::Down => f.1 .1 > curr_edge,
            })
            .min_by(|a, b| curr_center.distance(a.1).cmp(&curr_center.distance(b.1)));

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

fn get_corrective_pad(min_dim: i16, dim: i16) -> (i16, i16) {
    if dim >= min_dim {
        return (0, 0);
    }
    let offset = min_dim.saturating_sub(dim);
    let new_dim = (offset as f32 / 2.0).ceil() as i16;
    (-new_dim, new_dim - offset)
}

fn find_containing_monitor(monitors: &[Monitor], point: (i32, i32)) -> Option<&Monitor> {
    monitors.iter().find(|m| m.workspace_area.contains(point))
}
