use super::filter::skip_window;
use crate::app::mondrian_message::{MondrianMessage, WindowEvent};
use crate::app::structs::win_matcher::WinMatcher;
use crate::win32::api::window::{enum_user_manageable_windows, is_user_manageable_window, is_window_visible};
use crate::win32::callbacks::win_event_hook::WindowsEvent;
use crate::win32::win_events_manager::WinEventHandler;
use std::collections::HashSet;
use std::sync::mpsc::Sender;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    EVENT_OBJECT_CLOAKED, EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE, EVENT_OBJECT_SHOW, EVENT_OBJECT_UNCLOAKED,
    EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MINIMIZEEND,
};

pub struct OpenCloseEventHandler {
    sender: Sender<MondrianMessage>,
    filter: WinMatcher,
    windows: HashSet<isize>,
}

impl OpenCloseEventHandler {
    pub fn new(sender: Sender<MondrianMessage>, filter: WinMatcher) -> OpenCloseEventHandler {
        OpenCloseEventHandler {
            sender,
            filter,
            windows: HashSet::new(),
        }
    }
}

impl OpenCloseEventHandler {
    fn send_open_event(&mut self, hwnd: HWND) {
        let is_managed = is_user_manageable_window(hwnd, true, true, true);

        if is_managed && self.windows.insert(hwnd.0) {
            let win_event = WindowEvent::Opened(hwnd);
            if skip_window(&win_event, &self.filter) {
                return;
            }

            self.sender
                .send(win_event.into())
                .inspect_err(|_| log::warn!("Failed to send open event for window {}", hwnd.0))
                .ok();
        }
    }
}

impl WinEventHandler for OpenCloseEventHandler {
    fn init(&mut self) {
        self.windows = enum_user_manageable_windows()
            .into_iter()
            .filter(|w| is_user_manageable_window(w.hwnd, true, true, true))
            .map(|w| w.hwnd.0)
            .collect();
    }

    fn handle(&mut self, event: &WindowsEvent) {
        if event.hwnd.0 == 0 {
            return;
        }

        if event.event == EVENT_SYSTEM_FOREGROUND
            || event.event == EVENT_OBJECT_UNCLOAKED
            || event.event == EVENT_OBJECT_SHOW
        {
            self.send_open_event(event.hwnd);
            return;
        }

        if event.event == EVENT_SYSTEM_MINIMIZEEND {
            let is_managed = is_user_manageable_window(event.hwnd, true, true, true);
            if is_managed && !skip_window(&WindowEvent::Minimized(event.hwnd), &self.filter) {
                self.windows.insert(event.hwnd.0);
            }
            return;
        }

        if !self.windows.contains(&event.hwnd.0) {
            return;
        }

        let (is_cloaked, is_destroyed, is_hidden) = (
            event.event == EVENT_OBJECT_CLOAKED,
            event.event == EVENT_OBJECT_DESTROY,
            event.event == EVENT_OBJECT_HIDE,
        );

        if is_cloaked || ((is_destroyed || is_hidden) && !is_window_visible(event.hwnd)) {
            self.windows.remove(&event.hwnd.0);
            self.sender
                .send(WindowEvent::Closed(event.hwnd).into())
                .inspect_err(|e| log::warn!("Failed to send close event: {:?}", e))
                .ok();
        }
    }

    fn get_managed_events(&self) -> Option<Vec<u32>> {
        vec![
            EVENT_SYSTEM_FOREGROUND,
            EVENT_OBJECT_CLOAKED,
            EVENT_OBJECT_SHOW,
            EVENT_SYSTEM_MINIMIZEEND,
            EVENT_OBJECT_DESTROY,
            EVENT_OBJECT_UNCLOAKED,
            EVENT_OBJECT_HIDE,
        ]
        .into()
    }
}
