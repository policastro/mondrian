pub mod overlay {
    use crate::modules::overlays::lib::color::Color;
    use crate::modules::overlays::lib::overlay::OverlayParams;
    use crate::win32::api::window::create_window;
    use crate::win32::api::window::get_window_box;
    use crate::win32::api::window::show_window;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::COLORREF;
    use windows::Win32::Foundation::HMODULE;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Foundation::LPARAM;
    use windows::Win32::Foundation::LRESULT;
    use windows::Win32::Foundation::RECT;
    use windows::Win32::Foundation::WPARAM;
    use windows::Win32::Graphics::Gdi::BeginPaint;
    use windows::Win32::Graphics::Gdi::CreatePen;
    use windows::Win32::Graphics::Gdi::DeleteObject;
    use windows::Win32::Graphics::Gdi::EndPaint;
    use windows::Win32::Graphics::Gdi::FillRect;
    use windows::Win32::Graphics::Gdi::GetSysColorBrush;
    use windows::Win32::Graphics::Gdi::InvalidateRect;
    use windows::Win32::Graphics::Gdi::Rectangle;
    use windows::Win32::Graphics::Gdi::SelectObject;
    use windows::Win32::Graphics::Gdi::COLOR_WINDOW;
    use windows::Win32::Graphics::Gdi::PAINTSTRUCT;
    use windows::Win32::Graphics::Gdi::PS_SOLID;
    use windows::Win32::System::LibraryLoader::GetModuleHandleExW;
    use windows::Win32::UI::WindowsAndMessaging::GetClientRect;
    use windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW;
    use windows::Win32::UI::WindowsAndMessaging::PostQuitMessage;
    use windows::Win32::UI::WindowsAndMessaging::SetLayeredWindowAttributes;
    use windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW;
    use windows::Win32::UI::WindowsAndMessaging::CREATESTRUCTW;
    use windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA;
    use windows::Win32::UI::WindowsAndMessaging::HTCAPTION;
    use windows::Win32::UI::WindowsAndMessaging::LWA_COLORKEY;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNOACTIVATE;
    use windows::Win32::UI::WindowsAndMessaging::WM_CREATE;
    use windows::Win32::UI::WindowsAndMessaging::WM_DESTROY;
    use windows::Win32::UI::WindowsAndMessaging::WM_PAINT;
    use windows::Win32::UI::WindowsAndMessaging::WM_QUIT;
    use windows::Win32::UI::WindowsAndMessaging::WM_USER;
    use windows::Win32::UI::WindowsAndMessaging::WS_EX_LAYERED;
    use windows::Win32::UI::WindowsAndMessaging::WS_EX_NOACTIVATE;
    use windows::Win32::UI::WindowsAndMessaging::WS_EX_TOOLWINDOW;
    use windows::Win32::UI::WindowsAndMessaging::WS_EX_TRANSPARENT;
    use windows::Win32::UI::WindowsAndMessaging::WS_POPUP;

    pub const WM_USER_CONFIGURE: u32 = WM_USER + 1;

    pub trait OverlayBase {
        fn get_thickness(&self) -> u8;
        fn get_padding(&self) -> u8;
    }

    pub unsafe extern "system" fn overlay_win_proc(hwnd: HWND, msg: u32, _wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            WM_CREATE => {
                let create_struct = &*(lparam.0 as *const CREATESTRUCTW);
                let custom_value = create_struct.lpCreateParams as *mut OverlayParams;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, custom_value as isize);
                LRESULT(0)
            }
            WM_PAINT => {
                let params = *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverlayParams);
                draw_overlay(hwnd, params.thickness as i32, params.color);
                LRESULT(0)
            }
            WM_USER_CONFIGURE => {
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

    pub fn create<P: OverlayBase + Clone + PartialEq + Send + Copy>(
        params: P,
        target: Option<HWND>,
        class_name: &str,
    ) -> HWND {
        let color_alpha = Color::new(255, 255, 255); // WARNING: only works with white background

        let mut hmod: HMODULE = unsafe { std::mem::zeroed() };
        unsafe { GetModuleHandleExW(0, None, &mut hmod).unwrap() };

        let cs_w: Vec<u16> = OsStr::new(class_name).encode_wide().chain(Some(0)).collect();
        let cs_ptr = PCWSTR(cs_w.as_ptr());

        let ex_style = WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE;
        let style = WS_POPUP;

        let b = get_box_from_target(target.unwrap_or(HWND(0)), params.get_thickness(), params.get_padding());
        let hwnd = create_window(ex_style, cs_ptr, style, b.unwrap_or_default(), target, hmod, params);
        let hwnd = hwnd.unwrap_or(HWND(0));
        show_window(hwnd, SW_SHOWNOACTIVATE);

        unsafe {
            let _ = SetLayeredWindowAttributes(hwnd, COLORREF(color_alpha.into()), 0, LWA_COLORKEY);
        }
        hwnd
    }

    pub fn get_box_from_target(target: HWND, thickness: u8, padding: u8) -> Option<(i32, i32, i32, i32)> {
        let offset = (thickness as i32) / 2;
        let shift1 = offset + (padding as i32);
        let shift2 = offset + 2 * (padding as i32);
        let b = get_window_box(target)?;
        Some((b[0] + 7 - shift1, b[1] - shift1, b[2] - 10 + shift2, b[3] - 5 + shift2))
    }

    pub fn draw_overlay(hwnd: HWND, thickness: i32, color: Color) {
        unsafe {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            let mut rc: RECT = std::mem::zeroed();
            let _ = GetClientRect(hwnd, &mut rc);
            if thickness > 0 {
                let h_pen = CreatePen(PS_SOLID, thickness, COLORREF(color.into()));
                let old_pen = SelectObject(hdc, h_pen);

                let _ = Rectangle(hdc, rc.left, rc.top, rc.right, rc.bottom);

                SelectObject(hdc, old_pen);
                let _ = DeleteObject(h_pen);
            } else {
                let h_brush = GetSysColorBrush(COLOR_WINDOW);
                let _ = FillRect(hdc, &rc, h_brush);
            }

            let _ = EndPaint(hwnd, &ps);
        }
    }
}
