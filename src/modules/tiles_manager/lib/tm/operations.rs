use super::floating::FloatingProperties;
use super::floating::FloatingWindows;
use super::result::TilesManagerError;
use super::result::TilesManagerSuccess;
use super::TilesManager;
use crate::app::area_tree::leaf::AreaLeaf;
use crate::app::configs::floating::FloatingWinsConfig;
use crate::app::mondrian_message::WindowTileState;
use crate::app::structs::direction::Direction;
use crate::app::structs::orientation::Orientation;
use crate::modules::tiles_manager::lib::containers::container::ContainerLayer;
use crate::modules::tiles_manager::lib::containers::map::ActiveContainersMap;
use crate::modules::tiles_manager::lib::containers::map::ContainersMap;
use crate::modules::tiles_manager::lib::containers::Containers;
use crate::modules::tiles_manager::lib::containers::ContainersMut;
use crate::modules::tiles_manager::lib::structs::rules::Rules;
use crate::modules::tiles_manager::lib::utils::get_floating_win_area;
use crate::modules::tiles_manager::lib::utils::leaves_limited_by_edge;
use crate::win32::window::window_obj::WindowObjHandler;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;

type Success = TilesManagerSuccess;
type Error = TilesManagerError;
type TileState = WindowTileState;
type TMResult = Result<TilesManagerSuccess, Error>;

pub trait TilesManagerOperations {
    /// Adds a window to the tile manager.
    /// When `prefer_position` is Some, the window will be added to the monitor that contains the
    /// given position. Otherwise, the window will be added to the monitor that contains the center of the window.
    /// If the target monitor has a focalized window or a maximized window, it will be restored.
    fn add(
        &mut self,
        win: WindowRef,
        prefer_position: Option<(i32, i32)>,
        check_rules: bool,
        prevent_workspace_switch: bool,
    ) -> TMResult;

    /// Removes a window from the tile manager.
    fn remove(&mut self, win: WindowRef) -> TMResult;

    fn release(&mut self, window: WindowRef, release: Option<bool>, config: Option<FloatingWinsConfig>) -> TMResult;
    fn as_maximized(&mut self, window: WindowRef, maximize: bool) -> TMResult;
    fn focalize(&mut self, window: WindowRef, focalize: Option<bool>) -> TMResult;
    fn half_focalize(&mut self, window: WindowRef, half_focalize: Option<bool>) -> TMResult;
    fn find_neighbour(
        &self,
        window: WindowRef,
        direction: Direction,
        search_strategy: MonitorSearchStrategy,
    ) -> Option<AreaLeaf<WindowRef>>;

    /// Swaps the position of two windows. The new positions are propagated to the inactive
    /// containers if necessary.
    fn swap_windows(&mut self, win1: WindowRef, win2: WindowRef) -> TMResult;
    fn resize(&mut self, window: WindowRef, delta: (i32, i32, i32, i32)) -> TMResult;

    /// Moves a window to the monitor that contains the given position.
    /// If `free_move` is true, the window will be moved to the given position without following
    /// the layout strategy of the tile manager.
    /// If the target monitor is the same as the current monitor, the window will not be moved.
    fn insert_window(&mut self, win: WindowRef, point: (i32, i32), free_move: bool) -> TMResult;
    fn insert_window_to_workspace(&mut self, win: WindowRef, workspace: &str, monitor: Option<&str>) -> TMResult;
}

const C_ERR: Error = Error::ContainerNotFound { refresh: false };

impl TilesManagerOperations for TilesManager {
    fn add(
        &mut self,
        win: WindowRef,
        prefer_position: Option<(i32, i32)>,
        check_rules: bool,
        prevent_workspace_switch: bool,
    ) -> TMResult {
        let tile_state = self.get_window_state(win);
        if tile_state
            .as_ref()
            .is_ok_and(|s| matches!(s, TileState::Floating | TileState::Maximized | TileState::Focalized))
        {
            return Ok(Success::NoChange);
        }

        // INFO: if the window in an inactive workspace, activate it
        if let Some(k) = self
            .inactive_containers
            .get_key_with_window(self.current_vd.into(), &win)
        {
            self.activate_workspace(&k.monitor, &k.workspace, false, true)?;
            let c = self.containers.get_mut(&k.into()).ok_or(C_ERR)?;
            // INFO: if the window is not in the current layer, it must be in the normal one
            if !c.tree().has(win) {
                c.set_current(ContainerLayer::Normal);
            }
            return Ok(Success::LayoutChanged);
        }

        let add_opt = self.config.rules.get_add_options(win);

        let center = prefer_position.or_else(|| win.get_area().map(|a| a.get_center()));
        let center = center.ok_or(Error::NoWindow)?;
        let center = add_opt
            .as_ref()
            .and_then(|opt| opt.monitor.clone())
            .filter(|_| check_rules)
            .and_then(|m| {
                self.managed_monitors
                    .get(&m)
                    .map(|m| m.info.get_workspace().get_center())
            })
            .unwrap_or(center);

        let k = self.containers.find_near(center).map(|e| e.key)?;
        let prev_workspace = k.workspace.clone();
        let trg_workspace_opt = match check_rules {
            true => add_opt
                .as_ref()
                .and_then(|opt| opt.workspace_options.clone())
                .filter(|o| o.workspace != prev_workspace),
            false => None,
        };

        if let Some(opt) = trg_workspace_opt.as_ref() {
            let _ = self.activate_workspace(
                &k.monitor,
                &opt.workspace,
                opt.silent || prevent_workspace_switch,
                false,
            );
        }

        let ct = self.containers.get_mut(&k).ok_or(C_ERR)?.current();
        if ct.is_focalized_or_half() && tile_state.is_err() {
            self.restore_monitor(&k)?;
            if self.containers.find(win).is_ok() {
                return Ok(Success::LayoutChanged);
            }
        }

        if self.containers.find(win).is_ok() {
            return Err(Error::WindowAlreadyAdded(win));
        }

        let container = self.containers.get_mut(&k).ok_or(C_ERR)?;
        win.set_topmost(false).ok();
        container.tree_mut().insert(win);

        // INFO: if the monitor has a maximized window, restore it
        if trg_workspace_opt.is_none() {
            self.restore_maximized(&k)?;
        }

        if check_rules {
            if let Some(config) = add_opt.and_then(|opt| opt.floating_config) {
                return self.release(win, Some(true), Some(config));
            }
        }

        if trg_workspace_opt.is_some_and(|opt| opt.silent || prevent_workspace_switch) {
            let _ = self.activate_workspace(&k.monitor, &prev_workspace, false, false);
        }

        Ok(Success::LayoutChanged)
    }

    fn remove(&mut self, win: WindowRef) -> TMResult {
        let tile_state = self.get_window_state(win)?;

        if matches!(tile_state, TileState::Floating) {
            if self.floating_wins.can_be_closed(&win) {
                self.floating_wins.remove(&win);
            }
            return Ok(Success::NoChange);
        }

        let k = self.containers.find(win)?.key;
        if matches!(tile_state, TileState::Focalized | TileState::HalfFocalized) {
            self.restore_monitor(&k)?;
        }

        self.containers.find_mut(win)?.value.tree_mut().remove(win);

        if matches!(tile_state, TileState::Maximized) {
            self.maximized_wins.remove(&win);
        }

        Ok(Success::LayoutChanged)
    }

    fn release(&mut self, window: WindowRef, release: Option<bool>, config: Option<FloatingWinsConfig>) -> TMResult {
        let tile_state = self.get_window_state(window)?;

        if matches!(tile_state, TileState::Maximized) {
            return Ok(Success::NoChange);
        }

        let config = &config.unwrap_or(self.config.floating_wins);
        if release.unwrap_or(!matches!(tile_state, TileState::Floating)) {
            let monitor_area = self.containers.find(window)?.value.tree().get_area();
            self.remove(window)?;
            self.floating_wins.insert(window, FloatingProperties::new());
            let area = get_floating_win_area(&monitor_area, &window, config)?;
            let is_topmost = config.topmost;
            let _ = window.set_topmost(is_topmost);
            Ok(Success::queue(window, area, Some(is_topmost)))
        } else {
            self.floating_wins.remove(&window);
            self.add(window, None, false, true)?;
            let _ = window.set_topmost(false);
            Ok(Success::dequeue(window))
        }
    }

    fn as_maximized(&mut self, window: WindowRef, maximized: bool) -> TMResult {
        let tile_state = self.get_window_state(window)?;

        if matches!(tile_state, TileState::Floating) {
            return Ok(Success::NoChange);
        }

        if maximized {
            let src_e = self.containers.find(window)?;
            let win_center = window.get_area().ok_or(Error::NoWindowsInfo)?.get_center();
            let trg_e = self.containers.find_near(win_center)?;

            // NOTE: if the window is maximized to another monitor, remove it and add it again
            if src_e.key != trg_e.key {
                self.remove(window)?;
                self.add(window, None, true, false)?;
            }

            self.maximized_wins.insert(window);
        } else {
            self.maximized_wins.remove(&window);
        }

        Ok(Success::LayoutChanged)
    }

    fn focalize(&mut self, window: WindowRef, focalize: Option<bool>) -> TMResult {
        let tile_state = self.get_window_state(window);
        if tile_state.is_ok_and(|s| matches!(s, TileState::Floating)) {
            self.release(window, Some(false), None)?;
        }

        let e = self.containers.find(window)?;
        let k = e.key;
        let ct = e.value.current();

        if focalize.is_some_and(|f| f)
            || (focalize.is_none() && matches!(ct, ContainerLayer::Normal | ContainerLayer::HalfFocalized))
        {
            let wins = e.value.tree().get_ids().to_vec();
            let container = self.containers.get_mut(&k).ok_or(C_ERR)?;
            container.set_current(ContainerLayer::Focalized);
            container.tree_mut().clear();
            container.tree_mut().insert(window);
            wins.iter().filter(|w| **w != window).for_each(|w| {
                w.minimize(true);
            });
            return Ok(Success::LayoutChanged);
        }

        if focalize.is_some_and(|f| !f) || (focalize.is_none() && matches!(ct, ContainerLayer::Focalized)) {
            let container = self.containers.get_mut(&k).ok_or(C_ERR)?;
            container.tree_mut().clear();
            container.set_current(ContainerLayer::Normal);
            return Ok(Success::LayoutChanged);
        }

        Ok(Success::NoChange)
    }

    fn half_focalize(&mut self, window: WindowRef, half_focalize: Option<bool>) -> TMResult {
        let e = self.containers.find_mut(window)?;
        let k = e.key;
        let ct = e.value.current();

        if half_focalize.is_some_and(|f| f) || (half_focalize.is_none() && matches!(ct, ContainerLayer::Normal)) {
            let mut new_tree = (*e.value.tree()).clone();
            let mut leaves = e.value.tree().leaves(None);
            if leaves.len() < 2 {
                return Ok(Success::NoChange);
            }

            leaves.sort_by(|a, b| a.viewbox.calc_area().cmp(&b.viewbox.calc_area()));
            let biggest_leaf = match leaves.pop().ok_or(Error::Generic)? {
                l if l.id == window => leaves.pop().ok_or(Error::Generic)?,
                l => l,
            };

            for l in leaves.iter().filter(|l| l.id != window && l.id != biggest_leaf.id) {
                new_tree.remove(l.id);
                l.id.minimize(true);
            }

            let container = self.containers.get_mut(&k).ok_or(Error::Generic)?;
            container.set_current(ContainerLayer::HalfFocalized);
            container.tree_mut().replace_root(new_tree);
            return Ok(Success::LayoutChanged);
        } else if half_focalize.is_some_and(|f| !f)
            || (half_focalize.is_none() && matches!(ct, ContainerLayer::HalfFocalized))
        {
            let container = self.containers.get_mut(&k).ok_or(Error::Generic)?;
            container.tree_mut().clear();
            container.set_current(ContainerLayer::Normal);
            return Ok(Success::LayoutChanged);
        }

        Ok(Success::NoChange)
    }

    fn find_neighbour(
        &self,
        window: WindowRef,
        direction: Direction,
        strategy: MonitorSearchStrategy,
    ) -> Option<AreaLeaf<WindowRef>> {
        let src_entry = self.containers.find(window).ok()?;

        if (src_entry.value.current() == ContainerLayer::Focalized
            || self.get_maximized_win_in_monitor(&src_entry.key).is_some())
            && matches!(strategy, MonitorSearchStrategy::Same)
        {
            return None;
        };

        if self.get_maximized_win_in_monitor(&src_entry.key).is_some() && matches!(strategy, MonitorSearchStrategy::Any)
        {
            return self.find_neighbour_closest_monitor(window, direction);
        };

        if matches!(strategy, MonitorSearchStrategy::Same | MonitorSearchStrategy::Any) {
            if let Some(l) = self.find_neighbour_same_monitor(window, direction) {
                return Some(l);
            }
        }

        if matches!(strategy, MonitorSearchStrategy::Any) {
            if let Some(l) = self.find_neighbour_closest_monitor(window, direction) {
                return Some(l);
            }
        }

        None
    }

    fn swap_windows(&mut self, w1: WindowRef, w2: WindowRef) -> TMResult {
        let src_k = self.containers.find(w1)?.key;
        let trg_k = self.containers.find(w2)?.key;

        self.restore_maximized(&src_k).ok();
        self.restore_maximized(&trg_k).ok();
        if src_k == trg_k {
            let t = self.containers.get_mut(&src_k).ok_or(C_ERR)?;
            t.tree_mut().swap_ids(w1, w2);
            if t.current().is_focalized_or_half() {
                t.get_tree_mut(ContainerLayer::Normal).swap_ids(w1, w2);
            }
        } else {
            let t = self.containers.get_mut(&src_k).ok_or(C_ERR)?;
            t.tree_mut().replace_id(w1, w2);
            if t.current().is_focalized_or_half() {
                t.get_tree_mut(ContainerLayer::Normal).replace_id(w1, w2);
            }

            let t = self.containers.get_mut(&trg_k).ok_or(C_ERR)?;
            t.tree_mut().replace_id(w2, w1);
            if t.current().is_focalized_or_half() {
                t.get_tree_mut(ContainerLayer::Normal).replace_id(w2, w1);
            }
        };

        Ok(Success::LayoutChanged)
    }

    fn resize(&mut self, win: WindowRef, delta: (i32, i32, i32, i32)) -> TMResult {
        let tile_state = self.get_window_state(win)?;
        if matches!(
            tile_state,
            TileState::Floating | TileState::Maximized | TileState::Focalized
        ) {
            return Ok(Success::NoChange);
        }

        let (resize_w, resize_h) = (delta.2 != 0, delta.3 != 0);
        let (resize_left, resize_up) = (delta.0.abs() > 10, delta.1.abs() > 10);
        let has_w_neigh = match resize_w {
            true => self.find_neighbour(
                win,
                if resize_left { Direction::Left } else { Direction::Right },
                MonitorSearchStrategy::Same,
            ),
            false => None,
        };
        let has_h_neigh = match resize_h {
            true => self.find_neighbour(
                win,
                if resize_up { Direction::Up } else { Direction::Down },
                MonitorSearchStrategy::Same,
            ),
            false => None,
        };

        let t = self.containers.find_mut(win)?.value;
        let area = t.tree().find_leaf(win, 0).ok_or(Error::NoWindow)?.viewbox;
        let center = area.get_center();

        const CLAMP_VALUES: Option<(u8, u8)> = Some((10, 90));

        if resize_w && has_w_neigh.is_some() {
            let growth = delta.2;
            let (x, growth) = match resize_left {
                true => (area.get_left_center().0.saturating_sub(20), -growth),
                false => (area.get_right_center().0.saturating_add(20), growth),
            };
            t.tree_mut()
                .resize_ancestor(center, (x, center.1), growth, CLAMP_VALUES);
        }

        if resize_h && has_h_neigh.is_some() {
            let growth = delta.3;
            let (y, growth) = match resize_up {
                true => (area.get_top_center().1.saturating_sub(20), -growth),
                false => (area.get_bottom_center().1.saturating_add(20), growth),
            };
            t.tree_mut()
                .resize_ancestor(center, (center.0, y), growth, CLAMP_VALUES);
        }

        Ok(Success::LayoutChanged)
    }

    fn insert_window(&mut self, win: WindowRef, point: (i32, i32), free_move: bool) -> TMResult {
        let tile_state = self.get_window_state(win)?;
        if matches!(tile_state, TileState::Floating | TileState::Maximized) {
            return Ok(Success::NoChange);
        }

        let src_k = self.containers.find(win)?.key;
        let trg_k = self.containers.find_near(point)?.key;
        if src_k == trg_k {
            return Ok(Success::NoChange);
        }

        if !self
            .containers
            .get_mut(&trg_k)
            .ok_or(Error::NoWindow)?
            .tree()
            .contains(point, false)
            && free_move
        {
            return Ok(Success::NoChange);
        }

        self.restore_monitor(&src_k)?;
        self.containers
            .get_mut(&src_k)
            .ok_or(Error::NoWindow)?
            .tree_mut()
            .remove(win);

        self.restore_monitor(&trg_k)?;
        self.restore_maximized(&trg_k)?;

        let trg = self.containers.get_mut(&trg_k).ok_or(Error::NoWindow)?;
        match free_move {
            true => trg.tree_mut().insert_at(win, point),
            false => trg.tree_mut().insert(win),
        }

        Ok(Success::LayoutChanged)
    }

    fn insert_window_to_workspace(&mut self, win: WindowRef, workspace: &str, monitor: Option<&str>) -> TMResult {
        let tile_state = self.get_window_state(win)?;
        if matches!(tile_state, TileState::Floating | TileState::Maximized) {
            return Ok(Success::NoChange);
        }

        let win_container_k = self.containers.find(win)?.key;
        let trg_monitor = monitor.unwrap_or(&win_container_k.monitor);
        if win_container_k.workspace == workspace && win_container_k.monitor == trg_monitor {
            return Ok(Success::NoChange);
        }

        let prev_k = self.containers.get_key_by_monitor(trg_monitor)?;
        let trg_monitor_area = self
            .managed_monitors
            .get(trg_monitor)
            .map(|m| m.info.get_workspace())
            .ok_or(Error::NoWindow)?;

        self.remove(win)?;
        self.activate_workspace(trg_monitor, workspace, true, false).ok();
        self.add(win, Some(trg_monitor_area.get_center()), false, true)?;
        self.activate_workspace(trg_monitor, &prev_k.workspace, true, false)
            .ok();
        Ok(Success::LayoutChanged)
    }
}

impl TilesManager {
    fn find_neighbour_same_monitor(&self, window: WindowRef, direction: Direction) -> Option<AreaLeaf<WindowRef>> {
        let src_entry = self.containers.find(window).ok()?;
        let src_area = src_entry.value.tree().find_leaf(window, 0)?.viewbox;
        let pad = match direction.axis() {
            Orientation::Horizontal => (0, 1),
            Orientation::Vertical => (1, 0),
        };
        self.find_best_matching_leaf(
            window,
            direction,
            &self.containers.get(&src_entry.key)?.tree().leaves(None),
            src_area.pad_xy(pad).get_corners(direction),
        )
    }

    fn find_neighbour_closest_monitor(&self, window: WindowRef, direction: Direction) -> Option<AreaLeaf<WindowRef>> {
        let src_area = self.containers.find_leaf(window).ok()?.viewbox;
        let e = self.containers.find_closest_at(src_area.get_center(), direction).ok()?;

        self.get_maximized_leaf_in_monitor(&e.key).or_else(|| {
            self.find_best_matching_leaf(
                window,
                direction,
                &e.value.tree().leaves(None),
                e.value.tree().get_area().get_corners(direction.opposite()),
            )
        })
    }

    fn find_best_matching_leaf(
        &self,
        excluded_window: WindowRef,
        direction: Direction,
        leaves: &[AreaLeaf<WindowRef>],
        edge: [(i32, i32); 2],
    ) -> Option<AreaLeaf<WindowRef>> {
        let leaves = leaves_limited_by_edge(leaves, direction, edge);
        if self.config.history_based_navigation {
            let leaves: Vec<AreaLeaf<WindowRef>> = leaves.iter().filter(|l| l.id != excluded_window).copied().collect();
            let leaf = self.focus_history.latest(&leaves).copied();
            if leaf.is_some() {
                return leaf;
            }
        }
        leaves
            .first()
            .filter(|l| l.id != excluded_window)
            .or_else(|| leaves.get(1))
            .copied()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MonitorSearchStrategy {
    Same,
    Any,
}
