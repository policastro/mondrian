use std::sync::{Arc, Mutex};

use windows::Win32::UI::WindowsAndMessaging::{
    EVENT_OBJECT_LOCATIONCHANGE, EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MOVESIZESTART,
};

use crate::win32::{
    api::window::get_foreground_window, callbacks::win_event_hook::WindowsEvent, win_events_manager::WinEventHandler,
};

use super::overlay_manager::OverlaysManager;

pub struct OverlayEventHandler {
    overlays: Arc<Mutex<OverlaysManager>>,
    update_while_resizing: bool,
    moving: bool,
}

impl OverlayEventHandler {
    pub fn new(overlays: Arc<Mutex<OverlaysManager>>, update_while_resizing: bool) -> OverlayEventHandler {
        OverlayEventHandler {
            overlays,
            update_while_resizing,
            moving: false,
        }
    }
}

impl WinEventHandler for OverlayEventHandler {
    fn init(&mut self) {}

    fn handle(&mut self, event: &WindowsEvent) {
        if event.hwnd.0 == 0 {
            return;
        }

        if event.event == EVENT_OBJECT_LOCATIONCHANGE && !self.moving {
            self.overlays.lock().unwrap().move_overlay(event.hwnd);
        } else if event.event == EVENT_SYSTEM_FOREGROUND && get_foreground_window().is_some_and(|f| f.0 == event.hwnd.0)
        {
            self.overlays.lock().unwrap().focus(event.hwnd);
        } else if event.event == EVENT_SYSTEM_MOVESIZESTART {
            self.moving = !self.update_while_resizing;
        } else if event.event == EVENT_SYSTEM_MOVESIZEEND {
            self.moving = false;
        }
    }

    fn get_managed_events(&self) -> Option<Vec<u32>> {
        if self.update_while_resizing {
            Some(vec![EVENT_OBJECT_LOCATIONCHANGE, EVENT_SYSTEM_FOREGROUND])
        } else {
            Some(vec![
                EVENT_OBJECT_LOCATIONCHANGE,
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_MOVESIZESTART,
                EVENT_SYSTEM_MOVESIZEEND,
            ])
        }
    }
}
