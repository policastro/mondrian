use std::collections::HashSet;
use std::sync::mpsc::Sender;

use windows::Win32::UI::WindowsAndMessaging::{EVENT_OBJECT_DESTROY, EVENT_SYSTEM_FOREGROUND};

use crate::app::app_event::AppEvent;
use crate::win32utils::api::window::is_user_managable_window;
use crate::win32utils::win_events_manager::{WinEvent, WinEventHandler};

pub struct OpenCloseEventHandler {
    sender: Sender<AppEvent>,
    windows: HashSet<isize>,
}

impl OpenCloseEventHandler {
    pub fn new(sender: Sender<AppEvent>) -> OpenCloseEventHandler {
        OpenCloseEventHandler {
            sender,
            windows: HashSet::new(),
        }
    }
}

impl WinEventHandler for OpenCloseEventHandler {
    fn handle(&mut self, event: &WinEvent) {
        let is_managed_not_iconic = is_user_managable_window(event.hwnd, false, true);

        if (event.event == EVENT_SYSTEM_FOREGROUND) && is_managed_not_iconic {
            self.windows.insert(event.hwnd.0);
            self.sender
                .send(AppEvent::WindowOpened(event.hwnd))
                .expect("Failed to send event open");
        }
        if event.event == EVENT_OBJECT_DESTROY && self.windows.contains(&event.hwnd.0) {
            self.windows.remove(&event.hwnd.0);
            self.sender
                .send(AppEvent::WindowClosed(event.hwnd))
                .expect("Failed to send event close");
        }
    }
}
