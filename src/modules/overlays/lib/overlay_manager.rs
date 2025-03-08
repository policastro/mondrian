use super::overlay::Overlay;
use super::overlay::OverlayParams;
use super::utils::overlay::OverlayBase;
use crate::win32::api::window::get_foreground_window;
use std::collections::HashMap;
use windows::Win32::Foundation::HWND;

pub struct OverlaysManager {
    overlays: HashMap<isize, Overlay<OverlayParams>>,
    active_params: OverlayParams,
    inactive_params: OverlayParams,
    custom_active_params: HashMap<isize, OverlayParams>,
    class_name: String,
    locked: bool,
}

impl OverlaysManager {
    pub fn new(active: Option<OverlayParams>, inactive: Option<OverlayParams>, class_name: &str) -> OverlaysManager {
        OverlaysManager {
            overlays: HashMap::new(),
            active_params: active.unwrap_or(OverlayParams::empty()),
            inactive_params: inactive.unwrap_or(OverlayParams::empty()),
            custom_active_params: HashMap::new(),
            class_name: class_name.to_string(),
            locked: false,
        }
    }

    pub fn rebuild(&mut self, windows: &HashMap<isize, Option<OverlayParams>>) {
        let foreground = get_foreground_window().unwrap_or_default();

        self.custom_active_params = windows
            .iter()
            .filter_map(|(w, params)| params.as_ref().map(|p| (*w, *p)))
            .collect();

        windows.keys().for_each(|w| {
            let hwnd = HWND(*w);
            let is_foreground = hwnd == foreground;
            let o = self
                .overlays
                .entry(*w)
                .or_insert_with(|| Overlay::new(hwnd, &self.class_name.clone()));

            if !self.locked {
                let p = match is_foreground {
                    true => *self.custom_active_params.get(w).unwrap_or(&self.active_params),
                    false => self.inactive_params,
                };
                Self::reposition_or_create(o, p);
            }
        });

        self.overlays.retain(|w, _| windows.contains_key(w));
    }

    pub fn focus(&mut self, hwnd: HWND) {
        if self.locked {
            return;
        }

        if self.overlays.contains_key(&hwnd.0) {
            self.overlays.iter_mut().for_each(|(w, o)| {
                let p = match *w == hwnd.0 {
                    true => *self.custom_active_params.get(w).unwrap_or(&self.active_params),
                    false => self.inactive_params,
                };
                o.configure(p);
            });
        }
    }

    pub fn reposition(&mut self, hwnd: HWND) {
        if self.locked {
            return;
        }

        if let Some(o) = self.overlays.get_mut(&hwnd.0) {
            o.reposition(None);
        };
    }

    pub fn suspend(&mut self) {
        self.lock();
        self.overlays.iter_mut().for_each(|(_, o)| o.hide());
    }

    pub fn resume(&mut self) {
        let foreground = get_foreground_window().unwrap_or_default().0;
        let (active, inactive) = (self.active_params, self.inactive_params);
        self.overlays.iter_mut().for_each(|(w, o)| {
            let p = match foreground == *w {
                true => *self.custom_active_params.get(w).unwrap_or(&active),
                false => inactive,
            };
            Self::reposition_or_create(o, p);
        });
        self.unlock();
    }

    pub fn destroy(&mut self) {
        self.custom_active_params.clear();
        self.overlays.clear();
    }

    fn lock(&mut self) {
        self.locked = true;
    }

    fn unlock(&mut self) {
        self.locked = false;
    }

    fn reposition_or_create(overlay: &mut Overlay<OverlayParams>, params: OverlayParams) {
        match overlay.exists() {
            true => {
                overlay.reposition(Some(params));
                overlay.show();
            }
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
