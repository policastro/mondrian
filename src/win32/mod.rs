pub mod api {
    pub mod accessibility;
    pub mod cursor;
    pub mod gdiplus;
    pub mod key;
    pub mod misc;
    pub mod monitor;
    pub mod session;
    pub mod window;
}

pub mod callbacks {
    pub mod enum_monitors;
    pub mod enum_windows;
    pub mod win_event_hook;
}
pub mod window {
    pub mod window_obj;
    pub mod window_ref;
    pub mod window_snapshot;
}
pub mod win_event_loop;
pub mod win_events_manager;
