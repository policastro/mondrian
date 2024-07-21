pub mod overlay {
    use windows::Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, CreatePen, DeleteObject, EndPaint, Rectangle, SelectObject, PAINTSTRUCT, PS_SOLID,
        },
        UI::WindowsAndMessaging::{
            GetClientRect, GetWindowLongPtrW, PostQuitMessage, SetWindowLongPtrW, CREATESTRUCTW, GWLP_USERDATA,
            HTCAPTION, WM_CREATE, WM_DESTROY, WM_PAINT,
        },
    };

    use windows::Win32::{
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{WNDCLASSW, WS_VISIBLE},
    };

    use windows::Win32::{
        Foundation::GetLastError,
        UI::WindowsAndMessaging::{
            CreateWindowExW, RegisterClassW, SetLayeredWindowAttributes, CS_HREDRAW, CS_VREDRAW, LWA_COLORKEY,
            WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_OVERLAPPEDWINDOW, WS_POPUP,
        },
    };

    use windows::Win32::{
        Graphics::Gdi::InvalidateRect,
        UI::WindowsAndMessaging::{WM_QUIT, WM_USER},
    };

    use crate::{
        modules::overlays::lib::{color::Color, overlay::OverlayParams},
        win32::api::{misc::str_to_pcwstr, window::get_window_box},
    };

    pub const WM_CHANGE_BORDER: u32 = WM_USER + 1;
    unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, _wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            WM_CREATE => {
                let create_struct = &*(lparam.0 as *const CREATESTRUCTW);
                let custom_value = create_struct.lpCreateParams as *mut OverlayParams;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, custom_value as isize);
                LRESULT(0)
            }
            WM_PAINT => {
                let params = *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverlayParams);
                draw_border(hwnd, params.thickness as i32, params.color);
                LRESULT(0)
            }
            WM_CHANGE_BORDER => {
                let params = lparam.0 as *mut OverlayParams;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, params as isize);
                let _ = InvalidateRect(hwnd, None, false);
                LRESULT(0)
            }
            WM_DESTROY | WM_QUIT => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => LRESULT(HTCAPTION as isize),
        }
    }

    pub fn create(params: OverlayParams, target: Option<HWND>) -> HWND {
        let color_white = Color::new(255, 255, 255);
        unsafe {
            let handle = GetModuleHandleW(None).unwrap();
            let class = str_to_pcwstr("Mondrian:BorderFrame");

            let wc = WNDCLASSW {
                hInstance: handle.into(),
                lpszClassName: class,
                lpfnWndProc: Some(window_proc),
                style: CS_HREDRAW | CS_VREDRAW,
                ..Default::default()
            };

            let ex_style = WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT;
            let style = WS_OVERLAPPEDWINDOW | WS_POPUP | WS_VISIBLE;

            let data = Some(Box::into_raw(Box::new(params)) as *mut _ as _);
            let parent = target.unwrap_or(HWND(0));

            let mut hwnd = HWND(0);
            let mut retry = 5;
            while retry > 0 && hwnd.0 == 0 {
                RegisterClassW(&wc);
                let b = get_box_from_target(target.unwrap_or(HWND(0)), params.thickness, params.padding)
                    .unwrap_or_default();
                hwnd = CreateWindowExW(
                    ex_style, class, None, style, b.0, b.1, b.2, b.3, parent, None, handle, data,
                );
                if hwnd.0 == 0 {
                    retry -= 1;
                    let error = GetLastError();
                    log::warn!("Overlay window creation failed ({:?}). Retry: {}.", error, retry);
                }
            }
            let _ = SetLayeredWindowAttributes(hwnd, COLORREF(color_white.into()), 0, LWA_COLORKEY);
            hwnd
        }
    }

    pub fn get_box_from_target(target: HWND, thickness: u8, padding: u8) -> Option<(i32, i32, i32, i32)> {
        let offset = (thickness as i32) / 2;
        let shift1 = offset + (padding as i32);
        let shift2 = offset + 2 * (padding as i32);
        let b = get_window_box(target)?;
        Some((b[0] + 7 - shift1, b[1] - shift1, b[2] - 10 + shift2, b[3] - 5 + shift2))
    }

    pub fn draw_border(hwnd: HWND, thickness: i32, color: Color) {
        unsafe {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);
            let mut rc: RECT = std::mem::zeroed();
            let _ = GetClientRect(hwnd, &mut rc);

            let h_pen = CreatePen(PS_SOLID, thickness, COLORREF(color.into()));
            let old_pen = SelectObject(hdc, h_pen);

            let _ = Rectangle(hdc, rc.left, rc.top, rc.right, rc.bottom);

            SelectObject(hdc, old_pen);
            let _ = DeleteObject(h_pen);

            let _ = EndPaint(hwnd, &ps);
        }
    }
}
