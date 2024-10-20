use std::sync::{Arc, Mutex};

use windows::Win32::UI::WindowsAndMessaging::{EVENT_OBJECT_LOCATIONCHANGE, EVENT_SYSTEM_FOREGROUND};

use crate::win32::{
    api::window::get_foreground_window, callbacks::win_event_hook::WinEvent, win_events_manager::WinEventHandler,
};

use super::overlay_manager::OverlaysManager;

pub struct OverlayEventHandler {
    overlays: Arc<Mutex<OverlaysManager>>,
}

impl OverlayEventHandler {
    pub fn new(overlays: Arc<Mutex<OverlaysManager>>) -> OverlayEventHandler {
        OverlayEventHandler { overlays }
    }
}

impl WinEventHandler for OverlayEventHandler {
    fn init(&mut self) {}

    fn handle(&mut self, event: &WinEvent) {
        if event.hwnd.0 == 0 {
            return;
        }

        if event.event == EVENT_OBJECT_LOCATIONCHANGE {
            self.overlays.lock().unwrap().move_overlay(event.hwnd);
        } else if event.event == EVENT_SYSTEM_FOREGROUND && get_foreground_window().is_some_and(|f| f.0 == event.hwnd.0)
        {
            self.overlays.lock().unwrap().focus(event.hwnd);
        }
    }

    fn get_managed_events(&self) -> Option<Vec<u32>> {
        Some(vec![EVENT_OBJECT_LOCATIONCHANGE, EVENT_SYSTEM_FOREGROUND])
    }
}
