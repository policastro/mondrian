use std::collections::HashSet;
use std::sync::mpsc::Sender;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    EVENT_OBJECT_CLOAKED, EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE, EVENT_OBJECT_SHOW, EVENT_OBJECT_UNCLOAKED,
    EVENT_SYSTEM_FOREGROUND,
};

use crate::app::win32_event::Win32Event;
use crate::win32::api::window::{enum_user_manageable_windows, is_user_managable_window, is_window_visible};
use crate::win32::win_events_manager::{WinEvent, WinEventHandler};

pub struct OpenCloseEventHandler {
    sender: Sender<Win32Event>,
    windows: HashSet<isize>,
}

impl OpenCloseEventHandler {
    pub fn new(sender: Sender<Win32Event>) -> OpenCloseEventHandler {
        OpenCloseEventHandler {
            sender,
            windows: HashSet::new(),
        }
    }
}

impl OpenCloseEventHandler {
    fn send_open_event(&mut self, hwnd: HWND) {
        let is_managed = is_user_managable_window(hwnd, false, true); // TODO Can I check visibility here?
        let is_managed = is_managed && is_window_visible(hwnd);

        if is_managed && self.windows.insert(hwnd.0) {
            self.sender
                .send(Win32Event::WindowOpened(hwnd))
                .expect("Failed to send event open");
        }
    }
}

impl WinEventHandler for OpenCloseEventHandler {
    fn init(&mut self) {
        enum_user_manageable_windows()
            .iter()
            .for_each(|w| self.send_open_event(w.hwnd));
    }

    fn handle(&mut self, event: &WinEvent) {
        let foreground = event.event == EVENT_SYSTEM_FOREGROUND;
        let is_uncloaked = event.event == EVENT_OBJECT_UNCLOAKED;
        let is_shown = event.event == EVENT_OBJECT_SHOW;

        if foreground || is_uncloaked || is_shown {
            self.send_open_event(event.hwnd);
        }

        if !self.windows.contains(&event.hwnd.0) {
            return;
        }

        if (event.event == EVENT_OBJECT_DESTROY || event.event == EVENT_OBJECT_CLOAKED)
            || (event.event == EVENT_OBJECT_HIDE && !is_window_visible(event.hwnd))
        {
            self.windows.remove(&event.hwnd.0);
            self.sender
                .send(Win32Event::WindowClosed(event.hwnd))
                .expect("Failed to send event close");
        }
    }

    fn get_managed_events(&self) -> Option<Vec<u32>> {
        vec![
            EVENT_SYSTEM_FOREGROUND,
            EVENT_OBJECT_CLOAKED,
            EVENT_OBJECT_SHOW,
            EVENT_OBJECT_DESTROY,
            EVENT_OBJECT_UNCLOAKED,
            EVENT_OBJECT_HIDE,
        ]
        .into()
    }
}