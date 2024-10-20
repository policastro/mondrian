use std::sync::mpsc::Sender;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_MINIMIZEEND;

use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_MINIMIZESTART;

use crate::app::tiles_manager::tm_command::TMCommand;
use crate::win32::api::window::is_user_managable_window;
use crate::win32::callbacks::win_event_hook::WinEvent;
use crate::win32::win_events_manager::WinEventHandler;

pub struct MinimizeEventHandler {
    sender: Sender<TMCommand>,
}

impl MinimizeEventHandler {
    pub fn new(sender: Sender<TMCommand>) -> MinimizeEventHandler {
        MinimizeEventHandler { sender }
    }
}

impl WinEventHandler for MinimizeEventHandler {
    fn init(&mut self) {}

    fn handle(&mut self, event: &WinEvent) {
        if !is_user_managable_window(event.hwnd, false, false, true) {
            return;
        }

        let app_event = match event.event {
            EVENT_SYSTEM_MINIMIZESTART => TMCommand::WindowMinimized(event.hwnd),
            EVENT_SYSTEM_MINIMIZEEND => TMCommand::WindowRestored(event.hwnd),
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
