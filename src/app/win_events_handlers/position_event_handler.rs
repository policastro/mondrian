use std::{collections::HashMap, sync::mpsc::Sender};

use windows::Win32::{
    Foundation::HWND,
    UI::{
        Input::KeyboardAndMouse::{VK_MENU, VK_SHIFT},
        WindowsAndMessaging::{EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MOVESIZESTART},
    },
};

use crate::win32::api::key::get_key_state;
use crate::win32::api::window::is_user_managable_window;
use crate::win32::win_events_manager::WinEventHandler;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;
use crate::win32::window::window_snapshot::WindowSnapshot;
use crate::win32::{api::cursor::get_cursor_pos, callbacks::win_event_hook::WinEvent};
use crate::{app::tiles_manager::tm_command::TMCommand, win32::api::window::get_window_minmax_size};

pub struct PositionEventHandler {
    sender: Sender<TMCommand>,
    windows: HashMap<isize, WindowSnapshot>,
}

impl PositionEventHandler {
    pub fn new(sender: Sender<TMCommand>) -> PositionEventHandler {
        PositionEventHandler {
            sender,
            windows: HashMap::new(),
        }
    }

    fn start_movesize(&mut self, hwnd: HWND) {
        let info = WindowRef::new(hwnd).snapshot();
        if info.is_some() {
            let event = TMCommand::WindowStartMoveSize(hwnd);
            self.sender.send(event).expect("Failed to send event");
        }
        info.and_then(|info| self.windows.insert(hwnd.0, info));
    }

    fn end_movesize(&mut self, hwnd: HWND) {
        let (shift_key, alt_key) = (get_key_state(VK_SHIFT.0), get_key_state(VK_MENU.0));
        let (invert_op, switch_orientation) = (shift_key.pressed, alt_key.pressed);
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

        let event: TMCommand = match area_shift.2 != 0 || area_shift.3 != 0 {
            true => {
                let ((min_width, min_height), _) = get_window_minmax_size(hwnd);
                // BUG: used to enable window move when window has a minsize. However, it is not working correctly when is resized at the minsize (it should send a window resize event)
                let is_minsize = (curr_area.width as i32 == min_width) || (curr_area.height as i32 == min_height);
                if !curr_area.contains(dest_point) && is_minsize {
                    TMCommand::WindowMoved(hwnd, dest_point, invert_op, switch_orientation)
                } else {
                    TMCommand::WindowResized(hwnd, area, curr_area)
                }
            }
            false => TMCommand::WindowMoved(hwnd, dest_point, invert_op, switch_orientation),
        };

        if let Err(err) = self.sender.send(event) {
            log::error!("Failed to send event: {}", err);
        }
    }
}

impl WinEventHandler for PositionEventHandler {
    fn init(&mut self) {}

    fn handle(&mut self, event: &WinEvent) {
        if !is_user_managable_window(event.hwnd, true, true, true) {
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
