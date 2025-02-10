use crate::app::mondrian_message::MondrianMessage;
use crate::app::mondrian_message::SystemEvent;
use crate::win32::api::window::create_window;
use lazy_static::lazy_static;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use windows::core::PCWSTR;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::LRESULT;
use windows::Win32::Foundation::WPARAM;
use windows::Win32::System::LibraryLoader::GetModuleHandleExW;
use windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW;
use windows::Win32::UI::WindowsAndMessaging::PostQuitMessage;
use windows::Win32::UI::WindowsAndMessaging::RegisterClassExW;
use windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW;
use windows::Win32::UI::WindowsAndMessaging::CREATESTRUCTW;
use windows::Win32::UI::WindowsAndMessaging::CW_USEDEFAULT;
use windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA;
use windows::Win32::UI::WindowsAndMessaging::HTCAPTION;
use windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE;
use windows::Win32::UI::WindowsAndMessaging::WM_CREATE;
use windows::Win32::UI::WindowsAndMessaging::WM_DESTROY;
use windows::Win32::UI::WindowsAndMessaging::WM_DISPLAYCHANGE;
use windows::Win32::UI::WindowsAndMessaging::WM_QUIT;
use windows::Win32::UI::WindowsAndMessaging::WNDCLASSEXW;
use windows::Win32::UI::WindowsAndMessaging::WS_OVERLAPPEDWINDOW;

lazy_static! {
    static ref CLASS_REGISTER_MUTEX: Mutex<()> = Mutex::new(());
}

const MONITOR_EVENTS_DETECTION_CLASS_NAME: &str = "mondrian:monitor_events_detection";

unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, _wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            let create_struct = &*(lparam.0 as *const CREATESTRUCTW);
            let custom_value = create_struct.lpCreateParams as *mut Sender<MondrianMessage>;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, custom_value as isize);
            LRESULT(0)
        }
        WM_DISPLAYCHANGE => {
            let tx = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Sender<MondrianMessage>;
            if !tx.is_null() {
                let res = (*tx).send(SystemEvent::MonitorsLayoutChanged.into());
                res.expect("Failed to send message");
            }
            LRESULT(0)
        }
        WM_DESTROY | WM_QUIT => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => LRESULT(HTCAPTION as isize),
    }
}

pub fn create(tx: Sender<MondrianMessage>) -> Option<HWND> {
    let _lock = CLASS_REGISTER_MUTEX.lock().unwrap();

    let mut hmod: HMODULE = unsafe { std::mem::zeroed() };
    unsafe { GetModuleHandleExW(0, None, &mut hmod).unwrap() };

    let cs_name = OsStr::new(MONITOR_EVENTS_DETECTION_CLASS_NAME);
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
    let size = (CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT);

    create_window(WINDOW_EX_STYLE::default(), cs_ptr, style, size, None, hmod, tx)
}
