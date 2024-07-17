use windows::Win32::{
    Foundation::{GetLastError, COLORREF, HWND, LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
        CreateWindowExW, GetAncestor, GetForegroundWindow, GetWindowLongPtrW, PostQuitMessage, RegisterClassW,
        SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPos, ShowWindow, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW,
        GA_ROOT, GWLP_USERDATA, HTCAPTION, HWND_TOPMOST, LWA_COLORKEY, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE,
        SWP_NOSENDCHANGING, SWP_SHOWWINDOW, SW_HIDE, WM_CREATE, WM_DESTROY, WM_MOVE, WM_PAINT, WM_QUIT, WM_SHOWWINDOW,
        WNDCLASSW, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_OVERLAPPEDWINDOW, WS_POPUP,
    },
};

use crate::win32::api::window::{get_window_box, is_user_managable_window};

use super::{
    color::Color,
    utils::{create_border, to_pcwstr},
};

#[derive(Debug, Clone, Copy)]
struct WindowOverlayParams {
    color: Color,
    thickness: i32,
    padding: u8,
}

impl WindowOverlayParams {
    fn new(color: Color, thickness: i32, padding: u8) -> WindowOverlayParams {
        WindowOverlayParams {
            color,
            thickness,
            padding,
        }
    }
}

unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, _wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            let create_struct = &*(lparam.0 as *const CREATESTRUCTW);
            let custom_value = create_struct.lpCreateParams as *mut WindowOverlayParams;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, custom_value as isize);
            LRESULT(0)
        }
        WM_PAINT | WM_SHOWWINDOW | WM_MOVE => {
            let params = *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowOverlayParams);
            create_border(hwnd, params.thickness, params.color);
            LRESULT(0)
        }
        WM_DESTROY | WM_QUIT => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => LRESULT(HTCAPTION as isize),
    }
}

pub struct WindowOverlay {
    pub hwnd: HWND,
    params: WindowOverlayParams,
}

impl WindowOverlay {
    pub fn new(thickness: u8, color: Color, padding: u8) -> WindowOverlay {
        let params = WindowOverlayParams::new(color, thickness as i32, padding);
        let hwnd = WindowOverlay::create_overlay(params);
        WindowOverlay { hwnd, params }
    }

    pub fn move_to_foreground(&self) {
        let ancestor = unsafe { GetAncestor(GetForegroundWindow(), GA_ROOT) };
        self.move_to_ref(ancestor);
    }

    pub fn hide(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    fn move_to_ref(&self, ref_hwnd: HWND) {
        if !is_user_managable_window(ref_hwnd, true, true) {
            self.hide();
            return;
        }

        let offset = self.params.thickness / 2;
        let shift1 = offset + (self.params.padding as i32);
        let shift2 = offset + 2 * (self.params.padding as i32);
        let (x, y, cx, cy) = match get_window_box(ref_hwnd) {
            Some(r) => (r[0] + 7 - shift1, r[1] - shift1, r[2] - 10 + shift2, r[3] - 5 + shift2),
            None => return,
        };

        let flags = SWP_NOSENDCHANGING | SWP_SHOWWINDOW | SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE;
        unsafe {
            let _ = SetWindowPos(self.hwnd, HWND_TOPMOST, x, y, cx, cy, flags);
        }
    }

    fn create_overlay(params: WindowOverlayParams) -> HWND {
        let color_white = Color::new(255, 255, 255);
        unsafe {
            let h_instance = GetModuleHandleW(None).unwrap();
            let class = to_pcwstr("Mondrian:BorderFrame");

            let wc = WNDCLASSW {
                hInstance: h_instance.into(),
                lpszClassName: class,
                lpfnWndProc: Some(window_proc),
                style: CS_HREDRAW | CS_VREDRAW,
                ..Default::default()
            };

            let ex_style = WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT | WS_EX_TOPMOST;
            let style = WS_OVERLAPPEDWINDOW | WS_POPUP;

            let data = Some(Box::into_raw(Box::new(params)) as *mut _ as _);
            let mut retry = 5;

            RegisterClassW(&wc);
            let mut hwnd = CreateWindowExW(ex_style, class, None, style, 0, 0, 0, 0, None, None, h_instance, data);
            while retry > 0 && hwnd.0 == 0 {
                let error = GetLastError();
                log::warn!("Overlay window creation failed ({:?}). Retry: {}.", error, retry);
                RegisterClassW(&wc);
                hwnd = CreateWindowExW(ex_style, class, None, style, 0, 0, 0, 0, None, None, h_instance, data);
                retry -= 1;
            }
            let _ = SetLayeredWindowAttributes(hwnd, COLORREF(color_white.into()), 0, LWA_COLORKEY);

            hwnd
        }
    }
}
