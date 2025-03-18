use super::filter::skip_window;
use crate::app::mondrian_message::MondrianMessage;
use crate::app::mondrian_message::WindowEvent;
use crate::app::structs::win_matcher::WinMatcher;
use crate::win32::api::window::is_user_manageable_window;
use crate::win32::callbacks::win_event_hook::WindowsEvent;
use crate::win32::win_events_manager::WinEventHandler;
use std::sync::mpsc::Sender;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_MINIMIZEEND;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_MINIMIZESTART;

pub struct MinimizeEventHandler {
    sender: Sender<MondrianMessage>,
    filter: WinMatcher,
}

impl MinimizeEventHandler {
    pub fn new(sender: Sender<MondrianMessage>, filter: WinMatcher) -> MinimizeEventHandler {
        MinimizeEventHandler { sender, filter }
    }
}

impl WinEventHandler for MinimizeEventHandler {
    fn init(&mut self) {}

    fn handle(&mut self, event: &WindowsEvent) {
        if !is_user_manageable_window(event.hwnd, false, false, true) {
            return;
        }

        let win_event = match event.event {
            EVENT_SYSTEM_MINIMIZESTART => WindowEvent::Minimized(event.hwnd.into()),
            EVENT_SYSTEM_MINIMIZEEND => WindowEvent::Restored(event.hwnd.into()),
            _ => return,
        };

        if skip_window(&win_event, &self.filter) {
            return;
        }

        match self.sender.send(win_event.into()) {
            Ok(_) => (),
            Err(err) => log::warn!("Failed to send event min/max: {}", err),
        }
    }

    fn get_managed_events(&self) -> Option<Vec<u32>> {
        vec![EVENT_SYSTEM_MINIMIZESTART, EVENT_SYSTEM_MINIMIZEEND].into()
    }
}
