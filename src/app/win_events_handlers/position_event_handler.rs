use std::{collections::HashMap, sync::mpsc::Sender};

use notify_rust::Notification;
use windows::Win32::{
    Foundation::HWND,
    UI::{
        Input::KeyboardAndMouse::{VK_MENU, VK_SHIFT},
        WindowsAndMessaging::{EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MOVESIZESTART},
    },
};

use crate::app::structs::orientation::Orientation;
use crate::win32::api::cursor::get_cursor_pos;
use crate::win32::api::key::get_key_state;
use crate::win32::api::window::is_user_managable_window;
use crate::win32::win_events_manager::{WinEvent, WinEventHandler};
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;
use crate::win32::window::window_snapshot::WindowSnapshot;
use crate::{app::win32_event::Win32Event, win32::api::window::get_window_minmax_size};

pub struct PositionEventHandler {
    sender: Sender<Win32Event>,
    windows: HashMap<isize, WindowSnapshot>,
}

impl PositionEventHandler {
    pub fn new(sender: Sender<Win32Event>) -> PositionEventHandler {
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

        let area = match self
            .windows
            .remove(&hwnd.0)
            .map(|i| i.viewarea.expect("Could not get area"))
        {
            Some(area) => area,
            None => return,
        };

        let curr_area = match WindowRef::new(hwnd).get_window_box() {
            Some(area) => area,
            None => return,
        };

        let area_shift = area.get_shift(&curr_area);

        let event: Win32Event = match area_shift.2 != 0 || area_shift.3 != 0 {
            true => {
                let ((min_width, min_height), _) = get_window_minmax_size(hwnd);
                // TODO Used to enable window move when window has a minsize. However, it is not working correctly when is resized at the minsize (it should send a window resize event)
                let is_minsize = (curr_area.width as i32 == min_width) || (curr_area.height as i32 == min_height);
                if !curr_area.contains(dest_point) && is_minsize {
                    Win32Event::WindowMoved(hwnd, dest_point, orientation)
                } else {
                    Win32Event::WindowResized(hwnd)
                }
            }
            false => Win32Event::WindowMoved(hwnd, dest_point, orientation),
        };

        if let Err(err) = self.sender.send(event) {
            log::error!("Failed to send event: {}", err);
        }
    }
}

impl WinEventHandler for PositionEventHandler {
    fn init(&mut self) {}
    
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

    fn get_managed_events(&self) -> Option<Vec<u32>> {
        vec![EVENT_SYSTEM_MOVESIZESTART, EVENT_SYSTEM_MOVESIZEEND].into()
    }
}