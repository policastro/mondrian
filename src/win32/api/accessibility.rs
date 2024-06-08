use windows::Win32::UI::{
    Accessibility::{SetWinEventHook, HWINEVENTHOOK},
    WindowsAndMessaging::WINEVENT_OUTOFCONTEXT,
};

use crate::win32::callbacks::win_event_hook::win_event_hook;

pub fn set_global_win_event_hook(event_min: u32, event_max: u32) -> HWINEVENTHOOK {
    unsafe {
        SetWinEventHook(
            event_min,
            event_max,
            None,
            Some(win_event_hook),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        )
    }
}
