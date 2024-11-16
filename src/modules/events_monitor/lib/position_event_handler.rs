use super::filter::skip_window;
use crate::app::config::win_matcher::WinMatcher;
use crate::app::mondrian_message::IntermonitorMoveOp;
use crate::app::mondrian_message::IntramonitorMoveOp;
use crate::app::mondrian_message::MondrianMessage;
use crate::app::mondrian_message::WindowEvent;
use crate::app::structs::area::Area;
use crate::win32::api::cursor::get_cursor_info;
use crate::win32::api::cursor::get_cursor_pos;
use crate::win32::api::key::get_key_state;
use crate::win32::api::window::is_user_managable_window;
use crate::win32::callbacks::win_event_hook::WindowsEvent;
use crate::win32::win_events_manager::WinEventHandler;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::mpsc::Sender;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_CONTROL;
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
    sender: Sender<MondrianMessage>,
    filter: WinMatcher,
    windows: HashMap<isize, (Area, bool)>,
    resize_cursors: HashSet<isize>,
    default_insert_in_monitor: bool,
    default_free_move_in_monitor: bool,
}

impl PositionEventHandler {
    pub fn new(
        sender: Sender<MondrianMessage>,
        filter: WinMatcher,
        default_insert_in_monitor: bool,
        default_free_move_in_monitor: bool,
    ) -> PositionEventHandler {
        PositionEventHandler {
            sender,
            filter,
            windows: HashMap::new(),
            resize_cursors: HashSet::new(),
            default_insert_in_monitor,
            default_free_move_in_monitor,
        }
    }

    fn start_movesize(&mut self, hwnd: HWND) {
        let area = WindowRef::new(hwnd).get_window_box();
        if let Some(area) = area {
            let is_resize = self.is_resize_handle();
            let event = WindowEvent::StartMoveSize(hwnd);
            if !skip_window(&event, &self.filter) {
                self.sender.send(event.into()).expect("Failed to send event");
                self.windows.insert(hwnd.0, (area, is_resize));
            }
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
        let (shift, alt, ctrl) = (
            get_key_state(VK_SHIFT.0).pressed,
            get_key_state(VK_MENU.0).pressed,
            get_key_state(VK_CONTROL.0).pressed,
        );

        let dest_point = get_cursor_pos();

        let (prev_area, is_resize) = match self.windows.remove(&hwnd.0) {
            Some(area) => area,
            None => return,
        };

        let curr_area = match WindowRef::new(hwnd).get_window_box() {
            Some(area) => area,
            None => return,
        };

        let intramonitor_op = self.get_intramonitor_op(alt, ctrl);
        let intermonitor_op = self.get_intermonitor_op(alt, shift, ctrl);
        let win_event = match is_resize {
            true => WindowEvent::Resized(hwnd, prev_area, curr_area),
            false => WindowEvent::Moved(hwnd, dest_point, intramonitor_op, intermonitor_op),
        };

        let _ = self
            .sender
            .send(win_event.into())
            .inspect_err(|err| log::error!("Failed to send event: {}", err));
    }

    fn get_intramonitor_op(&self, invert_mod: bool, free_mode_mod: bool) -> IntramonitorMoveOp {
        // NOTE: precedence: invert_mod > free_mode_mod
        if invert_mod {
            return IntramonitorMoveOp::Invert;
        }

        match free_mode_mod {
            true => IntramonitorMoveOp::InsertFreeMove,
            false => IntramonitorMoveOp::Swap,
        }
    }

    fn get_intermonitor_op(&self, invert_mod: bool, insert_mod: bool, free_mode_mod: bool) -> IntermonitorMoveOp {
        // NOTE: precedence: invert_mod > free_mode_mod > insert_mod
        if invert_mod {
            return IntermonitorMoveOp::Invert;
        }

        if self.default_insert_in_monitor {
            match (self.default_free_move_in_monitor, free_mode_mod, insert_mod) {
                (_, false, true) => IntermonitorMoveOp::Swap,
                (true, false, false) | (false, true, _) => IntermonitorMoveOp::InsertFreeMove,
                (false, false, false) | (true, true, _) => IntermonitorMoveOp::Insert,
            }
        } else {
            match (free_mode_mod, insert_mod) {
                (true, _) => IntermonitorMoveOp::InsertFreeMove,
                (false, true) => IntermonitorMoveOp::Insert,
                (false, false) => IntermonitorMoveOp::Swap,
            }
        }
    }
}

impl WinEventHandler for PositionEventHandler {
    fn init(&mut self) {
        // NOTE: loads all the resize cursors
        self.resize_cursors = [IDC_SIZEALL, IDC_SIZENS, IDC_SIZENWSE, IDC_SIZENESW, IDC_SIZEWE]
            .iter()
            .filter_map(|cursor| unsafe { LoadCursorW(None, *cursor) }.map(|c| c.0).ok())
            .collect();
    }

    fn handle(&mut self, event: &WindowsEvent) {
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
