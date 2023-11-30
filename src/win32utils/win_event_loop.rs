use std::mem::MaybeUninit;

use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{DispatchMessageA, GetMessageA, TranslateMessage},
};

pub fn run_win_event_loop() {
    let mut msg = MaybeUninit::uninit();
    loop {
        unsafe {
            if GetMessageA(msg.as_mut_ptr(), HWND(0), 0, 0).0 > 0 {
                let _ = TranslateMessage(msg.as_ptr());
                let _ = DispatchMessageA(msg.as_ptr());
            } else {
                break;
            }
        }

        /*
        if let Ok(event) = TrayIconEvent::receiver().try_recv() {
            //println!("tray event: {:?}", event);
        }

        if let Ok(event) = MenuEvent::receiver().try_recv() {
            println!("menu event: {:?}", event);
        }
        */
    }
}
