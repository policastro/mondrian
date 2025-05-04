use super::filter::skip_window;
use crate::app::mondrian_message::{MondrianMessage, SystemEvent, WindowEvent};
use crate::app::structs::win_matcher::WinMatcher;
use crate::win32::api::cursor::get_cursor_pos;
use crate::win32::api::window::{get_desktop_window, is_program_manager_window, is_user_manageable_window};
use crate::win32::callbacks::win_event_hook::WindowsEvent;
use crate::win32::win_events_manager::WinEventHandler;
use crate::win32::window::window_ref::WindowRef;
use std::sync::mpsc::Sender;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_FOREGROUND;

pub struct FocusEventHandler {
    sender: Sender<MondrianMessage>,
    filter: WinMatcher,
    program_manager_hwnd: Option<WindowRef>,
}

impl FocusEventHandler {
    pub fn new(sender: Sender<MondrianMessage>, filter: WinMatcher) -> FocusEventHandler {
        FocusEventHandler {
            sender,
            filter,
            program_manager_hwnd: None,
        }
    }
}

impl WinEventHandler for FocusEventHandler {
    fn init(&mut self) {
        self.program_manager_hwnd = get_desktop_window().map(WindowRef::from);
    }

    fn handle(&mut self, event: &WindowsEvent) {
        let is_managed = is_user_manageable_window(event.hwnd, true, true, true);
        if is_managed {
            let app_event = WindowEvent::Focused(event.hwnd.into());
            if skip_window(&app_event, &self.filter) {
                return;
            }
            self.sender.send(app_event.into()).expect("Failed to send focus event");
        }

        if is_program_manager_window(event.hwnd) {
            let app_event = SystemEvent::DesktopFocused {
                at: get_cursor_pos().expect("Failed to get cursor position"),
            };
            self.sender.send(app_event.into()).expect("Failed to send focus event");
        }
    }

    fn get_managed_events(&self) -> Option<Vec<u32>> {
        vec![EVENT_SYSTEM_FOREGROUND].into()
    }
}
