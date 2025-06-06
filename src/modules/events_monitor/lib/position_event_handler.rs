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
use crate::win32::api::window::get_dpi_for_window;
use crate::win32::api::window::get_window_minmax_size;
use crate::win32::api::window::is_fullscreen;
use crate::win32::api::window::is_maximized;
use crate::win32::api::window::is_user_manageable_window;
use crate::win32::callbacks::win_event_hook::WindowsEvent;
use crate::win32::win_events_manager::WinEventHandler;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;
use crossbeam_channel::Sender;
use std::collections::HashMap;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_CONTROL;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_MENU;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_SHIFT;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_MOVESIZEEND;
use windows::Win32::UI::WindowsAndMessaging::EVENT_SYSTEM_MOVESIZESTART;

#[derive(Debug)]
struct WindowState {
    area: Area,
    is_maximized: bool,
    dpi: u32,
}

impl TryFrom<WindowRef> for WindowState {
    type Error = ();
    fn try_from(win: WindowRef) -> Result<WindowState, ()> {
        let is_maximized = is_maximized(win.hwnd);
        let is_fullscreen = if is_maximized { false } else { is_fullscreen(win.hwnd) };
        let area = win.get_area().ok_or(())?;
        let dpi = get_dpi_for_window(win.hwnd);
        Ok(WindowState {
            area,
            is_maximized: is_maximized || is_fullscreen,
            dpi,
        })
    }
}

pub struct PositionEventHandler {
    sender: Sender<MondrianMessage>,
    filter: WinMatcher,
    windows: HashMap<WindowRef, WindowState>,
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

        if let Ok(state) = winref.try_into() {
            let event = WindowEvent::StartMoveSize(winref);
            if !skip_window(&event, &self.filter) {
                self.sender.send(event.into()).expect("Failed to send event");
                self.windows.insert(winref, state);
            }
        }
    }

    fn end_movesize(&mut self, winref: WindowRef) {
        let (shift, alt, ctrl) = (
            get_key_state(VK_SHIFT.0).pressed,
            get_key_state(VK_MENU.0).pressed,
            get_key_state(VK_CONTROL.0).pressed,
        );

        let dest_point = match get_cursor_pos().inspect_err(|e| log::warn!("Failed to get cursor pos: {e:?}")) {
            Ok(point) => point,
            _ => return,
        };

        let prev_state = match self.windows.remove(&winref) {
            Some(s) => s,
            None => return,
        };

        let state: WindowState = match winref.try_into() {
            Ok(s) => s,
            Err(_) => return,
        };

        let (min_w, min_h) = get_window_minmax_size(winref.into()).0;
        let (pw, ph) = (prev_state.area.width as i32, prev_state.area.height as i32);
        let (w, h) = (state.area.width as i32, state.area.height as i32);

        let (def_insert, def_free_move) = (self.default_insert_in_monitor, self.default_free_move_in_monitor);
        let intramon_op = IntramonitorMoveOp::calc(alt, ctrl);
        let intermon_op = IntermonitorMoveOp::calc(def_insert, def_free_move, alt, shift, ctrl);

        if prev_state.is_maximized {
            let result = match state.is_maximized {
                true => MoveSizeResult::None,
                false => MoveSizeResult::Moved(dest_point, intramon_op, intermon_op),
            };
            return self.send_end(winref, result);
        }

        // INFO: if the window is now on a different monitor with different DPI
        if prev_state.dpi != state.dpi {
            return self.send_end(winref, MoveSizeResult::Moved(dest_point, intramon_op, intermon_op));
        }

        let is_shrinking = pw > w || ph > h;
        let is_resizing = (pw != w || ph != h) && w != min_w && h != min_h;
        if is_shrinking || is_resizing {
            return self.send_end(winref, MoveSizeResult::Resized(prev_state.area, state.area));
        }

        let (pcorners, corners) = (prev_state.area.get_all_corners(), state.area.get_all_corners());
        let corner_eqs = pcorners.iter().zip(corners.iter()).filter(|(p, c)| p.same(**c)).count();
        let result_event = match corner_eqs == 0 {
            true => MoveSizeResult::Moved(dest_point, intramon_op, intermon_op),
            false => MoveSizeResult::Resized(prev_state.area, state.area),
        };

        self.send_end(winref, result_event);
    }

    fn send_end(&self, winref: WindowRef, result: MoveSizeResult) {
        let _ = self
            .sender
            .send(WindowEvent::EndMoveSize(winref, result).into())
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
