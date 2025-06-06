use super::filter::skip_window;
use crate::app::mondrian_message::{MondrianMessage, WindowEvent};
use crate::app::structs::win_matcher::WinMatcher;
use crate::win32::api::window::{has_child_window_style, is_fullscreen, is_maximized, is_user_manageable_window};
use crate::win32::callbacks::win_event_hook::WindowsEvent;
use crate::win32::win_events_manager::WinEventHandler;
use crate::win32::window::window_ref::WindowRef;
use crossbeam_channel::Sender;
use std::collections::HashSet;
use windows::Win32::UI::WindowsAndMessaging::EVENT_OBJECT_LOCATIONCHANGE;

pub struct MaximizeEventHandler {
    sender: Sender<MondrianMessage>,
    filter: WinMatcher,
    maximized_wins: HashSet<WindowRef>,
}

impl MaximizeEventHandler {
    pub fn new(sender: Sender<MondrianMessage>, filter: WinMatcher) -> MaximizeEventHandler {
        MaximizeEventHandler {
            sender,
            filter,
            maximized_wins: HashSet::new(),
        }
    }
}

impl WinEventHandler for MaximizeEventHandler {
    fn init(&mut self) {}

    fn handle(&mut self, event: &WindowsEvent) {
        if event.hwnd.is_invalid() {
            return;
        }

        let contained = self.maximized_wins.contains(&event.hwnd.into());
        let is_maximized = is_maximized(event.hwnd);
        let is_fullscreen = if is_maximized { false } else { is_fullscreen(event.hwnd) };
        let is_max_full = is_maximized || is_fullscreen;
        if !contained && is_max_full {
            let is_managed = is_user_manageable_window(event.hwnd, true, true, !is_fullscreen);
            let is_managed = match is_fullscreen {
                false => is_managed,
                true => is_managed && !has_child_window_style(event.hwnd),
            };
            if is_managed {
                let app_event = WindowEvent::Maximized(event.hwnd.into());
                if skip_window(&app_event, &self.filter) {
                    return;
                }
                self.maximized_wins.insert(event.hwnd.into());
                self.sender.send(app_event.into()).expect("Failed to send event close");
            }
        } else if contained && !is_max_full {
            self.maximized_wins.remove(&event.hwnd.into());
            let win_event = WindowEvent::Unmaximized(event.hwnd.into());
            self.sender.send(win_event.into()).expect("Failed to send event close");
        }
    }

    fn get_managed_events(&self) -> Option<Vec<u32>> {
        vec![EVENT_OBJECT_LOCATIONCHANGE].into()
    }
}
