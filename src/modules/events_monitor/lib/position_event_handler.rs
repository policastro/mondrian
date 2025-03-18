use super::filter::skip_window;
use crate::app::mondrian_message::IntermonitorMoveOp;
use crate::app::mondrian_message::IntramonitorMoveOp;
use crate::app::mondrian_message::MondrianMessage;
use crate::app::mondrian_message::MoveSizeResult;
use crate::app::mondrian_message::WindowEvent;
use crate::app::structs::area::Area;
use crate::app::structs::point::Point;
use crate::app::structs::win_matcher::WinMatcher;
use crate::win32::api::cursor::get_cursor_pos;
use crate::win32::api::key::get_key_state;
use crate::win32::api::window::get_window_minmax_size;
use crate::win32::api::window::is_fullscreen;
use crate::win32::api::window::is_maximized;
use crate::win32::api::window::is_user_manageable_window;
use crate::win32::callbacks::win_event_hook::WindowsEvent;
use crate::win32::win_events_manager::WinEventHandler;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_CONTROL;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_MENU;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_SHIFT;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_MOVESIZEEND;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_MOVESIZESTART;

pub struct PositionEventHandler {
    sender: Sender<MondrianMessage>,
    filter: WinMatcher,
    windows: HashMap<WindowRef, (Area, bool)>,
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
            default_insert_in_monitor,
            default_free_move_in_monitor,
        }
    }

    fn start_movesize(&mut self, winref: WindowRef) {
        if self.windows.contains_key(&winref) {
            return; // NOTE: already in movesize
        }

        // NOTE: check is_maximized as first operation
        let is_maximized = self.is_maximized(winref.into());
        let area = winref.get_area();
        if let Some(area) = area {
            let event = WindowEvent::StartMoveSize(winref);
            if !skip_window(&event, &self.filter) {
                self.sender.send(event.into()).expect("Failed to send event");
                self.windows.insert(winref, (area, is_maximized));
            }
        }
    }

    fn end_movesize(&mut self, winref: WindowRef) {
        let (shift, alt, ctrl) = (
            get_key_state(VK_SHIFT.0).pressed,
            get_key_state(VK_MENU.0).pressed,
            get_key_state(VK_CONTROL.0).pressed,
        );

        let dest_point = match get_cursor_pos().inspect(|_| log::warn!("Failed to get cursor pos")) {
            Ok(point) => point,
            _ => return,
        };

        let (prev_area, was_maximized) = match self.windows.remove(&winref) {
            Some(prev_area) => prev_area,
            None => return,
        };

        let curr_area = match winref.get_area() {
            Some(area) => area,
            None => return,
        };

        let (min_w, min_h) = get_window_minmax_size(winref.into()).0;
        let (pw, ph) = (prev_area.width as i32, prev_area.height as i32);
        let (w, h) = (curr_area.width as i32, curr_area.height as i32);

        let (def_insert, def_free_move) = (self.default_insert_in_monitor, self.default_free_move_in_monitor);
        let intramon_op = IntramonitorMoveOp::calc(alt, ctrl);
        let intermon_op = IntermonitorMoveOp::calc(def_insert, def_free_move, alt, shift, ctrl);

        if was_maximized {
            let result = match self.is_maximized(winref.into()) {
                true => MoveSizeResult::None,
                false => MoveSizeResult::Moved(dest_point, intramon_op, intermon_op),
            };
            self.send(WindowEvent::EndMoveSize(winref, result));
            return;
        }

        let is_shrinking = pw > w || ph > h;
        let is_resizing = (pw != w || ph != h) && w != min_w && h != min_h;
        if is_shrinking || is_resizing {
            let win_event = WindowEvent::EndMoveSize(winref, MoveSizeResult::Resized(prev_area, curr_area));
            self.send(win_event);
            return;
        }

        let (pcorners, corners) = (prev_area.get_all_corners(), curr_area.get_all_corners());
        let corner_eqs = pcorners.iter().zip(corners.iter()).filter(|(p, c)| p.same(**c)).count();
        let result_event = match corner_eqs == 0 {
            true => MoveSizeResult::Moved(dest_point, intramon_op, intermon_op),
            false => MoveSizeResult::Resized(prev_area, curr_area),
        };

        self.send(WindowEvent::EndMoveSize(winref, result_event));
    }

    fn is_maximized(&self, hwnd: HWND) -> bool {
        let is_maximized = is_maximized(hwnd);
        let is_fullscreen = if is_maximized { false } else { is_fullscreen(hwnd) };
        is_maximized || is_fullscreen
    }

    fn send(&self, event: WindowEvent) {
        let _ = self
            .sender
            .send(event.into())
            .inspect_err(|err| log::error!("Failed to send event: {}", err));
    }
}

impl WinEventHandler for PositionEventHandler {
    fn init(&mut self) {}

    fn handle(&mut self, event: &WindowsEvent) {
        if !is_user_manageable_window(event.hwnd, true, true, true) {
            return;
        }

        let winref = event.hwnd.into();
        match event.event {
            EVENT_SYSTEM_MOVESIZESTART => self.start_movesize(winref),
            EVENT_SYSTEM_MOVESIZEEND => self.end_movesize(winref),
            _ => (),
        }
    }

    fn get_managed_events(&self) -> Option<Vec<u32>> {
        vec![EVENT_SYSTEM_MOVESIZESTART, EVENT_SYSTEM_MOVESIZEEND].into()
    }
}
