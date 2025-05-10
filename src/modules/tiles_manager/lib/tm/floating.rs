use super::result::TilesManagerError;
use super::result::TilesManagerSuccess;
use super::TilesManager;
use crate::app::mondrian_message::WindowTileState;
use crate::app::structs::direction::Direction;
use crate::app::structs::orientation::Orientation;
use crate::modules::tiles_manager::lib::utils::find_nearest_candidate;
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
const CROSS_MONITOR_THRESHOLD: i32 = 15; // INFO: seems reasonable

pub trait TilesManagerFloating {
    fn move_window(&mut self, window: WindowRef, direction: Direction, step: u16) -> TMResult;
    fn insert(&mut self, window: WindowRef, direction: Direction, center: bool) -> TMResult;
    fn resize(&mut self, window: WindowRef, axis: Orientation, increment: i16) -> TMResult;
    fn change_focus(&mut self, window: WindowRef, direction: Direction) -> TMResult;
}

impl TilesManagerFloating for TilesManager {
    fn move_window(&mut self, window: WindowRef, direction: Direction, step: u16) -> TMResult {
        if !matches!(self.get_window_state(window)?, WindowTileState::Floating) {
            return Ok(Success::NoChange);
        }

        // INFO: if I don't cancel the animation, the window will have an incosistent position
        self.cancel_animation();
        let area = window.get_area().ok_or(Error::NoWindowsInfo)?;
        let monitor = match find_containing_monitor(&self.managed_monitors, area.get_center()) {
            Some(monitor) => &monitor.info,
            None => return Ok(Success::NoChange),
        };
        let monitor_area = monitor.get_workspace();

        let is_on_the_edge =
            (area.get_edge(direction) - monitor_area.get_edge(direction)).abs() <= CROSS_MONITOR_THRESHOLD;

        if is_on_the_edge {
            return self.insert(window, direction, false);
        }

        let step = step.clamp(0, i16::MAX as u16) as i16;
        let area = match direction {
            Direction::Left => area.shift((-step, 0, 0, 0)),
            Direction::Right => area.shift((step, 0, 0, 0)),
            Direction::Up => area.shift((0, -step, 0, 0)),
            Direction::Down => area.shift((0, step, 0, 0)),
        };

        let area = fit_area(&area, &monitor_area);

        Ok(Success::queue(window, area, None))
    }

    fn insert(&mut self, window: WindowRef, direction: Direction, center: bool) -> TMResult {
        if !matches!(self.get_window_state(window)?, WindowTileState::Floating) {
            return Ok(Success::NoChange);
        }

        let area = window.get_area().ok_or(Error::NoWindowsInfo)?;
        let monitor = match find_nearest_monitor(&self.managed_monitors, area.get_center(), direction) {
            Some(monitor) => &monitor.info,
            None => return Ok(Success::NoChange),
        };

        let monitor_area = monitor.get_workspace();
        let area = fit_area(&area, &monitor_area);

        if center {
            let area = area.with_center(monitor_area.get_center());
            return Ok(Success::queue(window, area, None));
        }

        Ok(Success::queue(window, area, None))
    }

    fn resize(&mut self, window: WindowRef, axis: Orientation, increment: i16) -> TMResult {
        if !matches!(self.get_window_state(window)?, WindowTileState::Floating) {
            return Ok(Success::NoChange);
        }

        // INFO: if I don't cancel the animation, the window will have an incosistent size
        self.cancel_animation();
        let area = window.get_area().ok_or(Error::NoWindowsInfo)?;
        let increment = increment.clamp(-500, 500);
        if increment < 0
            && ((matches!(axis, Orientation::Horizontal) && area.width <= MIN_FLOATING_DIM)
                || (matches!(axis, Orientation::Vertical) && area.height <= MIN_FLOATING_DIM))
        {
            return Ok(Success::NoChange);
        }

        let monitor = find_containing_monitor(&self.managed_monitors, area.get_center());
        let monitor = &monitor.ok_or(Error::Generic)?.info;
        let monitor_area = monitor.get_workspace();

        let area = match axis {
            Orientation::Horizontal => area.pad_xy((-increment, 0)),
            Orientation::Vertical => area.pad_xy((0, -increment)),
        };

        let area = fit_area(&area, &monitor_area);

        let pad_x = get_corrective_pad(MIN_FLOATING_DIM as i16, area.width as i16);
        let pad_y = get_corrective_pad(MIN_FLOATING_DIM as i16, area.height as i16);
        let area = area.pad(pad_x, pad_y);

        Ok(Success::queue(window, area, None))
    }

    fn change_focus(&mut self, window: WindowRef, direction: Direction) -> TMResult {
        if !matches!(self.get_window_state(window)?, WindowTileState::Floating) {
            return Ok(Success::NoChange);
        }

        let curr_area = window.get_area().ok_or(Error::NoWindowsInfo)?;
        let candidates: Vec<(WindowRef, _)> = self
            .floating_wins
            .enabled_keys()
            .filter(|w| *w != window)
            .filter_map(|w| w.get_area().map(|a| (w, a)))
            .collect();

        let nearest = find_nearest_candidate(&curr_area, direction, &candidates);

        if let Some(nearest) = nearest {
            nearest.0.focus();
            if self.config.focus_follows_cursor {
                let (x, y) = nearest.1.get_center();
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
    use crate::{
        app::structs::{area::Area, direction::Direction, point::Point},
        modules::tiles_manager::lib::structs::managed_monitor::ManagedMonitor,
    };
    use std::collections::HashMap;

    pub fn get_corrective_pad(min_dim: i16, dim: i16) -> (i16, i16) {
        if dim >= min_dim {
            return (0, 0);
        }
        let offset = min_dim.saturating_sub(dim);
        let new_dim = (offset as f32 / 2.0).ceil() as i16;
        (-new_dim, new_dim - offset)
    }

    pub fn find_containing_monitor(
        monitors: &HashMap<String, ManagedMonitor>,
        point: (i32, i32),
    ) -> Option<&ManagedMonitor> {
        monitors.values().find(|m| m.info.get_workspace().contains(point))
    }

    pub fn find_nearest_monitor(
        monitors: &HashMap<String, ManagedMonitor>,
        origin: (i32, i32),
        direction: Direction,
    ) -> Option<&ManagedMonitor> {
        let opp_dir = direction.opposite();
        monitors
            .values()
            .filter(|m| !m.info.get_workspace().contains(origin))
            .filter(|m| {
                let edge = m.info.get_workspace().get_edge(opp_dir);
                match direction {
                    Direction::Left => edge <= origin.0,
                    Direction::Right => edge >= origin.0,
                    Direction::Up => edge <= origin.1,
                    Direction::Down => edge >= origin.1,
                }
            })
            .min_by(|m1, m2| {
                let d1 = m1.info.get_workspace().get_center_in_direction(opp_dir);
                let d1 = d1.distance(origin);
                let d2 = m2.info.get_workspace().get_center_in_direction(opp_dir);
                let d2 = d2.distance(origin);
                d1.cmp(&d2)
            })
    }

    pub fn fit_area(src_area: &Area, dest_area: &Area) -> Area {
        let area = Area::new(
            src_area.x.max(dest_area.x),
            src_area.y.max(dest_area.y),
            src_area.width.min(dest_area.width),
            src_area.height.min(dest_area.height),
        );

        let offset_h = (dest_area.get_bottom_edge() - area.get_bottom_edge()).min(0);
        let offset_w = (dest_area.get_right_edge() - area.get_right_edge()).min(0);

        area.shift((
            offset_w.max(i16::MIN as i32) as i16,
            offset_h.max(i16::MIN as i32) as i16,
            0,
            0,
        ))
    }
}
