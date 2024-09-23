use std::collections::HashSet;
use std::sync::mpsc::Sender;

use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_LOCATIONCHANGE;

use crate::app::tiles_manager::tm_command::TMCommand;
use crate::win32::api::window::{has_child_window_style, is_fullscreen, is_maximized, is_user_managable_window};
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
        if event.hwnd.0 == 0 {
            return;
        }

        let contained = self.maximized_wins.contains(&event.hwnd.0);
        let is_maximized = is_maximized(event.hwnd);
        let is_fullscreen = if is_maximized { false } else { is_fullscreen(event.hwnd) };
        let is_max_full = is_maximized || is_fullscreen;
        if !contained && is_max_full {
            let is_managed = is_user_managable_window(event.hwnd, true, true, !is_fullscreen);
            let is_managed = match is_fullscreen {
                false => is_managed,
                true => is_managed && !has_child_window_style(event.hwnd),
            };
            if is_managed {
                self.maximized_wins.insert(event.hwnd.0);
                self.sender
                    .send(TMCommand::WindowMaximized(event.hwnd))
                    .expect("Failed to send event close");
            }
        } else if contained && !is_max_full {
            self.maximized_wins.remove(&event.hwnd.0);
            self.sender
                .send(TMCommand::WindowUnmaximized(event.hwnd))
                .expect("Failed to send event close");
        }
    }

    fn get_managed_events(&self) -> Option<Vec<u32>> {
        vec![EVENT_OBJECT_LOCATIONCHANGE].into()
    }
}
