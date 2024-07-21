use std::sync::{Arc, Mutex};

use windows::Win32::UI::WindowsAndMessaging::{
    EVENT_OBJECT_CLOAKED, EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE, EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_UNCLOAKED,
    EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MINIMIZESTART, EVENT_SYSTEM_MOVESIZEEND,
    EVENT_SYSTEM_MOVESIZESTART,
};

use crate::win32::win_events_manager::{WinEvent, WinEventHandler};

use super::overlay_manager::OverlaysManager;

pub struct OverlayEventHandler {
    overlays: Arc<Mutex<OverlaysManager>>,
    follow_movements: bool,
    moving: bool,
}

impl OverlayEventHandler {
    pub fn new(overlays: Arc<Mutex<OverlaysManager>>, follow_movements: bool) -> OverlayEventHandler {
        OverlayEventHandler {
            moving: false,
            overlays,
            follow_movements,
        }
    }
}

impl WinEventHandler for OverlayEventHandler {
    fn init(&mut self) {}

    fn handle(&mut self, event: &WinEvent) {
        if event.event == EVENT_SYSTEM_FOREGROUND {
            self.overlays.lock().unwrap().focus(event.hwnd);
        }

        if event.event == EVENT_SYSTEM_MOVESIZESTART {
            self.moving = true;
        }

        if event.event == EVENT_SYSTEM_MOVESIZEEND {
            self.moving = false;
        }

        if self.moving && self.follow_movements && event.event == EVENT_OBJECT_LOCATIONCHANGE {
            self.overlays.lock().unwrap().move_overlay(event.hwnd);
        }
    }

    fn get_managed_events(&self) -> Option<Vec<u32>> {
        Some(vec![
            EVENT_OBJECT_LOCATIONCHANGE,
            EVENT_OBJECT_UNCLOAKED,
            EVENT_OBJECT_CLOAKED,
            EVENT_OBJECT_DESTROY,
            EVENT_OBJECT_HIDE,
            EVENT_SYSTEM_MOVESIZEEND,
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_MINIMIZESTART,
            EVENT_SYSTEM_MINIMIZEEND,
            EVENT_SYSTEM_MOVESIZESTART,
        ])
    }
}
