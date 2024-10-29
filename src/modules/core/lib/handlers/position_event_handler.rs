use crate::app::structs::area::Area;
use crate::modules::core::lib::tm_command::TMCommand;
use crate::win32::api::cursor::get_cursor_info;
use crate::win32::api::cursor::get_cursor_pos;
use crate::win32::api::key::get_key_state;
use crate::win32::api::window::is_user_managable_window;
use crate::win32::callbacks::win_event_hook::WinEvent;
use crate::win32::win_events_manager::WinEventHandler;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::mpsc::Sender;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_MENU;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_SHIFT;
use windows::Win32::UI::WindowsAndMessaging::LoadCursorW;
use windows::Win32::UI::WindowsAndMessaging::CURSORINFO;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_MOVESIZEEND;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_MOVESIZESTART;
use windows::Win32::UI::WindowsAndMessaging::IDC_SIZEALL;
use windows::Win32::UI::WindowsAndMessaging::IDC_SIZENESW;
use windows::Win32::UI::WindowsAndMessaging::IDC_SIZENS;
use windows::Win32::UI::WindowsAndMessaging::IDC_SIZENWSE;
use windows::Win32::UI::WindowsAndMessaging::IDC_SIZEWE;

pub struct PositionEventHandler {
    sender: Sender<TMCommand>,
    windows: HashMap<isize, (Area, bool)>,
    resize_cursors: HashSet<isize>,
}

impl PositionEventHandler {
    pub fn new(sender: Sender<TMCommand>) -> PositionEventHandler {
        PositionEventHandler {
            sender,
            windows: HashMap::new(),
            resize_cursors: HashSet::new(),
        }
    }

    fn start_movesize(&mut self, hwnd: HWND) {
        let area = WindowRef::new(hwnd).get_window_box();
        if let Some(area) = area {
            let is_resize = self.is_resize_handle();
            let event = TMCommand::WindowStartMoveSize(hwnd);
            self.sender.send(event).expect("Failed to send event");
            self.windows.insert(hwnd.0, (area, is_resize));
        }
    }

    fn is_resize_handle(&self) -> bool {
        let cursor_info: CURSORINFO = match get_cursor_info() {
            Ok(cursor_info) => cursor_info,
            Err(err) => {
                log::warn!("Failed to get cursor info: {}", err);
                return false;
            }
        };

        self.resize_cursors.contains(&cursor_info.hCursor.0)
    }

    fn end_movesize(&mut self, hwnd: HWND) {
        let (shift_key, alt_key) = (get_key_state(VK_SHIFT.0), get_key_state(VK_MENU.0));
        let (invert_op, switch_orientation) = (shift_key.pressed, alt_key.pressed);
        let dest_point = get_cursor_pos();

        let (prev_area, is_resize) = match self.windows.remove(&hwnd.0) {
            Some(area) => area,
            None => return,
        };

        let curr_area = match WindowRef::new(hwnd).get_window_box() {
            Some(area) => area,
            None => return,
        };

        let event = match is_resize {
            true => TMCommand::WindowResized(hwnd, prev_area, curr_area),
            false => TMCommand::WindowMoved(hwnd, dest_point, invert_op, switch_orientation),
        };

        if let Err(err) = self.sender.send(event) {
            log::error!("Failed to send event: {}", err);
        }
    }
}

impl WinEventHandler for PositionEventHandler {
    fn init(&mut self) {
        // NOTE: loads all the resize cursors
        [IDC_SIZEALL, IDC_SIZENS, IDC_SIZENWSE, IDC_SIZENESW, IDC_SIZEWE]
            .iter()
            .filter_map(|cursor| unsafe { LoadCursorW(None, *cursor) }.ok())
            .for_each(|cursor| {
                self.resize_cursors.insert(cursor.0);
            });
    }

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
