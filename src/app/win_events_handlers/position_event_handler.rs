use std::{collections::HashMap, sync::mpsc::Sender};

use notify_rust::Notification;
use windows::Win32::{
    Foundation::HWND,
    UI::{
        Input::KeyboardAndMouse::{VK_MENU, VK_SHIFT},
        WindowsAndMessaging::{EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MOVESIZESTART},
    },
};

use crate::app::app_event::AppEvent;
use crate::app::structs::orientation::Orientation;
use crate::win32utils::api::cursor::get_cursor_pos;
use crate::win32utils::api::key::get_key_state;
use crate::win32utils::api::window::is_user_managable_window;
use crate::win32utils::win_events_manager::{WinEvent, WinEventHandler};
use crate::win32utils::window::window_obj::WindowObj;
use crate::win32utils::window::window_ref::WindowRef;
use crate::win32utils::window::window_snapshot::WindowSnapshot;

pub struct PositionEventHandler {
    sender: Sender<AppEvent>,
    windows: HashMap<isize, WindowSnapshot>,
}

impl PositionEventHandler {
    pub fn new(sender: Sender<AppEvent>) -> PositionEventHandler {
        PositionEventHandler {
            sender,
            windows: HashMap::new(),
        }
    }

    fn start_movesize(&mut self, hwnd: HWND) {
        let info = WindowRef::new(hwnd).snapshot();
        info.and_then(|info| self.windows.insert(hwnd.0, info));
    }

    fn end_movesize(&mut self, hwnd: HWND) {
        let (alt_key, shift_key) = (get_key_state(VK_MENU.0), get_key_state(VK_SHIFT.0));
        let orientation = match (alt_key.pressed, shift_key.pressed) {
            (true, true) => {
                // TODO This should be removed in production
                log::info!(target: "app::inspector", "{:?}", WindowRef::new(hwnd).snapshot());
                let _ = Notification::new()
                    .summary("Mondrian: inspection done!")
                    .body("Added new window to the inspection file")
                    .icon("mondrian")
                    .show();
                None
            }
            (_, true) => Some(Orientation::Vertical),
            (true, _) => Some(Orientation::Horizontal),
            _ => None,
        };
        let dest_point = get_cursor_pos();

        let area = match self.windows.remove(&hwnd.0).map(|i| i.viewarea) {
            Some(area) => area,
            None => return,
        };

        let curr_area = match WindowRef::new(hwnd).get_window_box() {
            Some(area) => area,
            None => return,
        };

        let area_shift = area.get_shift(&curr_area);
        let event = match area_shift.2 != 0 || area_shift.3 != 0 {
            true => AppEvent::WindowResized(hwnd),
            false => AppEvent::WindowMoved(hwnd, dest_point, orientation),
        };

        if let Err(err) = self.sender.send(event) {
            log::error!("Failed to send event: {}", err);
        }
    }
}

impl WinEventHandler for PositionEventHandler {
    fn handle(&mut self, event: &WinEvent) {
        if !is_user_managable_window(event.hwnd, true, true) {
            return;
        }

        match event.event {
            EVENT_SYSTEM_MOVESIZESTART => self.start_movesize(event.hwnd),
            EVENT_SYSTEM_MOVESIZEEND => self.end_movesize(event.hwnd),
            _ => (),
        }
    }
}
