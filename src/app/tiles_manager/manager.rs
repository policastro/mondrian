use crate::app::structs::area_tree::leaf::AreaLeaf;
use crate::app::structs::area_tree::tree::WinTree;
use crate::app::structs::direction::Direction;
use crate::app::structs::point::Point;
use crate::win32::api::window::get_foreground_window;
use crate::win32::window::window_obj::{WindowObjHandler, WindowObjInfo};
use crate::win32::window::window_ref::WindowRef;
use std::collections::{HashMap, HashSet};
use windows::Win32::Foundation::HWND;

use super::config::TilesManagerConfig;
use super::container::{Animator, Container, ContainerLayer};
use super::containers_manager::Containers;
use super::monitor_layout::MonitorLayout;

pub struct TilesManager {
    containers: HashMap<isize, Container<String>>,
    unmanaged_wins: HashSet<isize>,
    config: TilesManagerConfig,
}

impl TilesManager {
    /// Creates a new [`TilesManager`].
    pub fn new(monitors_layout: Vec<MonitorLayout>, config: Option<TilesManagerConfig>) -> Self {
        let config = config.unwrap_or_default();
        let containers = monitors_layout
            .into_iter()
            .map(|m| (m.monitor.id, m.monitor.into(), m.layout))
            .map(|(id, w, l)| {
                let mut c = Container::<String>::new();
                c.add(ContainerType::Normal.into(), WinTree::new(w, l.clone()));
                c.add(ContainerType::Focalized.into(), WinTree::new(w, l));
                let _ = c.set_active(ContainerType::Normal.into());
                (id, c)
            })
            .collect();

        TilesManager {
            unmanaged_wins: HashSet::new(),
            containers,
            config,
        }
    }

    // TODO ok
    pub fn add(&mut self, win: WindowRef, update: bool) -> Result<(), Error> {
        if self.unmanaged_wins.contains(&win.hwnd.0) {
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
        let tree = self.containers.find_at_mut(center).and_then(|c| c.get_active_mut());
        tree.ok_or(Error::NoWindowsInfo)?.insert(win.hwnd.0);

        self.update_if(update);
        Ok(())
    }

    pub fn remove(&mut self, hwnd: HWND, skip_focalized: bool, update: bool) -> Result<(), Error> {
        if self.unmanaged_wins.contains(&hwnd.0) {
            return Ok(());
        }

        if !self.has_window(hwnd) {
            return Err(Error::NoWindowFound);
        }

        let c = self.containers.find_mut(hwnd.0, false); //TODO tutti
        if skip_focalized && c.is_some_and(|c| c.is_focalized()) {
            return Ok(());
        }

        let c = self.containers.find_mut(hwnd.0, false).ok_or(Error::NoWindowFound)?; //TODO tutti
        c.iter_mut().for_each(|(_, t)| t.remove(hwnd.0));

        if get_foreground_window().is_none() {
            self.focus_next();
        }

        self.update_if(update);
        Ok(())
    }

    fn find_at(&self, direction: Direction, same_monitor: bool) -> Option<AreaLeaf<isize>> {
        let current = get_foreground_window()?;
        let orig_c = self.containers.find(current.0, true)?;

        let padxy = self.config.get_tile_pad_xy();
        let area = WindowRef::new(current).get_window_box()?.pad_xy((-padxy.0, -padxy.1));
        let point = match direction {
            Direction::Right => area.get_ne_corner().with_offset(20, 5), // TODO Prefer up
            Direction::Down => area.get_se_corner().with_offset(-5, 20), // TODO Prefer left
            Direction::Left => area.get_sw_corner().with_offset(-20, -5), // TODO Prefer up
            Direction::Up => area.get_nw_corner().with_offset(5, -20),   // TODO Prefer left
        };

        let params = if let Some(c) = self.containers.find_at(point) {
            // If the point is in a tree
            Some((c, point))
        } else if let Some(c) = self.containers.find_nearest(area.get_center(), direction) {
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
            true => orig_c.get_active()?,
            false => params?.0.get_active()?,
        };

        t.find_leaf_at(params?.1, 0).filter(|w| w.id != current.0)
    }

    pub fn focus_at(&mut self, direction: Direction) -> Result<(), Error> {
        let leaf = self.find_at(direction, false).ok_or(Error::NoWindowFound)?;
        WindowRef::new(HWND(leaf.id)).focus();
        Ok(())
    }

    pub fn on_move(
        &mut self,
        hwnd: HWND,
        target: (i32, i32),
        invert_monitor_op: bool,
        switch_orient: bool,
    ) -> Result<(), Error> {
        if self.unmanaged_wins.contains(&hwnd.0) {
            return Ok(());
        }

        if !self.has_window(hwnd) {
            return Err(Error::NoWindow);
        }

        let c = self.containers.find(hwnd.0, false).and_then(|c| c.get_active());
        let leaf = c.and_then(|t| t.find_leaf(hwnd.0, 0)).ok_or(Error::Generic)?;
        let center = leaf.viewbox.get_center();

        let cs = &mut self.containers;
        let c_err = Error::ContainerNotFound { refresh: true };

        if cs.is_same_container(center, target) {
            // If it is in the same monitor
            let tree = cs.find_at_mut(center).and_then(|c| c.get_active_mut());
            tree.ok_or(Error::Generic)?.swap_ids_at(center, target);
        } else if self.config.is_insert_in_monitor(invert_monitor_op) || switch_orient {
            // If it is in another monitor and insert
            let c = cs.find_at_mut(target).ok_or(c_err)?;
            c.unfocalize();
            c.get_active_mut().ok_or(c_err)?.insert(hwnd.0);

            let c = cs.find_at_mut(center).ok_or(c_err)?;
            c.unfocalize();
            c.get_active_mut().ok_or(c_err)?.remove_at(center);
        } else {
            // If it is in another monitor and swap
            let tree = cs.find_at_mut(target).and_then(|c| c.get_active_mut());
            let replaced_id = tree.ok_or(c_err)?.replace_id_at(target, hwnd.0);

            let tree = cs.find_at_mut(center).and_then(|c| c.get_active_mut());
            let tree = tree.ok_or(c_err)?;
            match replaced_id {
                Some(id) => drop(tree.replace_id_at(center, id)),
                None => tree.remove_at(center),
            };
        };

        if switch_orient {
            let tree = cs.find_at_mut(target).and_then(|c| c.get_active_mut());
            tree.ok_or(c_err)?.switch_subtree_orientations(target);
        }

        self.update();
        Ok(())
    }

    pub(crate) fn move_focused(&mut self, direction: Direction) -> Result<(), Error> {
        let curr = get_foreground_window().ok_or(Error::NoWindow)?;

        let c = self.containers.find(curr.0, true).and_then(|c| c.get_active());
        let leaf = c.and_then(|t| t.find_leaf(curr.0, 0)).ok_or(Error::Generic)?;
        let src_center = leaf.viewbox.get_center();
        let w = self.find_at(direction, false).ok_or(Error::NoWindowFound)?;
        let trg_center = w.viewbox.get_center();

        let cs = &mut self.containers;
        if cs.is_same_container(src_center, trg_center) {
            let c = cs.find_at_mut(src_center).ok_or(Error::Generic)?;
            c.iter_mut().for_each(|(_, t)| t.swap_ids_at(src_center, trg_center));
        } else {
            let tree = cs.find_at_mut(trg_center).and_then(|c| c.get_active_mut());
            let replaced_id = tree.and_then(|t| t.replace_id_at(trg_center, curr.0));

            if let Some(id) = replaced_id {
                let src_tree = cs.find_at_mut(src_center).and_then(|c| c.get_active_mut());
                src_tree.ok_or(Error::Generic)?.replace_id_at(src_center, id);
            }
        };

        self.update();
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
        self.on_resize(curr, orig_area.get_shift(&area))
    }

    pub(crate) fn invert_orientation(&mut self) -> Result<(), Error> {
        let curr = get_foreground_window().ok_or(Error::NoWindow)?;
        let c = self.containers.find_mut(curr.0, true).and_then(|c| c.get_active_mut());
        let center = WindowRef::new(curr).get_window_box().map(|a| a.get_center());
        let center = center.ok_or(Error::NoWindowsInfo)?;
        c.ok_or(Error::Generic)?.switch_subtree_orientations(center);

        self.update();
        Ok(())
    }

    pub(crate) fn on_resize(&mut self, hwnd: HWND, delta: (i32, i32, i32, i32)) -> Result<(), Error> {
        if self.unmanaged_wins.contains(&hwnd.0) {
            return Ok(());
        }

        let c = self.containers.find_mut(hwnd.0, true).ok_or(Error::NoWindow)?;
        let t = c.get_active_mut().ok_or(Error::NoWindowFound)?;
        let area = t.find_leaf(hwnd.0, 0).ok_or(Error::Generic)?.viewbox;
        let center = area.get_center();

        let clamp_values = Some((10, 90));
        if delta.2 != 0 {
            let growth = (delta.2 as f32 / area.width as f32) * 100f32;
            let (x, growth_perc) = match delta.0.abs() > 10 {
                true => (area.get_left_center().0.saturating_sub(20), -growth),
                false => (area.get_right_center().0.saturating_add(20), growth),
            };
            t.resize_ancestor(center, (x, center.1), growth_perc as i32, clamp_values);
        }

        if delta.3 != 0 {
            let growth = (delta.3 as f32 / area.height as f32) * 100f32;
            let (y, growth_perc) = match delta.1.abs() > 10 {
                true => (area.get_top_center().1.saturating_sub(20), -growth),
                false => (area.get_bottom_center().1.saturating_add(20), growth),
            };
            t.resize_ancestor(center, (center.0, y), growth_perc as i32, clamp_values);
        }

        self.update();
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
        let c = self.containers.find_at_mut(center).ok_or(Error::NoWindowFound)?;

        if c.is_focalized() {
            match c.get_active_mut().ok_or(Error::Generic)?.has(hwnd.0) {
                true => c.unfocalize(),
                false => c.focalize(hwnd.0),
            }
        } else {
            let wins = c.get_active().ok_or(Error::Generic)?.get_ids();
            let wins = wins.iter().filter(|h| **h != hwnd.0).map(|h| WindowRef::new(HWND(*h)));
            wins.for_each(|w| {
                w.minimize();
            });
            c.focalize(hwnd.0);
            let _ = self.release_focused(Some(false), Some(hwnd));
        }

        self.update();
        Ok(())
    }

    pub(crate) fn release_focused(&mut self, release: Option<bool>, window: Option<HWND>) -> Result<(), Error> {
        let hwnd = window.or_else(get_foreground_window).ok_or(Error::NoWindow)?;
        let is_managed = self.has_window(hwnd);
        let is_unmanaged = self.unmanaged_wins.contains(&hwnd.0);

        if !is_managed && !is_unmanaged {
            return Err(Error::NoWindow);
        }

        let release = release.unwrap_or(!is_unmanaged);
        if release {
            self.remove(hwnd, false, false)?;
            self.unmanaged_wins.insert(hwnd.0);
        } else {
            self.unmanaged_wins.remove(&hwnd.0);
            self.add(WindowRef::new(hwnd), false)?;
        }

        self.update();
        Ok(())
    }

    pub fn update(&mut self) {
        self.containers.values_mut().for_each(|c| {
            let border_pad = match c.is_focalized() {
                true => self.config.get_focalized_pad(),
                false => self.config.get_border_pad(),
            };

            if let Some(c) = c.get_active_mut() {
                let mut anim = Animator {};
                let _ = c.update(border_pad, self.config.get_tile_pad_xy(), &mut anim);
            }
        });
    }

    fn focus_next(&mut self) {
        let directions = [Direction::Right, Direction::Down, Direction::Left, Direction::Up];
        if let Some(leaf) = directions.iter().find_map(|d| self.find_at(*d, false)) {
            WindowRef::new(HWND(leaf.id)).focus();
        }
    }

    fn update_if(&mut self, condition: bool) {
        if condition {
            self.update();
        }
    }

    pub fn get_managed_windows(&self) -> HashSet<isize> {
        let cs = self.containers.values();
        cs.flat_map(|c| c.iter().flat_map(|(_, t)| t.get_ids())).collect()
    }

    pub fn has_window(&self, hwnd: HWND) -> bool {
        let mut cs = self.containers.values();
        cs.any(|c| c.get(ContainerType::Normal.into()).unwrap().has(hwnd.0))
    }
}

enum ContainerType {
    Normal,
    Focalized,
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
            let _ = self.set_active(ContainerType::Normal.into());
        }
    }

    fn is_focalized(&self) -> bool {
        self.is_active(ContainerType::Focalized.into())
    }
}

impl From<ContainerType> for String {
    fn from(val: ContainerType) -> Self {
        match val {
            ContainerType::Normal => String::from("Normal"),
            ContainerType::Focalized => String::from("Focalized"),
        }
    }
}

impl Error {
    pub fn is_warn(&self) -> bool {
        matches!(self, Error::WindowAlreadyAdded | Error::NoWindowsInfo)
    }

    pub fn require_refresh(&self) -> bool {
        matches!(self, Error::ContainerNotFound { refresh: true })
    }
}
