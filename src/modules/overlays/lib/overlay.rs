use std::sync::mpsc::{channel, Sender};

use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::WM_QUIT};

use crate::win32::{
    api::{
        misc::{post_empyt_message, post_message},
        window::destroy_window,
    },
    win_event_loop::start_mono_win_event_loop,
};
use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, SWP_NOACTIVATE, SWP_NOSENDCHANGING, SWP_SHOWWINDOW};

use super::{color::Color, utils};
use std::thread;

pub struct Overlay {
    target: HWND,
    main_thread: Option<thread::JoinHandle<()>>,
    msg_tx: Option<Sender<OverlayMessage>>,
    active_params: OverlayParams,
    inactive_params: OverlayParams,
}
enum OverlayMessage {
    Quit,
    SetActive,
    SetInactive,
    Reposition(Option<bool>),
}

impl Overlay {
    pub fn new(target: HWND, active_params: OverlayParams, inactive_params: OverlayParams) -> Overlay {
        Overlay {
            target,
            main_thread: None,
            active_params,
            inactive_params,
            msg_tx: None,
        }
    }

    pub fn create(&mut self, activated: bool) {
        if self.exists() {
            return;
        }

        let (active, inactive) = (self.active_params, self.inactive_params);
        let target = self.target;
        let (tx, rx) = channel();
        self.msg_tx = Some(tx);
        let main_thread = thread::spawn(move || {
            let params = if activated { active } else { inactive };
            let hwnd = utils::overlay::create(params, Some(target));

            let mut curr_activated = activated;
            let t = thread::spawn(move || loop {
                match rx.recv() {
                    Ok(OverlayMessage::SetActive) => {
                        post_message(hwnd, utils::overlay::WM_CHANGE_BORDER, Some(active));
                        curr_activated = true;
                    }
                    Ok(OverlayMessage::SetInactive) => {
                        post_message(hwnd, utils::overlay::WM_CHANGE_BORDER, Some(inactive));
                        curr_activated = false;
                    }
                    Ok(OverlayMessage::Reposition(activated)) => {
                        let activated = activated.unwrap_or(curr_activated);
                        let (tickness, padding) = match activated {
                            true => (active.thickness, active.padding),
                            false => (inactive.thickness, inactive.padding),
                        };
                        if curr_activated != activated {
                            curr_activated = activated;
                            post_message(hwnd, utils::overlay::WM_CHANGE_BORDER, Some(params));
                        }
                        Self::move_overlay_to_target(hwnd, target, tickness, padding);
                    }
                    _ => {
                        post_empyt_message(hwnd, WM_QUIT);
                        break;
                    }
                }
            });

            start_mono_win_event_loop(hwnd);
            destroy_window(hwnd);
            t.join().unwrap();
        });

        self.main_thread = Some(main_thread);
    }

    pub fn exists(&self) -> bool {
        self.main_thread.as_ref().is_some_and(|t| !t.is_finished())
    }

    pub fn destroy(&mut self) {
        if !self.exists() {
            return;
        }

        if let Some(th) = self.main_thread.take() {
            self.send_msg(OverlayMessage::Quit);
            th.join().unwrap();
        }

        self.msg_tx = None;
    }

    pub fn activate(&mut self, active: bool) {
        match active {
            true => self.send_msg(OverlayMessage::SetActive),
            false => self.send_msg(OverlayMessage::SetInactive),
        };
    }

    pub fn reposition(&mut self, activated: Option<bool>) {
        if !self.exists() {
            self.create(activated.unwrap_or(false));
            return;
        }
        self.send_msg(OverlayMessage::Reposition(activated));
    }

    fn send_msg(&self, msg: OverlayMessage) {
        if let Some(tx) = &self.msg_tx {
            tx.send(msg).unwrap();
        }
    }

    fn move_overlay_to_target(overlay: HWND, target: HWND, thickness: u8, padding: u8) {
        let (x, y, cx, cy) = match utils::overlay::get_box_from_target(target, thickness, padding) {
            Some(b) => b,
            None => return,
        };
        let flags = SWP_NOSENDCHANGING | SWP_SHOWWINDOW | SWP_NOACTIVATE;
        let _ = unsafe { SetWindowPos(overlay, target, x, y, cx, cy, flags) };
    }
}

impl Drop for Overlay {
    fn drop(&mut self) {
        self.destroy();
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OverlayParams {
    pub color: Color,
    pub thickness: u8,
    pub padding: u8,
}

impl Default for OverlayParams {
    fn default() -> Self {
        OverlayParams {
            color: Color::new(0, 0, 0),
            thickness: 0,
            padding: 0,
        }
    }
}

impl OverlayParams {
    pub fn new(color: Color, thickness: u8, padding: u8) -> OverlayParams {
        OverlayParams {
            color,
            thickness,
            padding,
        }
    }

    pub fn empty() -> OverlayParams {
        OverlayParams::new(Color::new(0, 0, 0), 0, 0)
    }
}
