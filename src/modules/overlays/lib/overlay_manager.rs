use super::overlay::Overlay;
use super::overlay::OverlayParams;
use super::utils::overlay::OverlayBase;
use crate::win32::api::window::get_foreground_window;
use std::collections::HashMap;
use windows::Win32::Foundation::HWND;

struct OverlayEntry {
    overlay: Overlay<OverlayParams>,
    suspended: bool,
}
impl OverlayEntry {
    fn new(overlay: Overlay<OverlayParams>) -> OverlayEntry {
        OverlayEntry {
            overlay,
            suspended: false,
        }
    }
}

pub struct OverlaysManager {
    overlays: HashMap<isize, OverlayEntry>,
    active_params: OverlayParams,
    inactive_params: OverlayParams,
    custom_active_params: HashMap<isize, OverlayParams>,
    class_name: String,
}

impl OverlaysManager {
    pub fn new(active: Option<OverlayParams>, inactive: Option<OverlayParams>, class_name: &str) -> OverlaysManager {
        OverlaysManager {
            overlays: HashMap::new(),
            active_params: active.unwrap_or(OverlayParams::empty()),
            inactive_params: inactive.unwrap_or(OverlayParams::empty()),
            custom_active_params: HashMap::new(),
            class_name: class_name.to_string(),
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
            let overlay_entry = OverlayEntry::new(Overlay::new(hwnd, &self.class_name.clone()));
            let entry = self.overlays.entry(*w).or_insert_with(|| overlay_entry);

            if entry.suspended {
                return;
            }

            let p = match is_foreground {
                true => *self.custom_active_params.get(w).unwrap_or(&self.active_params),
                false => self.inactive_params,
            };

            Self::reposition_or_create(&mut entry.overlay, p);
        });

        self.overlays.retain(|w, _| windows.contains_key(w));
    }

    pub fn focus(&mut self, hwnd: HWND) {
        if let Some(focus_e) = self.overlays.get_mut(&hwnd.0).filter(|o| !o.suspended) {
            let params = *self.custom_active_params.get(&hwnd.0).unwrap_or(&self.active_params);
            focus_e.overlay.configure(params);

            self.overlays.iter_mut().filter(|o| o.0 != &hwnd.0).for_each(|(_, e)| {
                e.overlay.configure(self.inactive_params);
            });
        }
    }

    pub fn reposition(&mut self, hwnd: HWND) {
        if let Some(e) = self.overlays.get_mut(&hwnd.0).filter(|o| !o.suspended) {
            e.overlay.reposition(None);
        };
    }

    pub fn suspend(&mut self, window: HWND) {
        if let Some(e) = self.overlays.get_mut(&window.0).filter(|o| !o.suspended) {
            e.suspended = true;
            e.overlay.hide();
        }
    }

    pub fn resume(&mut self, window: HWND) {
        if let Some(e) = self.overlays.get_mut(&window.0).filter(|o| o.suspended) {
            let foreground = get_foreground_window().unwrap_or_default().0;
            let (active, inactive) = (self.active_params, self.inactive_params);
            let p = match foreground == window.0 {
                true => *self.custom_active_params.get(&window.0).unwrap_or(&active),
                false => inactive,
            };
            e.suspended = false;
            Self::reposition_or_create(&mut e.overlay, p);
        }
    }

    pub fn resume_all(&mut self) {
        let foreground = get_foreground_window().unwrap_or_default().0;
        let (active, inactive) = (self.active_params, self.inactive_params);
        self.overlays.iter_mut().filter(|o| o.1.suspended).for_each(|e| {
            let p = match foreground == *e.0 {
                true => *self.custom_active_params.get(e.0).unwrap_or(&active),
                false => inactive,
            };
            e.1.suspended = false;
            Self::reposition_or_create(&mut e.1.overlay, p);
        });
    }

    pub fn destroy(&mut self) {
        self.custom_active_params.clear();
        self.overlays.clear();
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
