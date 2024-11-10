use super::filter::skip_window;
use crate::app::config::win_matcher::WinMatcher;
use crate::app::mondrian_message::{MondrianMessage, WindowEvent};
use crate::win32::api::window::{enum_user_manageable_windows, is_user_managable_window, is_window_visible};
use crate::win32::callbacks::win_event_hook::WindowsEvent;
use crate::win32::win_events_manager::WinEventHandler;
use crate::win32::window::window_obj::WindowObjInfo;
use std::collections::HashSet;
use std::sync::mpsc::Sender;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    EVENT_OBJECT_CLOAKED, EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE, EVENT_OBJECT_SHOW, EVENT_OBJECT_UNCLOAKED,
    EVENT_SYSTEM_FOREGROUND,
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
        let is_managed = is_user_managable_window(hwnd, true, true, true);

        if is_managed && self.windows.insert(hwnd.0) {
            let win_event = WindowEvent::Opened(hwnd);
            if skip_window(&win_event, &self.filter) {
                return;
            }
            self.sender.send(win_event.into()).expect("Failed to send event open");
        }
    }
}

impl WinEventHandler for OpenCloseEventHandler {
    fn init(&mut self) {
        // INFO: bigger windows first
        let mut wins: Vec<(u32, HWND)> = enum_user_manageable_windows()
            .into_iter()
            .map(|w| (w.get_window_box().unwrap_or_default().get_area(), w.hwnd))
            .collect();
        wins.sort_by(|a, b| a.0.cmp(&b.0).reverse());
        wins.into_iter().for_each(|w| self.send_open_event(w.1));
    }

    fn handle(&mut self, event: &WindowsEvent) {
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
                .send(WindowEvent::Closed(event.hwnd).into())
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
