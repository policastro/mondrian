use std::collections::{HashMap, HashSet};

use windows::Win32::Foundation::HWND;

use crate::win32::api::window::get_foreground_window;

use super::overlay::{Overlay, OverlayParams};

pub struct OverlaysManager {
    overlays: HashMap<isize, Overlay>,
    active: OverlayParams,
    inactive: OverlayParams,
    locked: bool,
}
impl OverlaysManager {
    pub fn new(active: Option<OverlayParams>, inactive: Option<OverlayParams>) -> OverlaysManager {
        OverlaysManager {
            overlays: HashMap::new(),
            active: active.unwrap_or(OverlayParams::empty()),
            inactive: inactive.unwrap_or(OverlayParams::empty()),
            locked: false,
        }
    }

    pub fn rebuild(&mut self, windows: &HashSet<isize>) {
        let foreground = get_foreground_window().unwrap_or_default();

        windows.iter().for_each(|w| {
            let hwnd = HWND(*w);
            let is_foreground = hwnd == foreground;
            let o = self
                .overlays
                .entry(*w)
                .or_insert_with(|| Overlay::new(hwnd, self.active, self.inactive));

            o.reposition(Some(is_foreground));
        });

        self.overlays.retain(|w, _| windows.contains(w));
    }

    pub fn focus(&mut self, hwnd: HWND) {
        if self.locked {
            return;
        }
        if self.overlays.contains_key(&hwnd.0) {
            self.overlays.iter_mut().for_each(|(w, o)| o.activate(*w == hwnd.0));
        }
    }

    pub fn move_overlay(&mut self, hwnd: HWND) {
        if self.locked {
            return;
        }

        if let Some(o) = self.overlays.get_mut(&hwnd.0) {
            o.reposition(None)
        };
    }

    pub fn lock(&mut self) {
        self.locked = true;
    }

    pub fn unlock(&mut self) {
        self.locked = false;
    }

    pub fn suspend(&mut self) {
        self.lock();
        self.overlays.iter_mut().for_each(|(_, o)| o.hide());
    }

    pub fn resume(&mut self) {
        self.overlays.iter_mut().for_each(|(_, o)| o.reposition(None));
        self.unlock();
    }

    pub fn destroy(&mut self) {
        self.overlays.clear();
    }
}

impl Drop for OverlaysManager {
    fn drop(&mut self) {
        self.destroy();
    }
}
