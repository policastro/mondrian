use super::overlay::Overlay;
use super::overlay::OverlayParams;
use crate::win32::api::window::get_foreground_window;
use crate::win32::window::window_ref::WindowRef;
use enum_dispatch::enum_dispatch;
use std::collections::HashMap;

#[enum_dispatch(OverlaysManagerTrait)]
pub enum OverlaysManagerEnum {
    MultiOverlaysManager,
    MonoOverlaysManager,
}

#[enum_dispatch]
pub trait OverlaysManagerTrait {
    fn rebuild(&mut self, windows: &HashMap<WindowRef, Option<OverlayParams>>);
    fn focus(&mut self, window: WindowRef);
    fn reposition(&mut self, window: WindowRef);
    fn suspend(&mut self, window: WindowRef);
    fn resume(&mut self, window: WindowRef);
    fn resume_all(&mut self);
}

struct OverlayEntry {
    overlay: Overlay,
    suspended: bool,
}

impl OverlayEntry {
    fn new(overlay: Overlay) -> OverlayEntry {
        OverlayEntry {
            overlay,
            suspended: false,
        }
    }
}

pub struct MonoOverlaysManager {
    last_foreground: Option<WindowRef>,
    overlays: HashMap<WindowRef, OverlayEntry>,
    default_params: OverlayParams,
    params_map: HashMap<WindowRef, OverlayParams>,
    class_name: String,
}

impl MonoOverlaysManager {
    pub fn new(default_params: OverlayParams, class_name: &str) -> MonoOverlaysManager {
        MonoOverlaysManager {
            last_foreground: None,
            overlays: HashMap::new(),
            default_params,
            params_map: HashMap::new(),
            class_name: class_name.to_string(),
        }
    }
}

impl OverlaysManagerTrait for MonoOverlaysManager {
    fn rebuild(&mut self, windows: &HashMap<WindowRef, Option<OverlayParams>>) {
        self.params_map = windows
            .iter()
            .map(|(w, params)| (*w, params.unwrap_or(self.default_params)))
            .collect();

        windows.keys().for_each(|w| {
            let winref = *w;
            let overlay = Overlay::new(winref, &self.class_name.clone(), Default::default());
            self.overlays
                .entry(winref)
                .or_insert_with(|| OverlayEntry::new(overlay));
        });

        self.overlays.retain(|w, _| windows.contains_key(w));
        self.focus(get_foreground_window().unwrap_or_default().into());
    }

    fn focus(&mut self, window: WindowRef) {
        if let Some(e) = self
            .last_foreground
            .filter(|lf| *lf == window)
            .and_then(|lf| self.overlays.get_mut(&lf))
        {
            update_overlay(e, Some(*self.params_map.get(&window).unwrap()));
            return;
        }

        if let Some(e) = self.last_foreground.and_then(|lf| self.overlays.get_mut(&lf)) {
            e.overlay.hide();
        }

        if let Some(e) = self.overlays.get_mut(&window) {
            update_overlay(e, Some(*self.params_map.get(&window).unwrap()));
            self.last_foreground = Some(window);
            if !e.suspended {
                e.overlay.show();
            }
        } else {
            self.last_foreground = None;
        }
    }

    fn reposition(&mut self, window: WindowRef) {
        if let Some(e) = self.overlays.get_mut(&window).filter(|o| !o.suspended) {
            update_overlay(e, None);
        }
    }

    fn suspend(&mut self, window: WindowRef) {
        if let Some(e) = self.overlays.get_mut(&window).filter(|o| !o.suspended) {
            e.suspended = true;
            e.overlay.hide();
        }
    }

    fn resume(&mut self, window: WindowRef) {
        if let Some(e) = self.overlays.get_mut(&window).filter(|o| o.suspended) {
            e.suspended = false;
            if self.last_foreground == Some(window) {
                update_overlay(e, Some(*self.params_map.get(&window).unwrap()));
                e.overlay.show();
            }
        }
    }

    fn resume_all(&mut self) {
        for (w, o) in self.overlays.iter_mut().filter(|(_, o)| o.suspended) {
            o.suspended = false;
            if self.last_foreground == Some(*w) {
                update_overlay(o, Some(*self.params_map.get(w).unwrap()));
                o.overlay.show();
            }
        }
    }
}

pub struct MultiOverlaysManager {
    last_foreground: Option<WindowRef>,
    overlays: HashMap<WindowRef, OverlayEntry>,
    default_active_params: OverlayParams,
    inactive_params: OverlayParams,
    active_params_map: HashMap<WindowRef, OverlayParams>,
    class_name: String,
}

impl MultiOverlaysManager {
    pub fn new(
        default_active_params: OverlayParams,
        inactive_params: OverlayParams,
        class_name: &str,
    ) -> MultiOverlaysManager {
        MultiOverlaysManager {
            last_foreground: None,
            overlays: HashMap::new(),
            default_active_params,
            inactive_params,
            active_params_map: HashMap::new(),
            class_name: class_name.to_string(),
        }
    }
}

impl OverlaysManagerTrait for MultiOverlaysManager {
    fn rebuild(&mut self, windows: &HashMap<WindowRef, Option<OverlayParams>>) {
        let foreground = get_foreground_window().unwrap_or_default().into();

        self.active_params_map = windows
            .iter()
            .map(|(w, params)| (*w, params.unwrap_or(self.default_active_params)))
            .collect();

        windows.keys().for_each(|w| {
            let overlay = Overlay::new(*w, &self.class_name.clone(), Default::default());
            let entry = self.overlays.entry(*w).or_insert_with(|| OverlayEntry::new(overlay));

            let p = if *w == foreground {
                self.last_foreground = Some(*w);
                self.active_params_map.get(w).copied().unwrap()
            } else {
                self.inactive_params
            };

            update_overlay(entry, Some(p));
        });

        self.overlays.retain(|w, _| windows.contains_key(w));
    }

    fn focus(&mut self, window: WindowRef) {
        if self.last_foreground.is_some_and(|lf| lf == window) {
            if let Some(e) = self.last_foreground.and_then(|lf| self.overlays.get_mut(&lf)) {
                update_overlay(e, Some(*self.active_params_map.get(&window).unwrap()));
            }
            return;
        }

        if let Some(e) = self.last_foreground.and_then(|lf| self.overlays.get_mut(&lf)) {
            update_overlay(e, Some(self.inactive_params));
        }

        if let Some(focus_entry) = self.overlays.get_mut(&window) {
            update_overlay(focus_entry, Some(*self.active_params_map.get(&window).unwrap()));
            self.last_foreground = Some(window);
        } else {
            self.last_foreground = None;
        }
    }

    fn reposition(&mut self, window: WindowRef) {
        if let Some(e) = self.overlays.get_mut(&window).filter(|o| !o.suspended) {
            update_overlay(e, None);
        };
    }

    fn suspend(&mut self, window: WindowRef) {
        if let Some(e) = self.overlays.get_mut(&window).filter(|o| !o.suspended) {
            e.suspended = true;
            e.overlay.hide();
        }
    }

    fn resume(&mut self, window: WindowRef) {
        if let Some(e) = self.overlays.get_mut(&window).filter(|o| o.suspended) {
            e.suspended = false;
            update_overlay(e, None);
            e.overlay.show();
        }
    }

    fn resume_all(&mut self) {
        self.overlays.values_mut().filter(|e| e.suspended).for_each(|e| {
            e.suspended = false;
            update_overlay(e, None);
            e.overlay.show();
        });
    }
}

fn update_overlay(overlay_entry: &mut OverlayEntry, new_params: Option<OverlayParams>) {
    let overlay = &mut overlay_entry.overlay;
    if overlay.exists() {
        if !new_params.is_some_and(|p| overlay.configure(p)) {
            overlay.reposition();
        }
    } else {
        overlay.create(new_params);
    }
}
