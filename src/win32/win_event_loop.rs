use std::mem::MaybeUninit;

use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{DispatchMessageA, GetMessageA, TranslateMessage, MSG},
};

pub fn start_win_event_loop() {
    start_mono_win_event_loop(HWND::default());
}

pub fn start_mono_win_event_loop(hwnd: HWND) {
    let mut msg: MaybeUninit<MSG> = MaybeUninit::uninit();
    unsafe {
        while GetMessageA(msg.as_mut_ptr(), hwnd, 0, 0).0 > 0 {
            let _ = TranslateMessage(msg.as_ptr());
            let _ = DispatchMessageA(msg.as_ptr());
        }
    }
}

pub fn next_win_event_loop_iteration(msg: Option<&mut MaybeUninit<MSG>>) -> bool {
    let mut msg = msg.map_or(MaybeUninit::uninit(), |m| *m);

    unsafe {
        if GetMessageA(msg.as_mut_ptr(), HWND::default(), 0, 0).0 > 0 {
            let _ = TranslateMessage(msg.as_ptr());
            let _ = DispatchMessageA(msg.as_ptr());
            true
        } else {
            false
        }
    }
}
