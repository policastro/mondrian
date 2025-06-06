use super::filter::skip_window;
use crate::app::mondrian_message::{MondrianMessage, WindowEvent};
use crate::app::structs::win_matcher::WinMatcher;
use crate::win32::api::window::{enum_user_manageable_windows, is_user_manageable_window, is_window_visible};
use crate::win32::callbacks::win_event_hook::WindowsEvent;
use crate::win32::win_events_manager::WinEventHandler;
use crate::win32::window::window_ref::WindowRef;
use crossbeam_channel::Sender;
use std::collections::HashSet;
use std::thread;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    EVENT_OBJECT_CLOAKED, EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE, EVENT_OBJECT_SHOW, EVENT_OBJECT_UNCLOAKED,
    EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MINIMIZEEND,
};

enum QueueMessage {
    Event(WindowEvent, u32),
    Quit,
}

pub struct OpenCloseEventHandler {
    sender: Sender<MondrianMessage>,
    filter: WinMatcher,
    windows: HashSet<WindowRef>,
    delayed_filter: Vec<(WinMatcher, u32)>,
    message_queue_tx: Option<Sender<QueueMessage>>,
    message_queue_thread: Option<thread::JoinHandle<()>>,
}

impl OpenCloseEventHandler {
    pub fn new(
        sender: Sender<MondrianMessage>,
        filter: WinMatcher,
        delayed_filter: Vec<(WinMatcher, u32)>,
    ) -> OpenCloseEventHandler {
        OpenCloseEventHandler {
            sender,
            filter,
            windows: HashSet::new(),
            message_queue_tx: None,
            message_queue_thread: None,
            delayed_filter,
        }
    }
}

impl OpenCloseEventHandler {
    fn send_open_event(&mut self, hwnd: HWND) {
        let is_managed = is_user_manageable_window(hwnd, true, true, true);
        let winref = WindowRef::new(hwnd);

        if is_managed && self.windows.insert(winref) {
            let win_event = WindowEvent::Opened(winref);
            if skip_window(&win_event, &self.filter) {
                return;
            }

            self.sender
                .send(win_event.into())
                .inspect_err(|_| log::warn!("Failed to send open event for window {:?}", hwnd))
                .ok();

            // INFO: sends a reposition event after a delay for the matched windows
            if let Some((_, delay)) = self.delayed_filter.iter().find(|(f, _)| f.matches(winref)) {
                self.message_queue_tx
                    .as_ref()
                    .unwrap()
                    .send(QueueMessage::Event(WindowEvent::Repositioned(winref), *delay))
                    .ok();
            }
        }
    }
}

impl WinEventHandler for OpenCloseEventHandler {
    fn init(&mut self) {
        self.windows = enum_user_manageable_windows()
            .into_iter()
            .filter(|w| is_user_manageable_window(w.hwnd, true, true, true))
            .map(|w| w.hwnd.into())
            .collect();

        let (tx, rx) = crossbeam_channel::unbounded();
        self.message_queue_tx = Some(tx);
        let sender = self.sender.clone();
        self.message_queue_thread = Some(thread::spawn(move || loop {
            match rx.recv() {
                Ok(QueueMessage::Event(event, delay)) => {
                    thread::sleep(std::time::Duration::from_millis(delay.into()));
                    sender.send(event.into()).ok();
                }
                Ok(QueueMessage::Quit) => break,
                Err(_) => break,
            }
        }));
    }

    fn handle(&mut self, event: &WindowsEvent) {
        if event.hwnd.is_invalid() {
            return;
        }

        if event.event == EVENT_SYSTEM_FOREGROUND
            || event.event == EVENT_OBJECT_UNCLOAKED
            || event.event == EVENT_OBJECT_SHOW
        {
            self.send_open_event(event.hwnd);
            return;
        }

        if event.event == EVENT_SYSTEM_MINIMIZEEND {
            let is_managed = is_user_manageable_window(event.hwnd, true, true, true);
            if is_managed && !skip_window(&WindowEvent::Minimized(event.hwnd.into()), &self.filter) {
                self.windows.insert(event.hwnd.into());
            }
            return;
        }

        if !self.windows.contains(&event.hwnd.into()) {
            return;
        }

        let (is_cloaked, is_destroyed, is_hidden) = (
            event.event == EVENT_OBJECT_CLOAKED,
            event.event == EVENT_OBJECT_DESTROY,
            event.event == EVENT_OBJECT_HIDE,
        );

        if is_cloaked || ((is_destroyed || is_hidden) && !is_window_visible(event.hwnd)) {
            self.windows.remove(&event.hwnd.into());
            self.sender
                .send(WindowEvent::Closed(event.hwnd.into()).into())
                .inspect_err(|e| log::warn!("Failed to send close event: {:?}", e))
                .ok();
        }
    }

    fn get_managed_events(&self) -> Option<Vec<u32>> {
        vec![
            EVENT_SYSTEM_FOREGROUND,
            EVENT_OBJECT_CLOAKED,
            EVENT_OBJECT_SHOW,
            EVENT_SYSTEM_MINIMIZEEND,
            EVENT_OBJECT_DESTROY,
            EVENT_OBJECT_UNCLOAKED,
            EVENT_OBJECT_HIDE,
        ]
        .into()
    }
}

impl Drop for OpenCloseEventHandler {
    fn drop(&mut self) {
        if let Some(tx) = self.message_queue_tx.take() {
            tx.send(QueueMessage::Quit).ok();
            self.message_queue_thread.take().unwrap().join().ok();
        }
    }
}
