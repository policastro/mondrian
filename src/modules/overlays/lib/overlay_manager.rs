use super::overlay::Overlay;
use super::overlay::OverlayParams;
use super::utils::overlay::OverlayBase;
use crate::win32::api::window::get_foreground_window;
use std::collections::HashMap;
use std::collections::HashSet;
use windows::Win32::Foundation::HWND;

pub struct OverlaysManager {
    overlays: HashMap<isize, Overlay<OverlayParams>>,
    active: OverlayParams,
    inactive: OverlayParams,
    class_name: String,
    locked: bool,
}

impl OverlaysManager {
    pub fn new(active: Option<OverlayParams>, inactive: Option<OverlayParams>, class_name: &str) -> OverlaysManager {
        OverlaysManager {
            overlays: HashMap::new(),
            active: active.unwrap_or(OverlayParams::empty()),
            inactive: inactive.unwrap_or(OverlayParams::empty()),
            class_name: class_name.to_string(),
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
                .or_insert_with(|| Overlay::new(Some(hwnd), &self.class_name.clone()));

            if !self.locked {
                let p = if is_foreground { self.active } else { self.inactive };
                Self::reposition_overlay(o, p);
            }
        });

        self.overlays.retain(|w, _| windows.contains(w));
    }

    pub fn focus(&mut self, hwnd: HWND) {
        if self.locked {
            return;
        }

        if self.overlays.contains_key(&hwnd.0) {
            self.overlays.iter_mut().for_each(|(w, o)| {
                let p = if *w == hwnd.0 { self.active } else { self.inactive };
                o.configure(p);
            });
        }
    }

    pub fn move_overlay(&mut self, hwnd: HWND) {
        if self.locked {
            return;
        }

        if let Some(o) = self.overlays.get_mut(&hwnd.0) {
            o.reposition(None);
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
        let foreground = get_foreground_window().unwrap_or_default().0;
        let (active, inactive) = (self.active, self.inactive);
        self.overlays.iter_mut().for_each(|(hwnd, o)| {
            let p = if foreground == *hwnd { active } else { inactive };
            Self::reposition_overlay(o, p);
        });
        self.unlock();
    }

    pub fn destroy(&mut self) {
        self.overlays.clear();
    }

    fn reposition_overlay(overlay: &mut Overlay<OverlayParams>, params: OverlayParams) {
        match overlay.exists() {
            true => overlay.reposition(Some(params)),
            false => overlay.create(params),
        }
    }
}

impl Drop for OverlaysManager {
    fn drop(&mut self) {
        self.destroy();
    }
}

impl OverlayBase for OverlayParams {
    fn get_padding(&self) -> u8 {
        self.padding
    }

    fn get_thickness(&self) -> u8 {
        self.thickness
    }
}
