use crate::app::structs::area::Area;
use crate::app::structs::area_tree::leaf::AreaLeaf;
use crate::app::structs::direction::Direction;
use crate::win32::api::window::get_foreground_window;
use crate::win32::window::window_obj::{WindowObjHandler, WindowObjInfo};
use crate::win32::window::window_ref::WindowRef;
use ::windows::Win32::Foundation::HWND;
use std::collections::{HashMap, HashSet};

use super::config::TilesManagerConfig;
use super::container::Container;
use super::containers_manager::ContainersManager;
use super::monitor_layout::MonitorLayout;

pub struct TilesManager {
    containers: ContainersManager<isize>,
    unmanaged_wins: HashSet<isize>,
    config: TilesManagerConfig,
}

impl TilesManager {
    /// Creates a new [`TilesManager`].
    pub fn new(monitors_layout: Vec<MonitorLayout>, config: Option<TilesManagerConfig>) -> Self {
        let config = config.unwrap_or_default();

        let cointainers: HashMap<isize, Container> = monitors_layout
            .into_iter()
            .map(|m| (m.monitor.id, m.monitor.into(), m.layout))
            .map(|(id, w, l)| (id, Container::new(w, l, config.get_border_pad())))
            .collect();

        let containers_manager = ContainersManager::new(cointainers);

        TilesManager {
            unmanaged_wins: HashSet::new(),
            containers: containers_manager,
            config,
        }
    }

    pub fn add(&mut self, win: WindowRef, update: bool) -> Result<(), Error> {
        if self.unmanaged_wins.contains(&win.hwnd.0) {
            return Ok(());
        }

        if self.has_window(win.hwnd) {
            return Err(Error::WindowAlreadyAdded);
        }

        let snapshot = win.snapshot().ok_or(Error::NoWindowsInfo)?;
        let viewarea = snapshot.viewarea.ok_or(Error::NoWindowsInfo)?;
        let tree = self.containers.which_tree_at_mut(viewarea.get_center());
        let tree = tree.ok_or(Error::NoWindowsInfo)?;

        tree.insert(win.hwnd.0);

        if update {
            self.update();
        }

        Ok(())
    }

    pub fn find_at(&mut self, direction: Direction) -> Option<AreaLeaf<isize>> {
        let current = get_foreground_window()?;

        if !self.has_window(current) || self.containers.get_windows_len() < 2 {
            return None;
        }

        let padxy = self.config.get_tile_pad_xy();
        let area = WindowRef::new(current).get_window_box()?.pad_xy((-padxy.0, -padxy.1));

        let selection_point = match direction {
            Direction::Right => area.get_ne_corner().with_offset(20, 5), // TODO Prefer up
            Direction::Down => area.get_se_corner().with_offset(-5, 20), // TODO Prefer left
            Direction::Left => area.get_sw_corner().with_offset(-20, -5), // TODO Prefer up
            Direction::Up => area.get_nw_corner().with_offset(5, -20),   // TODO Prefer left
        };

        if let Some(tree) = self.containers.which_tree_at_mut(selection_point) {
            // If the point is in a tree
            return tree.find_leaf_at(selection_point, self.config.get_border_pad());
        } else if let Some(container) = self.containers.which_nearest_mut(area.get_center(), direction) {
            // Otherwise, find the nearest container
            let area = container.workarea;
            let selection_point = match direction {
                Direction::Right => area.get_nw_corner(),
                Direction::Down => area.get_ne_corner(),
                Direction::Left => area.get_se_corner(),
                Direction::Up => area.get_sw_corner(),
            };
            return container
                .tree
                .find_leaf_at(selection_point, self.config.get_border_pad());
        }

        None
    }

    pub fn focus_at(&mut self, direction: Direction) -> Result<(), Error> {
        let leaf = self.find_at(direction).ok_or(Error::NoWindowFound)?;
        self.focus_window(leaf.id);

        Ok(())
    }

    pub fn focus_next(&mut self) {
        let current = match get_foreground_window() {
            Some(hwnd) => hwnd,
            None => return,
        };

        if !self.has_window(current) || self.containers.get_windows_len() < 2 {
            return;
        }

        let mut directions = vec![Direction::Right, Direction::Down, Direction::Left, Direction::Up];

        let mut leaf = None;
        while leaf.is_none() && !directions.is_empty() {
            leaf = self.find_at(directions.pop().unwrap());
        }

        if let Some(leaf) = leaf {
            self.focus_window(leaf.id);
        }
    }

    pub fn remove(&mut self, hwnd: HWND, update: bool) -> Result<(), Error> {
        if self.unmanaged_wins.contains(&hwnd.0) {
            return Ok(());
        }

        if !self.containers.has_window(hwnd.0) {
            return Err(Error::NoWindowFound);
        }

        let tree = self.containers.which_tree_mut(hwnd.0).ok_or(Error::Generic)?;
        tree.remove(hwnd.0);

        if get_foreground_window().is_none() {
            self.focus_next();
        }

        if update {
            self.update();
        }

        Ok(())
    }

    pub fn refresh_window_size(&mut self, hwnd: HWND, update: bool) -> Result<(), Error> {
        let win = WindowRef::new(hwnd);
        let wb = win.get_window_box().ok_or(Error::NoWindowsInfo)?;
        self.resize(hwnd, wb, update)?;

        Ok(())
    }

    pub(crate) fn move_to(&mut self, direction: Direction, update: bool) -> Result<(), Error> {
        let curr = get_foreground_window().ok_or(Error::NoWindow)?;

        let src_center = self.get_stored_area(curr, true).ok_or(Error::Generic)?.get_center();
        let w = self.find_at(direction).ok_or(Error::NoWindowFound)?;
        let target_center = w.viewbox.get_center();

        let containers = &mut self.containers;
        if containers.is_same_container(src_center, target_center) {
            let tree = containers.which_tree_at_mut(src_center).ok_or(Error::Generic)?;
            tree.swap_ids_at(src_center, target_center);
        } else {
            let replaced_id = containers
                .which_tree_at_mut(target_center)
                .map(|t| t.replace_id(target_center, curr.0))
                .ok_or(Error::Generic)?
                .ok_or(Error::Generic)?;

            let tree = containers.which_tree_at_mut(src_center).ok_or(Error::Generic)?;
            tree.replace_id(src_center, replaced_id);
        };

        if update {
            self.update();
        }

        Ok(())
    }

    pub(crate) fn resize_on(&mut self, direction: Direction, size: u8, update: bool) -> Result<(), Error> {
        let curr = get_foreground_window().ok_or(Error::NoWindow)?;
        let area = self.get_stored_area(curr, true).ok_or(Error::NoWindowsInfo)?;
        let size = size as i16;
        let padding = match direction {
            Direction::Right => (Some((0, -size)), None),
            Direction::Down => (None, Some((0, -size))),
            Direction::Left => (Some((-size, 0)), None),
            Direction::Up => (None, Some((-size, 0))),
        };
        self.resize(curr, area.pad(padding.0, padding.1), update)
    }

    pub(crate) fn invert_orientation(&mut self, update: bool) -> Result<(), Error> {
        let curr = get_foreground_window().ok_or(Error::NoWindow)?;
        let area = self.get_stored_area(curr, true).ok_or(Error::Generic)?;
        let center = area.get_center();
        let tree = self.containers.which_tree_at_mut(center).ok_or(Error::Generic)?;
        tree.switch_subtree_orientations(center);

        if update {
            self.update();
        }

        Ok(())
    }

    pub fn move_window(
        &mut self,
        hwnd: HWND,
        new_point: (i32, i32),
        invert_monitor_op: bool,
        switch_orient: bool,
        update: bool,
    ) -> Result<(), Error> {
        if self.unmanaged_wins.contains(&hwnd.0) {
            return Ok(());
        }

        if !self.has_window(hwnd) {
            return Err(Error::NoWindow);
        }

        let area = self.get_stored_area(hwnd, true).ok_or(Error::Generic)?;
        let center = area.get_center();

        let cs = &mut self.containers;
        let not_found_err = Error::ContainerNotFound { refresh: true };
        if cs.is_same_container(center, new_point) {
            // If it is in the same monitor
            let tree = cs.which_tree_at_mut(center).ok_or(Error::Generic)?;
            tree.swap_ids_at(center, new_point);
        } else if self.config.is_insert_in_monitor(invert_monitor_op) || switch_orient {
            // If it is in another monitor and insert
            {
                let tree = cs.which_tree_at_mut(new_point).ok_or(not_found_err)?;
                tree.insert(hwnd.0);
            }
            {
                let tree = cs.which_tree_at_mut(center).ok_or(not_found_err)?;
                tree.remove_at(center);
            }
        } else {
            // If it is in another monitor and swap
            let replaced_id = cs
                .which_tree_at_mut(new_point)
                .map(|t| t.replace_id(new_point, hwnd.0))
                .ok_or(not_found_err)?;

            let tree = cs.which_tree_at_mut(center).ok_or(not_found_err)?;
            match replaced_id {
                Some(id) => drop(tree.replace_id(center, id)),
                None => tree.remove_at(center),
            };
        };

        if switch_orient {
            let tree = cs.which_tree_at_mut(new_point).ok_or(not_found_err)?;
            tree.switch_subtree_orientations(new_point);
        }

        if update {
            self.update();
        }

        Ok(())
    }

    pub fn update(&mut self) {
        let ids = self.containers.get_containers_ids();
        ids.into_iter().for_each(|c| {
            let _ = self.update_container(c);
        });
    }

    pub fn get_managed_windows(&self) -> HashSet<isize> {
        self.containers.get_windows()
    }

    pub fn focus_window(&mut self, id: isize) {
        if !get_foreground_window().is_some_and(|hwnd| hwnd.0 == id) {
            WindowRef::new(HWND(id)).focus()
        }
    }

    pub fn has_window(&self, hwnd: HWND) -> bool {
        self.containers.has_window(hwnd.0)
    }

    pub fn minimize(&mut self) -> Result<(), Error> {
        let win_ref = WindowRef::new(get_foreground_window().ok_or(Error::NoWindow)?);
        win_ref.minimize();
        self.focus_next();
        Ok(())
    }

    pub(crate) fn set_release(&mut self, release: Option<bool>) -> Result<(), Error> {
        let hwnd = get_foreground_window().ok_or(Error::NoWindow)?;
        let is_managed = self.has_window(hwnd);
        let is_unmanaged = self.unmanaged_wins.contains(&hwnd.0);

        if !is_managed && !is_unmanaged {
            return Err(Error::NoWindow);
        }

        let release = release.unwrap_or(!is_unmanaged);
        if release {
            self.remove(hwnd, false)?;
            self.unmanaged_wins.insert(hwnd.0);
            self.update();
        } else {
            self.unmanaged_wins.remove(&hwnd.0);
            match self.add(WindowRef::new(hwnd), false) {
                Ok(_) => {}
                Err(e) => {
                    self.unmanaged_wins.insert(hwnd.0);
                    return Err(e);
                }
            }
            self.update();
        }
        Ok(())
    }

    fn resize(&mut self, hwnd: HWND, new_area: Area, update: bool) -> Result<(), Error> {
        if self.unmanaged_wins.contains(&hwnd.0) {
            return Ok(());
        }

        if !self.has_window(hwnd) {
            return Err(Error::NoWindow);
        }

        let win = WindowRef::new(hwnd);
        let area = self.get_stored_area(win.hwnd, true).ok_or(Error::NoWindowsInfo)?;
        let center = area.get_center();

        let win_tree = self.containers.which_tree_at_mut(center).ok_or(Error::Generic)?;
        let area_shift = new_area.get_shift(&area);
        let clamp_values = Some((10, 90));

        if area_shift.2 != 0 {
            let growth = (area_shift.2 as f32 / area.width as f32) * 100f32;
            let (x, growth) = match area_shift.0.abs() > 10 {
                true => (area.get_left_center().0.saturating_sub(20), -growth),
                false => (area.get_right_center().0.saturating_add(20), growth),
            };
            win_tree.resize_ancestor(center, (x, center.1), growth as i32, clamp_values);
        }

        if area_shift.3 != 0 {
            let growth = (area_shift.3 as f32 / area.height as f32) * 100f32;
            let (y, grow_perc) = match area_shift.1.abs() > 10 {
                true => (area.get_top_center().1.saturating_sub(20), -growth),
                false => (area.get_bottom_center().1.saturating_add(20), growth),
            };

            win_tree.resize_ancestor(center, (center.0, y), grow_perc as i32, clamp_values);
        }

        if update {
            self.update();
        }

        Ok(())
    }

    fn update_container(&mut self, container_id: isize) -> Result<(), Error> {
        let container = self.containers.which_by_id_mut(container_id);
        let tree = container
            .map(|c| &mut c.tree)
            .ok_or(Error::ContainerNotFound { refresh: false })?;

        let leaves: Vec<AreaLeaf<isize>> = tree.leaves(self.config.get_border_pad());
        let mut errors = vec![];
        for leaf in &leaves {
            let win_ref = WindowRef::from(leaf.id);
            let area = leaf.viewbox.pad_xy(self.config.get_tile_pad_xy());
            let res = win_ref.resize_and_move(area.get_origin(), area.get_size());

            if res.is_err() {
                log::warn!("Failed to resize window: {}", win_ref.hwnd.0);
                errors.push(win_ref.hwnd.0);
            }
        }

        if !errors.is_empty() {
            errors.iter().for_each(|e| tree.remove(*e));
            return self.update_container(container_id);
        }

        Ok(())
    }

    fn get_stored_area(&self, hwnd: HWND, with_padding: bool) -> Option<Area> {
        let border_pad = if with_padding { self.config.get_border_pad() } else { 0 };
        let tile_pad = match with_padding {
            true => self.config.get_tile_pad_xy(),
            false => (0, 0),
        };
        self.containers
            .get_leaf(hwnd.0, border_pad)
            .map(|l| l.viewbox.pad_xy(tile_pad))
    }
}

trait Point {
    fn with_offset(&self, offset_x: i32, offset_y: i32) -> (i32, i32);
}

impl Point for (i32, i32) {
    fn with_offset(&self, offset_x: i32, offset_y: i32) -> (i32, i32) {
        (self.0.saturating_add(offset_x), self.1.saturating_add(offset_y))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Error {
    Generic,
    WindowAlreadyAdded,
    NoWindowsInfo,
    ContainerNotFound { refresh: bool },
    NoWindow,
    NoWindowFound,
}

impl Error {
    pub fn is_warn(&self) -> bool {
        matches!(self, Error::WindowAlreadyAdded | Error::NoWindowsInfo)
    }

    pub fn require_refresh(&self) -> bool {
        matches!(self, Error::ContainerNotFound { refresh: true })
    }
}
