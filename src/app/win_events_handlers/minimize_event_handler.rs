use std::sync::mpsc::Sender;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_MINIMIZEEND;

use windows::Win32::
    UI::
        WindowsAndMessaging::
            EVENT_SYSTEM_MINIMIZESTART
        
    
;

use crate::app::app_event::AppEvent;
use crate::win32utils::api::window::is_user_managable_window;
use crate::win32utils::win_events_manager::{WinEvent, WinEventHandler};

pub struct MinimizeEventHandler {
    sender: Sender<AppEvent>,
}

impl MinimizeEventHandler {
    pub fn new(sender: Sender<AppEvent>) -> MinimizeEventHandler {
        MinimizeEventHandler { sender }
    }
}

impl WinEventHandler for MinimizeEventHandler {
    fn handle(&mut self, event: &WinEvent) {
        if !is_user_managable_window(event.hwnd, false, false) {
            return;
        }

        let app_event = match event.event {
            EVENT_SYSTEM_MINIMIZESTART => AppEvent::WindowMinimized(event.hwnd),
            EVENT_SYSTEM_MINIMIZEEND => AppEvent::WindowRestored(event.hwnd),
            _ => return,
        };

        match self.sender.send(app_event) {
            Ok(_) => (),
            Err(err) => log::error!("Failed to send event min/max: {}", err),
        }
    }
}
