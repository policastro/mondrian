use std::collections::{HashMap, HashSet};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::{
    Accessibility::{UnhookWinEvent, HWINEVENTHOOK},
    WindowsAndMessaging::{EVENT_MAX, EVENT_MIN},
};

use super::api::accessibility::set_global_win_event_hook;
use super::callbacks::win_event_hook::WINEVENT_CHANNEL;

#[allow(dead_code)]
#[derive(Debug)]
pub struct WinEvent {
    pub h_win_event_hook: HWINEVENTHOOK,
    pub event: u32,
    pub hwnd: HWND,
    pub id_object: i32,
    pub id_child: i32,
    pub id_event_thread: u32,
    pub dwms_event_time: u32,
}

pub trait WinEventHandler {
    fn init(&mut self);
    fn handle(&mut self, event: &WinEvent);
    /// Returns the list of events managed by this handler. If None, all events are managed.
    fn get_managed_events(&self) -> Option<Vec<u32>>;
}

pub struct WinEventManager {
    pub hooks_ids: Vec<HWINEVENTHOOK>,
    pub handlers: Vec<Box<dyn WinEventHandler + Send>>,
    pub events_map: HashMap<u32, HashSet<usize>>,
}

impl WinEventManager {
    pub fn new(handlers: Vec<Box<dyn WinEventHandler + Send>>) -> WinEventManager {
        let events_map = WinEventManager::build_events_map(&handlers);

        let hooks_ids = match events_map.contains_key(&0) {
            false => events_map.keys().map(|e| set_global_win_event_hook(*e, *e)).collect(),
            true => vec![set_global_win_event_hook(EVENT_MIN, EVENT_MAX)],
        };

        let mut wem = WinEventManager {
            handlers,
            events_map,
            hooks_ids,
        };
        wem.init_handlers();
        wem
    }

    pub fn check_for_last_event(&mut self) -> bool {
        if let Ok(event) = WINEVENT_CHANNEL.with(|channel| channel.lock().unwrap().1.try_recv()) {
            self.execute_hooks(&event);
            return true;
        }
        false
    }

    pub fn check_for_events(&mut self) {
        while self.check_for_last_event() {}
    }

    fn execute_hooks(&mut self, event: &WinEvent) {
        if let Some(handlers) = self.events_map.get(&event.event) {
            handlers.iter().for_each(|i| {
                self.handlers[*i].as_mut().handle(event);
            })
        }

        if let Some(handlers) = self.events_map.get(&0) {
            handlers.iter().for_each(|i| {
                self.handlers[*i].as_mut().handle(event);
            })
        }
    }

    pub fn builder() -> WinEventManagerBuilder {
        WinEventManagerBuilder::new()
    }

    fn init_handlers(&mut self) {
        self.handlers.iter_mut().for_each(|h| h.as_mut().init());
    }

    fn build_events_map(handlers: &[Box<dyn WinEventHandler + Send>]) -> HashMap<u32, HashSet<usize>> {
        let mut events_map: HashMap<u32, HashSet<usize>> = HashMap::new();
        for (i, h) in handlers.iter().enumerate() {
            if let Some(events) = h.get_managed_events() {
                events.iter().for_each(|e| {
                    events_map.entry(*e).or_default().insert(i);
                })
            } else {
                events_map.entry(0).or_default().insert(i);
            }
        }

        events_map
    }

    pub fn disconnect(&mut self) {
        self.hooks_ids.iter().for_each(|h| unsafe {
            let _ = UnhookWinEvent(*h);
        });
    }
}

impl Drop for WinEventManager {
    fn drop(&mut self) {
        self.disconnect();
    }
}

pub struct WinEventManagerBuilder {
    handlers: Vec<Box<dyn WinEventHandler + Send>>,
}

impl WinEventManagerBuilder {
    pub fn new() -> WinEventManagerBuilder {
        WinEventManagerBuilder { handlers: vec![] }
    }

    pub fn handler(mut self, handler: impl WinEventHandler + Send + 'static) -> Self {
        self.handlers.push(Box::new(handler));
        self
    }

    pub fn build(self) -> WinEventManager {
        WinEventManager::new(self.handlers)
    }
}
