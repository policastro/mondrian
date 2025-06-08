use super::overlay_manager::OverlaysManagerEnum;
use super::overlay_manager::OverlaysManagerTrait;
use crate::win32::api::window::get_foreground_window;
use crate::win32::callbacks::win_event_hook::WindowsEvent;
use crate::win32::win_events_manager::WinEventHandler;
use std::sync::Arc;
use std::sync::Mutex;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_FOCUS;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_LOCATIONCHANGE;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_REORDER;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_FOREGROUND;

pub struct OverlayEventHandler {
    overlays: Arc<Mutex<OverlaysManagerEnum>>,
}

impl OverlayEventHandler {
    pub fn new(overlays: Arc<Mutex<OverlaysManagerEnum>>) -> OverlayEventHandler {
        OverlayEventHandler { overlays }
    }
}

impl WinEventHandler for OverlayEventHandler {
    fn init(&mut self) {}

    fn handle(&mut self, event: &WindowsEvent) {
        if event.hwnd.is_invalid() {
            return;
        }

        if event.event == EVENT_OBJECT_LOCATIONCHANGE {
            if let Ok(mut om) = self.overlays.try_lock() {
                om.reposition(event.hwnd.into());
            }
        } else if event.event == EVENT_SYSTEM_FOREGROUND || event.event == EVENT_OBJECT_FOCUS {
            let fg = get_foreground_window();
            if let Some(fg) = fg.filter(|fg| *fg == event.hwnd) {
                self.overlays.lock().unwrap().focus(fg.into());
            }
        } else if event.event == EVENT_OBJECT_REORDER {
            self.overlays.lock().unwrap().reposition_all();
        }
    }

    fn get_managed_events(&self) -> Option<Vec<u32>> {
        Some(vec![
            EVENT_OBJECT_LOCATIONCHANGE,
            EVENT_OBJECT_REORDER,
            EVENT_SYSTEM_FOREGROUND,
            EVENT_OBJECT_FOCUS,
        ])
    }
}
