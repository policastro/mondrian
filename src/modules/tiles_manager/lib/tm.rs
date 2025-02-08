use super::containers::{Container, ContainerLayer, Containers};
use super::monitor_layout::MonitorLayout;
use super::tm_configs::TilesManagerConfig;
use super::window_animation_player::WindowAnimationPlayer;
use crate::app::area_tree::leaf::AreaLeaf;
use crate::app::area_tree::tree::WinTree;
use crate::app::mondrian_message::{IntermonitorMoveOp, IntramonitorMoveOp, WindowTileState};
use crate::app::structs::direction::Direction;
use crate::app::structs::point::Point;
use crate::win32::api::window::get_foreground_window;
use crate::win32::window::window_obj::{WindowObjHandler, WindowObjInfo};
use crate::win32::window::window_ref::WindowRef;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Duration;
use windows::Win32::Foundation::HWND;

type IntraOp = IntramonitorMoveOp;
type InterOp = IntermonitorMoveOp;
type Error = TilesManagerError;

pub struct TilesManager {
    containers: HashMap<isize, Container<String>>,
    floating_wins: HashSet<isize>,
    maximized_wins: HashSet<isize>,
    config: TilesManagerConfig,
    animation_player: WindowAnimationPlayer,
}

impl TilesManager {
    /// Creates a new [`TilesManager`].
    pub fn new<S, E, C>(
        monitors_layout: Vec<MonitorLayout>,
        config: Option<TilesManagerConfig>,
        on_update_start: S,
        on_update_error: E,
        on_update_complete: C,
    ) -> Self
    where
        S: Fn() + Sync + Send + 'static,
        E: Fn() + Sync + Send + 'static,
        C: Fn() + Sync + Send + 'static,
    {
        let config = config.unwrap_or_default();
        let containers = monitors_layout
            .into_iter()
            .map(|m| (m.monitor.id, m.monitor.into(), m.layout))
            .map(|(id, w, l)| {
                let mut c = Container::<String>::new();
                c.add(ContainerType::Normal(0).into(), WinTree::new(w, l.clone()));
                c.add(ContainerType::Focalized.into(), WinTree::new(w, l));
                let _ = c.set_active(ContainerType::Normal(0).into());
                (id, c)
            })
            .collect();

        let animation_duration = Duration::from_millis(config.get_animation_duration().into());
        let animation_player = WindowAnimationPlayer::new(
            animation_duration,
            config.get_framerate(),
            on_update_start,
            on_update_error,
            on_update_complete,
        );
        TilesManager {
            floating_wins: HashSet::new(),
            maximized_wins: HashSet::new(),
            containers,
            config,
            animation_player,
        }
    }

    pub fn add(&mut self, win: WindowRef, update: bool) -> Result<(), Error> {
        let tile_state = self.get_window_tile_state(win.hwnd);
        if tile_state.is_some_and(|s| matches!(s, WindowTileState::Floating | WindowTileState::Ignored)) {
            return Ok(());
        }

        if let Some(c) = self.containers.find_mut(win.hwnd.0, false) {
            match c.is_focalized() && !c.has(win.hwnd.0) {
                true => c.unfocalize(),
                false => return Err(Error::WindowAlreadyAdded),
            }
        }

        let center = win.get_window_box().map(|a| a.get_center());
        let center = center.ok_or(Error::NoWindowsInfo)?;
        let c = self.containers.find_at_or_near_mut(center);
        let c = c.ok_or(Error::NoWindowsInfo)?;
        c.unfocalize();
        c.get_active_mut().ok_or(Error::NoWindowsInfo)?.insert(win.hwnd.0);

        // INFO: if the monitor has a maximized window, restore it
        if let Some(maximized_win) = self.maximized_wins.iter().find(|w| c.has(**w)) {
            self.on_maximize(Some(HWND(*maximized_win)), false)?;
        }

        self.update_if(update);
        Ok(())
    }

    pub fn remove(&mut self, hwnd: HWND, skip_focalized: bool, update: bool) -> Result<(), Error> {
        let tile_state = self.get_window_tile_state(hwnd).ok_or(Error::NoWindow)?;
        if matches!(tile_state, WindowTileState::Floating) {
            return Ok(());
        }

        let c = self.containers.find_mut(hwnd.0, false);
        if skip_focalized && c.is_some_and(|c| c.is_focalized()) {
            return Ok(());
        }

        let c = self.containers.find_mut(hwnd.0, false).ok_or(Error::NoWindow)?;
        c.iter_mut().for_each(|(_, t)| t.remove(hwnd.0));

        if matches!(tile_state, WindowTileState::Ignored) {
            self.maximized_wins.remove(&hwnd.0);
        }

        if get_foreground_window().is_none() {
            self.focus_next();
        }

        self.update_if(update);
        Ok(())
    }

    fn find_at(&self, direction: Direction, same_monitor: bool) -> Option<AreaLeaf<isize>> {
        let current = get_foreground_window()?;
        let src_c = self.containers.find(current.0, true)?;
        let src_area = src_c.get_active()?.find_leaf(current.0, 0)?.viewbox;
        let point = match direction {
            Direction::Right => src_area.get_ne_corner().with_offset(1, 1), // INFO: prefer up
            Direction::Down => src_area.get_se_corner().with_offset(-1, 1), // INFO: prefer right
            Direction::Left => src_area.get_sw_corner().with_offset(-1, -1), // INFO: prefer down
            Direction::Up => src_area.get_nw_corner().with_offset(1, -1),   // INFO: prefer left
        };

        let params = if let Some(c) = self.containers.find_at(point) {
            // If the point is in a tree
            Some((c, point))
        } else if let Some(c) = self.containers.find_closest_at(src_area.get_center(), direction) {
            // Otherwise, find the nearest container
            let area = c.get_active()?.area;
            let point = match direction {
                Direction::Right => area.get_nw_corner(),
                Direction::Down => area.get_ne_corner(),
                Direction::Left => area.get_se_corner(),
                Direction::Up => area.get_sw_corner(),
            };
            Some((c as &Container<String>, point))
        } else {
            None
        };

        let t = match same_monitor {
            true => src_c.get_active()?,
            false => params?.0.get_active()?,
        };

        t.find_leaf_at(params?.1, 0).filter(|w| w.id != current.0)
    }

    pub fn focus_at(&mut self, direction: Direction) -> Result<(), Error> {
        let leaf = self.find_at(direction, false).ok_or(Error::NoWindow)?;
        WindowRef::new(HWND(leaf.id)).focus();
        Ok(())
    }

    pub fn on_move(
        &mut self,
        hwnd: HWND,
        target: (i32, i32),
        intra_op: IntraOp,
        inter_op: InterOp,
    ) -> Result<(), Error> {
        let tile_state = self.get_window_tile_state(hwnd).ok_or(Error::NoWindow)?;
        if matches!(tile_state, WindowTileState::Floating | WindowTileState::Ignored) {
            return Ok(());
        }

        const C_ERR: Error = Error::ContainerNotFound { refresh: true };

        let cs = &mut self.containers;
        let src_leaf = cs.find(hwnd.0, true).and_then(|c| c.get_active());
        let src_leaf = src_leaf.and_then(|t| t.find_leaf(hwnd.0, 0)).ok_or(Error::Generic)?;
        let trg_leaf = cs.find_at(target).and_then(|c| c.get_active());
        let trg_leaf = trg_leaf.and_then(|t| t.find_leaf_at(target, 0));

        let is_same_monitor = cs.is_same_container(src_leaf.viewbox.get_center(), target);
        if is_same_monitor {
            // If it is in the same monitor
            let c = cs.find_mut(src_leaf.id, true).ok_or(C_ERR)?;
            if matches!(intra_op, IntraOp::InsertFreeMove) && !c.is_focalized() {
                c.get_active_mut().ok_or(C_ERR)?.move_to(src_leaf.id, target);
            } else if let Some(leaf) = trg_leaf {
                c.iter_mut().for_each(|(_, t)| t.swap_ids(src_leaf.id, leaf.id));
            }
        } else if matches!(inter_op, InterOp::Insert | InterOp::Invert | InterOp::InsertFreeMove) || trg_leaf.is_none()
        {
            // If it is in another monitor and insert
            let c = cs.find_mut(src_leaf.id, true).ok_or(C_ERR)?;
            c.unfocalize();
            c.get_active_mut().ok_or(C_ERR)?.remove(src_leaf.id);

            let c = cs.find_at_mut(target).ok_or(C_ERR)?;
            c.unfocalize();

            match matches!(inter_op, InterOp::InsertFreeMove) {
                true => c.get_active_mut().ok_or(C_ERR)?.insert_at(src_leaf.id, target),
                false => c.get_active_mut().ok_or(C_ERR)?.insert(src_leaf.id),
            }
        } else {
            // If it is in another monitor and swap
            let src_trees = cs.find_mut(src_leaf.id, true).ok_or(C_ERR)?;
            match trg_leaf {
                Some(leaf) => src_trees.iter_mut().for_each(|(_, t)| {
                    t.replace_id(src_leaf.id, leaf.id);
                }),
                None => src_trees.iter_mut().for_each(|(_, t)| t.remove(src_leaf.id)),
            };

            let trg_trees = cs.find_at_mut(target).ok_or(C_ERR)?;
            match trg_leaf {
                Some(trg) => trg_trees.iter_mut().for_each(|(_, t)| {
                    t.replace_id(trg.id, src_leaf.id);
                }),
                None => trg_trees.get_active_mut().ok_or(C_ERR)?.insert(src_leaf.id),
            };
        };

        let switch_orient = match is_same_monitor {
            true => matches!(intra_op, IntramonitorMoveOp::Invert),
            false => matches!(inter_op, InterOp::Invert),
        };

        if switch_orient {
            let tree = cs.find_at_mut(target).and_then(|c| c.get_active_mut());
            tree.ok_or(C_ERR)?.switch_subtree_orientations(target);
        }

        self.update(true);
        Ok(())
    }

    pub(crate) fn move_focused(&mut self, direction: Direction) -> Result<(), Error> {
        let curr = get_foreground_window().ok_or(Error::NoWindow)?;
        let c = self.containers.find(curr.0, true).and_then(|c| c.get_active());
        let src_leaf = c.and_then(|t| t.find_leaf(curr.0, 0)).ok_or(Error::Generic)?;
        let trg_leaf = self.find_at(direction, false).ok_or(Error::NoWindow)?;

        let cs = &mut self.containers;
        if cs.is_same_container(src_leaf.viewbox.get_center(), trg_leaf.viewbox.get_center()) {
            let c = cs.find_mut(src_leaf.id, true).ok_or(Error::Generic)?;
            c.iter_mut().for_each(|(_, t)| t.swap_ids(src_leaf.id, trg_leaf.id));
        } else {
            let trees = cs.find_mut(trg_leaf.id, true).ok_or(Error::Generic)?;
            trees.iter_mut().for_each(|(_, t)| {
                t.replace_id(trg_leaf.id, src_leaf.id);
            });

            let trees = cs.find_at_mut(src_leaf.viewbox.get_center()).ok_or(Error::Generic)?;
            trees.iter_mut().for_each(|(_, t)| {
                t.replace_id(src_leaf.id, trg_leaf.id);
            });
        };

        self.update(true);
        Ok(())
    }

    pub(crate) fn insert_focused(&mut self, direction: Direction) -> Result<(), Error> {
        const C_ERR: Error = Error::ContainerNotFound { refresh: false };
        let curr = get_foreground_window().ok_or(Error::NoWindow)?;

        let src_c_key = self.containers.find_key(curr.0, true).ok_or(C_ERR)?;

        let src_leaf = self
            .containers
            .get(&src_c_key)
            .and_then(|c| c.get_active())
            .and_then(|t| t.find_leaf(curr.0, 0))
            .ok_or(C_ERR)?;

        self.containers
            .find_closest_at_mut(src_leaf.viewbox.get_center(), direction)
            .and_then(|c| c.get_active_mut())
            .ok_or(C_ERR)?
            .insert(src_leaf.id);

        self.containers
            .get_mut(&src_c_key)
            .ok_or(C_ERR)?
            .iter_mut()
            .for_each(|(_, t)| t.remove(src_leaf.id));

        self.update(true);
        Ok(())
    }

    pub(crate) fn resize_focused(&mut self, direction: Direction, size: u8) -> Result<(), Error> {
        let curr = get_foreground_window().ok_or(Error::NoWindow)?;
        if !self.has_window(curr) {
            return Err(Error::NoWindow);
        }

        let orig_area = WindowRef::new(curr).get_window_box().ok_or(Error::NoWindowsInfo)?;
        let size = size as i16;
        let has_neigh1 = self.find_at(direction, true).is_some();
        let has_neigh2 = self.find_at(direction.opposite(), true).is_some();

        let get_pad = |neigh1: bool, neigh2: bool, v1: (i16, i16), v2: (i16, i16)| match (neigh1, neigh2) {
            (true, _) => v1,
            (false, true) => v2,
            _ => (0, 0),
        };
        let padding = match direction {
            Direction::Left => (get_pad(has_neigh1, has_neigh2, (size, 0), (0, -size)), (0, 0)),
            Direction::Right => (get_pad(has_neigh1, has_neigh2, (0, size), (-size, 0)), (0, 0)),
            Direction::Up => ((0, 0), get_pad(has_neigh1, has_neigh2, (size, 0), (0, -size))),
            Direction::Down => ((0, 0), get_pad(has_neigh1, has_neigh2, (0, size), (-size, 0))),
        };

        let area = orig_area.pad(Some(padding.0), Some(padding.1));
        self.on_resize(curr, orig_area.get_shift(&area), true)
    }

    pub(crate) fn invert_orientation(&mut self) -> Result<(), Error> {
        let curr = get_foreground_window().ok_or(Error::NoWindow)?;
        let c = self.containers.find_mut(curr.0, true).and_then(|c| c.get_active_mut());
        let center = WindowRef::new(curr).get_window_box().map(|a| a.get_center());
        let center = center.ok_or(Error::NoWindowsInfo)?;
        c.ok_or(Error::Generic)?.switch_subtree_orientations(center);

        self.update(true);
        Ok(())
    }

    pub(crate) fn on_resize(&mut self, hwnd: HWND, delta: (i32, i32, i32, i32), animate: bool) -> Result<(), Error> {
        let tile_state = self.get_window_tile_state(hwnd).ok_or(Error::NoWindow)?;
        if matches!(tile_state, WindowTileState::Floating | WindowTileState::Ignored) {
            return Ok(());
        }

        let (resize_w, resize_h) = (delta.2 != 0, delta.3 != 0);
        let (resize_left, resize_up) = (delta.0.abs() > 10, delta.1.abs() > 10);
        let has_w_neigh = match resize_w {
            true => self.find_at(if resize_left { Direction::Left } else { Direction::Right }, true),
            false => None,
        };
        let has_h_neigh = match resize_h {
            true => self.find_at(if resize_up { Direction::Up } else { Direction::Down }, true),
            false => None,
        };

        let c = self.containers.find_mut(hwnd.0, true).ok_or(Error::NoWindow)?;
        let t = c.get_active_mut().ok_or(Error::NoWindow)?;
        let area = t.find_leaf(hwnd.0, 0).ok_or(Error::Generic)?.viewbox;
        let center = area.get_center();

        let clamp_values = Some((10, 90));
        let padding = self.config.get_tile_pad_xy();
        if resize_w && has_w_neigh.is_some() {
            let growth = (delta.2.saturating_add(padding.0.into()) as f32 / area.width as f32) * 100f32;
            let (x, growth_perc) = match resize_left {
                true => (area.get_left_center().0.saturating_sub(20), -growth),
                false => (area.get_right_center().0.saturating_add(20), growth),
            };
            t.resize_ancestor(center, (x, center.1), growth_perc, clamp_values);
        }

        if resize_h && has_h_neigh.is_some() {
            let growth = (delta.3 as f32 / area.height as f32) * 100f32;
            let (y, growth_perc) = match resize_up {
                true => (area.get_top_center().1.saturating_sub(20), -growth),
                false => (area.get_bottom_center().1.saturating_add(20), growth),
            };
            t.resize_ancestor(center, (center.0, y), growth_perc, clamp_values);
        }

        self.update(animate);
        Ok(())
    }

    pub fn minimize_focused(&mut self) -> Result<(), Error> {
        let win_ref = WindowRef::new(get_foreground_window().ok_or(Error::NoWindow)?);
        win_ref.minimize();
        self.focus_next();
        Ok(())
    }

    pub(crate) fn focalize_focused(&mut self) -> Result<(), Error> {
        let hwnd = get_foreground_window().ok_or(Error::NoWindow)?;
        let area = WindowRef::new(hwnd).get_window_box();
        let center = area.ok_or(Error::NoWindowsInfo)?.get_center();
        let c = self.containers.find_at_mut(center).ok_or(Error::NoWindow)?;

        if c.is_focalized() && c.get_active_mut().ok_or(Error::Generic)?.has(hwnd.0) {
            c.unfocalize();
        } else {
            let wins = c.get_active().ok_or(Error::Generic)?.get_ids();
            let wins = wins.iter().filter(|h| **h != hwnd.0).map(|h| WindowRef::new(HWND(*h)));
            wins.for_each(|w| {
                w.minimize();
            });
            c.focalize(hwnd.0);
            let _ = self.release(Some(false), Some(hwnd));
        }

        self.update(true);
        Ok(())
    }

    pub(crate) fn release(&mut self, release: Option<bool>, window: Option<HWND>) -> Result<(), Error> {
        let hwnd = window.or_else(get_foreground_window).ok_or(Error::NoWindow)?;
        let tile_state = self.get_window_tile_state(hwnd).ok_or(Error::NoWindow)?;

        if matches!(tile_state, WindowTileState::Ignored) {
            return Ok(());
        }

        if release.unwrap_or(!matches!(tile_state, WindowTileState::Floating)) {
            self.remove(hwnd, false, false)?;
            self.floating_wins.insert(hwnd.0);
            let _ = WindowRef::new(hwnd).set_topmost(true);
        } else {
            self.floating_wins.remove(&hwnd.0);
            self.add(WindowRef::new(hwnd), false)?;
            let _ = WindowRef::new(hwnd).set_topmost(false);
        }

        self.update(true);
        Ok(())
    }

    pub(crate) fn on_maximize(&mut self, window: Option<HWND>, maximize: bool) -> Result<(), Error> {
        let hwnd = window.or_else(get_foreground_window).ok_or(Error::NoWindow)?;
        let tile_state = self.get_window_tile_state(hwnd).ok_or(Error::NoWindow)?;

        if matches!(tile_state, WindowTileState::Floating) {
            return Ok(());
        }

        if maximize {
            self.maximized_wins.insert(hwnd.0);
        } else {
            self.maximized_wins.remove(&hwnd.0);
        }

        self.update(true);
        Ok(())
    }

    pub fn update(&mut self, animate: bool) {
        let anim_player = &mut self.animation_player;
        self.containers.values_mut().for_each(|c| {
            let (border_pad, tile_pad) = match c.is_focalized() {
                true => (self.config.get_focalized_pad(), (0, 0)),
                false => (self.config.get_border_pad(), self.config.get_tile_pad_xy()),
            };

            // INFO: prevent updates when the monitor has a maximized window
            if self.maximized_wins.iter().any(|w| c.has(*w)) {
                return;
            }

            if let Some(c) = c.get_active_mut() {
                let _ = c.update(border_pad, tile_pad, anim_player, &self.maximized_wins);
            }
        });
        let animation = self.config.get_animations().filter(|_| animate);
        anim_player.play(animation);
    }

    pub fn cancel_animation(&mut self) {
        self.animation_player.cancel();
    }

    fn focus_next(&mut self) {
        let directions = [Direction::Right, Direction::Down, Direction::Left, Direction::Up];
        if let Some(leaf) = directions.iter().find_map(|d| self.find_at(*d, false)) {
            WindowRef::new(HWND(leaf.id)).focus();
        }
    }

    fn update_if(&mut self, condition: bool) {
        if condition {
            self.update(true);
        }
    }

    pub fn get_managed_windows(&self) -> HashMap<isize, WindowTileState> {
        let cs = self.containers.values();
        cs.flat_map(|c| c.iter().flat_map(|(_, t)| t.get_ids()))
            .filter_map(|hwnd| self.get_window_tile_state(HWND(hwnd)).map(|state| (hwnd, state)))
            .collect()
    }

    pub fn has_window(&self, hwnd: HWND) -> bool {
        let mut cs = self.containers.values();
        cs.any(|c| c.get(ContainerType::Normal(0).into()).unwrap().has(hwnd.0))
    }

    fn get_window_tile_state(&self, hwnd: HWND) -> Option<WindowTileState> {
        let is_managed = self.has_window(hwnd);
        let is_floating = self.floating_wins.contains(&hwnd.0);
        let is_ignored = self.maximized_wins.contains(&hwnd.0);

        if is_managed && !is_floating && !is_ignored {
            Some(WindowTileState::Normal)
        } else if is_floating {
            Some(WindowTileState::Floating)
        } else if is_ignored {
            Some(WindowTileState::Ignored)
        } else {
            None
        }
    }
}

enum ContainerType {
    Normal(u8),
    Focalized,
}

trait FocalizableContainer {
    fn focalize(&mut self, win: isize);
    fn unfocalize(&mut self);
    fn is_focalized(&self) -> bool;
}

impl FocalizableContainer for Container<String> {
    fn focalize(&mut self, win: isize) {
        let _ = self.set_active(ContainerType::Focalized.into());
        if let Some(c) = self.get_active_mut() {
            c.clear();
            c.insert(win);
        }
    }

    fn unfocalize(&mut self) {
        if self.is_focalized() {
            self.get_active_mut().expect("Active should be Some").clear();
            let _ = self.set_active(ContainerType::Normal(0).into());
        }
    }

    fn is_focalized(&self) -> bool {
        self.is_active(ContainerType::Focalized.into())
    }
}

impl From<ContainerType> for String {
    fn from(val: ContainerType) -> Self {
        match val {
            ContainerType::Normal(i) => format!("normal-{}", i),
            ContainerType::Focalized => String::from("focalized"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TilesManagerError {
    Generic,
    WindowAlreadyAdded,
    NoWindowsInfo,
    ContainerNotFound { refresh: bool },
    NoWindow,
}

impl TilesManagerError {
    pub fn is_warn(&self) -> bool {
        matches!(self, Error::WindowAlreadyAdded | Error::NoWindowsInfo)
    }

    pub fn require_refresh(&self) -> bool {
        matches!(self, Error::ContainerNotFound { refresh: true })
    }
}
