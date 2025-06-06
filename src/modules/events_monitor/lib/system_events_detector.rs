use crate::app::mondrian_message::MondrianMessage;
use crate::app::mondrian_message::SystemEvent;
use crate::win32::api::session::is_user_logged_in;
use crate::win32::api::window::create_window;
use crossbeam_channel::Sender;
use lazy_static::lazy_static;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::sync::Mutex;
use windows::core::PCWSTR;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::LRESULT;
use windows::Win32::Foundation::WPARAM;
use windows::Win32::System::LibraryLoader::GetModuleHandleExW;
use windows::Win32::System::RemoteDesktop::WTSRegisterSessionNotification;
use windows::Win32::System::RemoteDesktop::WTSUnRegisterSessionNotification;
use windows::Win32::System::RemoteDesktop::NOTIFY_FOR_THIS_SESSION;
use windows::Win32::UI::WindowsAndMessaging::DefWindowProcW;
use windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW;
use windows::Win32::UI::WindowsAndMessaging::PostQuitMessage;
use windows::Win32::UI::WindowsAndMessaging::RegisterClassExW;
use windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW;
use windows::Win32::UI::WindowsAndMessaging::CREATESTRUCTW;
use windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA;
use windows::Win32::UI::WindowsAndMessaging::PBT_APMRESUMEAUTOMATIC;
use windows::Win32::UI::WindowsAndMessaging::PBT_APMSUSPEND;
use windows::Win32::UI::WindowsAndMessaging::SPI_SETWORKAREA;
use windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE;
use windows::Win32::UI::WindowsAndMessaging::WM_CREATE;
use windows::Win32::UI::WindowsAndMessaging::WM_DESTROY;
use windows::Win32::UI::WindowsAndMessaging::WM_DISPLAYCHANGE;
use windows::Win32::UI::WindowsAndMessaging::WM_POWERBROADCAST;
use windows::Win32::UI::WindowsAndMessaging::WM_QUIT;
use windows::Win32::UI::WindowsAndMessaging::WM_SETTINGCHANGE;
use windows::Win32::UI::WindowsAndMessaging::WM_WTSSESSION_CHANGE;
use windows::Win32::UI::WindowsAndMessaging::WNDCLASSEXW;
use windows::Win32::UI::WindowsAndMessaging::WS_OVERLAPPEDWINDOW;
use windows::Win32::UI::WindowsAndMessaging::WTS_SESSION_LOCK;
use windows::Win32::UI::WindowsAndMessaging::WTS_SESSION_LOGOFF;
use windows::Win32::UI::WindowsAndMessaging::WTS_SESSION_LOGON;
use windows::Win32::UI::WindowsAndMessaging::WTS_SESSION_UNLOCK;

lazy_static! {
    static ref CLASS_REGISTER_MUTEX: Mutex<()> = Mutex::new(());
}

const SYSTEM_EVENTS_DETECTION_CLASS_NAME: &str = "mondrian:system_events_detection";

fn send_system_event(hwnd: HWND, event: SystemEvent) {
    unsafe {
        let tx = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Sender<MondrianMessage>;
        if !tx.is_null() {
            let _ = (*tx)
                .send(event.into())
                .inspect_err(|_| log::warn!("Failed to send event {event:?}"));
        }
    };
}

unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            let create_struct = &*(lparam.0 as *const CREATESTRUCTW);
            let custom_value = create_struct.lpCreateParams as *mut Sender<MondrianMessage>;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, custom_value as isize);
            let _ = WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION);
            LRESULT(0)
        }
        WM_DISPLAYCHANGE => {
            send_system_event(hwnd, SystemEvent::MonitorsLayoutChanged);
            LRESULT(0)
        }
        WM_SETTINGCHANGE => {
            if wparam.0 as u32 == SPI_SETWORKAREA.0 {
                send_system_event(hwnd, SystemEvent::WorkareaChanged);
            }
            LRESULT(0)
        }
        WM_POWERBROADCAST => {
            match wparam.0 as u32 {
                PBT_APMSUSPEND => send_system_event(hwnd, SystemEvent::Standby),
                PBT_APMRESUMEAUTOMATIC => send_system_event(
                    hwnd,
                    SystemEvent::Resume {
                        logged_in: is_user_logged_in(),
                    },
                ),
                _ => (),
            };

            LRESULT(1)
        }
        WM_WTSSESSION_CHANGE => {
            match wparam.0 as u32 {
                WTS_SESSION_LOGON => send_system_event(hwnd, SystemEvent::SessionLogon),
                WTS_SESSION_LOGOFF => send_system_event(hwnd, SystemEvent::SessionLogoff),
                WTS_SESSION_LOCK => send_system_event(hwnd, SystemEvent::SessionLocked),
                WTS_SESSION_UNLOCK => send_system_event(hwnd, SystemEvent::SessionUnlocked),
                _ => (),
            };

            LRESULT(0)
        }
        WM_DESTROY | WM_QUIT => {
            let _ = WTSUnRegisterSessionNotification(hwnd);
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

pub fn create(tx: Sender<MondrianMessage>) -> Option<HWND> {
    let _lock = CLASS_REGISTER_MUTEX.lock().unwrap();

    let mut hmod: HMODULE = unsafe { std::mem::zeroed() };
    unsafe { GetModuleHandleExW(0, None, &mut hmod).unwrap() };

    let cs_name = OsStr::new(SYSTEM_EVENTS_DETECTION_CLASS_NAME);
    let cs_w: Vec<u16> = cs_name.encode_wide().chain(Some(0)).collect();
    let cs_ptr = PCWSTR(cs_w.as_ptr());

    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        hInstance: hmod.into(),
        lpszClassName: cs_ptr,
        lpfnWndProc: Some(window_proc),
        ..Default::default()
    };

    unsafe { RegisterClassExW(&wc) };
    let style = WS_OVERLAPPEDWINDOW;
    let size = (0, 0, 0, 0);

    create_window(WINDOW_EX_STYLE::default(), cs_ptr, style, size, None, hmod, tx)
}
