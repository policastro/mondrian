use std::sync::mpsc::Sender;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_MINIMIZEEND;

use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_MINIMIZESTART;

use crate::app::win32_event::Win32Event;
use crate::win32::api::window::is_user_managable_window;
use crate::win32::win_events_manager::{WinEvent, WinEventHandler};

pub struct MinimizeEventHandler {
    sender: Sender<Win32Event>,
}

impl MinimizeEventHandler {
    pub fn new(sender: Sender<Win32Event>) -> MinimizeEventHandler {
        MinimizeEventHandler { sender }
    }
}

impl WinEventHandler for MinimizeEventHandler {
    fn init(&mut self) {}

    fn handle(&mut self, event: &WinEvent) {
        if !is_user_managable_window(event.hwnd, false, false) {
            return;
        }

        let app_event = match event.event {
            EVENT_SYSTEM_MINIMIZESTART => Win32Event::WindowMinimized(event.hwnd),
            EVENT_SYSTEM_MINIMIZEEND => Win32Event::WindowRestored(event.hwnd),
            _ => return,
        };

        match self.sender.send(app_event) {
            Ok(_) => (),
            Err(err) => log::error!("Failed to send event min/max: {}", err),
        }
    }

    fn get_managed_events(&self) -> Option<Vec<u32>> {
        vec![EVENT_SYSTEM_MINIMIZESTART, EVENT_SYSTEM_MINIMIZEEND].into()
    }
}
