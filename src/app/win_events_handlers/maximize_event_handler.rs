use std::collections::HashSet;
use std::sync::mpsc::Sender;

use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_LOCATIONCHANGE;

use crate::app::tiles_manager::tm_command::TMCommand;
use crate::win32::api::window::{is_maximized, is_user_managable_window, is_window_visible};
use crate::win32::win_events_manager::{WinEvent, WinEventHandler};

pub struct MaximizeEventHandler {
    sender: Sender<TMCommand>,
    maximized_wins: HashSet<isize>,
}

impl MaximizeEventHandler {
    pub fn new(sender: Sender<TMCommand>) -> MaximizeEventHandler {
        MaximizeEventHandler {
            sender,
            maximized_wins: HashSet::new(),
        }
    }
}

impl WinEventHandler for MaximizeEventHandler {
    fn init(&mut self) {}

    fn handle(&mut self, event: &WinEvent) {
        let contained = self.maximized_wins.contains(&event.hwnd.0);
        if !contained && is_maximized(event.hwnd) {
            let is_managed = is_user_managable_window(event.hwnd, false, true); // TODO Can I check visibility here?
            let is_managed = is_managed && is_window_visible(event.hwnd);
            if is_managed {
                self.maximized_wins.insert(event.hwnd.0);
                self.sender
                    .send(TMCommand::WindowClosed(event.hwnd))
                    .expect("Failed to send event close");
            }
        } else if contained && !is_maximized(event.hwnd) {
            self.maximized_wins.remove(&event.hwnd.0);
            self.sender
                .send(TMCommand::WindowOpened(event.hwnd))
                .expect("Failed to send event close");
        }
    }

    fn get_managed_events(&self) -> Option<Vec<u32>> {
        vec![EVENT_OBJECT_LOCATIONCHANGE].into()
    }
}
