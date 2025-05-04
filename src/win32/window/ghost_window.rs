use crate::app::structs::area::Area;
use crate::win32::api::misc::get_current_thread_id;
use crate::win32::api::misc::post_empty_thread_message;
use crate::win32::api::window::create_window;
use crate::win32::api::window::destroy_window;
use crate::win32::win_event_loop::start_win_event_loop;
use crate::win32::window::window_obj::WindowObjHandler;
use crate::win32::window::window_ref::WindowRef;
use lazy_static::lazy_static;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::sync::atomic::AtomicIsize;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use windows::core::PCWSTR;
use windows::Win32::Foundation::HMODULE;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::LRESULT;
use windows::Win32::Foundation::WPARAM;
use windows::Win32::System::LibraryLoader::GetModuleHandleExW;
use windows::Win32::System::RemoteDesktop::WTSUnRegisterSessionNotification;
use windows::Win32::UI::WindowsAndMessaging::DefWindowProcW;
use windows::Win32::UI::WindowsAndMessaging::PostQuitMessage;
use windows::Win32::UI::WindowsAndMessaging::RegisterClassExW;
use windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE;
use windows::Win32::UI::WindowsAndMessaging::WM_DESTROY;
use windows::Win32::UI::WindowsAndMessaging::WM_QUIT;
use windows::Win32::UI::WindowsAndMessaging::WNDCLASSEXW;
use windows::Win32::UI::WindowsAndMessaging::WS_DISABLED;
use windows::Win32::UI::WindowsAndMessaging::WS_EX_NOACTIVATE;
use windows::Win32::UI::WindowsAndMessaging::WS_EX_TOOLWINDOW;
use windows::Win32::UI::WindowsAndMessaging::WS_OVERLAPPEDWINDOW;
use windows::Win32::UI::WindowsAndMessaging::WS_POPUP;

lazy_static! {
    static ref CLASS_REGISTER_MUTEX: Mutex<()> = Mutex::new(());
}
const GHOST_WINDOW_CLASS_NAME: &str = "mondrian:ghost_window";

pub struct GhostWindow {
    main_thread: Option<thread::JoinHandle<()>>,
    main_thread_id: Arc<AtomicU32>,
    hwnd: Arc<AtomicIsize>,
}

impl GhostWindow {
    pub fn create(initial_area: Area) -> GhostWindow {
        let mut placeholder = GhostWindow {
            main_thread: None,
            main_thread_id: Arc::new(AtomicU32::new(0)),
            hwnd: Arc::new(AtomicIsize::new(0)),
        };

        let thread_id = placeholder.main_thread_id.clone();
        let shared_hwnd = placeholder.hwnd.clone();
        let area = initial_area;
        placeholder.main_thread = Some(thread::spawn(move || {
            let hwnd = match create() {
                Some(hwnd) => hwnd,
                None => {
                    log::warn!("Failure while trying to create the monitor placeholder window");
                    return;
                }
            };

            shared_hwnd.store(hwnd.0 as isize, Ordering::Release);
            thread_id.store(get_current_thread_id(), Ordering::Release);

            let winref: WindowRef = hwnd.into();
            winref
                .resize_and_move(area.get_origin(), area.get_size(), false, SWP_NOACTIVATE)
                .ok();
            start_win_event_loop();
            shared_hwnd.store(0, Ordering::Release);
            destroy_window(hwnd);
        }));

        placeholder
    }
}

impl TryFrom<&GhostWindow> for WindowRef {
    type Error = ();
    fn try_from(placeholder: &GhostWindow) -> Result<WindowRef, ()> {
        match placeholder.hwnd.load(Ordering::Acquire) {
            0 => Err(()),
            hwnd => Ok(hwnd.into()),
        }
    }
}

impl Drop for GhostWindow {
    fn drop(&mut self) {
        if let Some(thread) = self.main_thread.take() {
            post_empty_thread_message(self.main_thread_id.load(Ordering::Acquire), WM_QUIT);
            thread.join().unwrap();
        };
    }
}

unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY | WM_QUIT => {
            let _ = WTSUnRegisterSessionNotification(hwnd);
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn create() -> Option<HWND> {
    let _lock = CLASS_REGISTER_MUTEX.lock().unwrap();

    let mut hmod: HMODULE = unsafe { std::mem::zeroed() };
    unsafe { GetModuleHandleExW(0, None, &mut hmod).unwrap() };

    let cs_name = OsStr::new(GHOST_WINDOW_CLASS_NAME);
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
    let style = WS_OVERLAPPEDWINDOW | WS_POPUP | WS_DISABLED;
    let size = (0, 0, 0, 0);

    create_window(WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE, cs_ptr, style, size, None, hmod, 0)
}
