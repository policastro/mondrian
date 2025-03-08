use super::color::Color;
use super::utils;
use super::utils::overlay::OverlayBase;
use crate::win32::api::misc::post_empyt_message;
use crate::win32::api::misc::post_message;
use crate::win32::api::window::destroy_window;
use crate::win32::api::window::show_window;
use crate::win32::win_event_loop::start_win_event_loop;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::thread;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::SetWindowPos;
use windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE;
use windows::Win32::UI::WindowsAndMessaging::SWP_NOSENDCHANGING;
use windows::Win32::UI::WindowsAndMessaging::SWP_SHOWWINDOW;
use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;
use windows::Win32::UI::WindowsAndMessaging::WM_QUIT;

pub struct Overlay<P: OverlayBase + Clone + PartialEq + Copy + Send + 'static> {
    target: Option<HWND>,
    main_thread: Option<thread::JoinHandle<()>>,
    msg_tx: Option<Sender<OverlayMessage<P>>>,
    class_name: String,
}

enum OverlayMessage<P: OverlayBase + Clone + PartialEq + Copy> {
    Configure(P),
    Reposition(Option<P>),
    Quit,
    Hide,
}

impl<P: OverlayBase + Clone + PartialEq + Send + Copy> Overlay<P> {
    pub fn new(target: Option<HWND>, class_name: &str) -> Overlay<P> {
        Overlay::<P> {
            target,
            main_thread: None,
            msg_tx: None,
            class_name: class_name.to_string(),
        }
    }

    pub fn create(&mut self, params: P) {
        if self.exists() {
            return;
        }

        let target = self.target;
        let (tx, rx) = channel();
        self.msg_tx = Some(tx);
        let class_name = self.class_name.clone();
        let main_thread = thread::spawn(move || {
            let hwnd = utils::overlay::create(params, target, class_name.as_str());

            let t = thread::spawn(move || {
                let mut params = params;
                loop {
                    match rx.recv() {
                        Ok(OverlayMessage::Configure(p)) => {
                            if p != params {
                                post_message(hwnd, utils::overlay::WM_USER_CONFIGURE, Some(p));
                                params = p;
                                if let Some(target) = target {
                                    Self::move_overlay_to_target(hwnd, target, &params);
                                }
                            }
                        }
                        Ok(OverlayMessage::Reposition(p)) => {
                            if let Some(p) = p.filter(|p| *p != params) {
                                post_message(hwnd, utils::overlay::WM_USER_CONFIGURE, Some(p));
                                params = p;
                            }
                            if let Some(target) = target {
                                Self::move_overlay_to_target(hwnd, target, &params);
                            }
                        }
                        Ok(OverlayMessage::Hide) => {
                            show_window(hwnd, SW_HIDE);
                        }
                        _ => {
                            post_empyt_message(hwnd, WM_QUIT);
                            break;
                        }
                    }
                }
            });

            start_win_event_loop();
            destroy_window(hwnd);
            t.join().unwrap();
        });

        self.main_thread = Some(main_thread);
    }

    pub fn exists(&self) -> bool {
        self.main_thread.as_ref().is_some_and(|t| !t.is_finished())
    }

    pub fn destroy(&mut self) {
        if let Some(th) = self.main_thread.take() {
            self.send_msg(OverlayMessage::Quit, false);
            th.join().unwrap();
        }

        self.msg_tx = None;
    }

    pub fn configure(&self, params: P) {
        self.send_msg(OverlayMessage::Configure(params), true);
    }

    pub fn reposition(&self, params: Option<P>) {
        self.send_msg(OverlayMessage::Reposition(params), true);
    }

    pub fn hide(&self) {
        self.send_msg(OverlayMessage::Hide, true);
    }

    fn send_msg(&self, msg: OverlayMessage<P>, check_exists: bool) {
        if check_exists && !self.exists() {
            return;
        }

        if let Some(tx) = &self.msg_tx {
            tx.send(msg).unwrap();
        }
    }

    fn move_overlay_to_target(overlay: HWND, target: HWND, params: &P) {
        let (x, y, cx, cy) =
            match utils::overlay::get_box_from_target(target, params.get_thickness(), params.get_padding()) {
                Some(b) => b.into(),
                None => return,
            };
        let flags = SWP_NOSENDCHANGING | SWP_SHOWWINDOW | SWP_NOACTIVATE;
        let _ = unsafe { SetWindowPos(overlay, target, x, y, cx, cy, flags) };
    }
}

impl<P: OverlayBase + Clone + PartialEq + Send + Copy> Drop for Overlay<P> {
    fn drop(&mut self) {
        self.destroy();
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverlayParams {
    pub enabled: bool,
    pub color: Color,
    pub thickness: u8,
    pub border_radius: u8,
    pub padding: u8,
}

impl Default for OverlayParams {
    fn default() -> Self {
        OverlayParams::empty()
    }
}

impl OverlayParams {
    pub fn new(enabled: bool, color: Color, thickness: u8, border_radius: u8, padding: u8) -> OverlayParams {
        OverlayParams {
            enabled,
            color,
            thickness,
            border_radius,
            padding,
        }
    }

    pub fn empty() -> OverlayParams {
        OverlayParams::new(false, Color::new(0, 0, 0), 0, 0, 0)
    }
}
