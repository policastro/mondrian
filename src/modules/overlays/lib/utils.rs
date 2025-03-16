pub mod overlay {
    use crate::app::structs::area::Area;
    use crate::modules::overlays::lib::overlay::OverlayParams;
    use crate::win32::api::gdiplus::init_gdiplus;
    use crate::win32::api::window::create_window;
    use crate::win32::api::window::show_window;
    use crate::win32::window::window_obj::WindowObjInfo;
    use crate::win32::window::window_ref::WindowRef;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::COLORREF;
    use windows::Win32::Foundation::HMODULE;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Foundation::LPARAM;
    use windows::Win32::Foundation::LRESULT;
    use windows::Win32::Foundation::POINT;
    use windows::Win32::Foundation::SIZE;
    use windows::Win32::Foundation::WPARAM;
    use windows::Win32::Graphics::Gdi::CreateCompatibleDC;
    use windows::Win32::Graphics::Gdi::CreateDIBSection;
    use windows::Win32::Graphics::Gdi::DeleteDC;
    use windows::Win32::Graphics::Gdi::DeleteObject;
    use windows::Win32::Graphics::Gdi::GetDC;
    use windows::Win32::Graphics::Gdi::ReleaseDC;
    use windows::Win32::Graphics::Gdi::SelectObject;
    use windows::Win32::Graphics::Gdi::AC_SRC_ALPHA;
    use windows::Win32::Graphics::Gdi::AC_SRC_OVER;
    use windows::Win32::Graphics::Gdi::BITMAPINFO;
    use windows::Win32::Graphics::Gdi::BITMAPINFOHEADER;
    use windows::Win32::Graphics::Gdi::BI_RGB;
    use windows::Win32::Graphics::Gdi::BLENDFUNCTION;
    use windows::Win32::Graphics::Gdi::DIB_RGB_COLORS;
    use windows::Win32::Graphics::GdiPlus::FillModeWinding;
    use windows::Win32::Graphics::GdiPlus::GdipAddPathArc;
    use windows::Win32::Graphics::GdiPlus::GdipClosePathFigure;
    use windows::Win32::Graphics::GdiPlus::GdipCreateFromHDC;
    use windows::Win32::Graphics::GdiPlus::GdipCreatePath;
    use windows::Win32::Graphics::GdiPlus::GdipCreatePen1;
    use windows::Win32::Graphics::GdiPlus::GdipCreateSolidFill;
    use windows::Win32::Graphics::GdiPlus::GdipDeleteBrush;
    use windows::Win32::Graphics::GdiPlus::GdipDeleteGraphics;
    use windows::Win32::Graphics::GdiPlus::GdipDeletePath;
    use windows::Win32::Graphics::GdiPlus::GdipDeletePen;
    use windows::Win32::Graphics::GdiPlus::GdipDrawPath;
    use windows::Win32::Graphics::GdiPlus::GdipDrawRectangle;
    use windows::Win32::Graphics::GdiPlus::GdipFillPath;
    use windows::Win32::Graphics::GdiPlus::GdipSetSmoothingMode;
    use windows::Win32::Graphics::GdiPlus::GpGraphics;
    use windows::Win32::Graphics::GdiPlus::GpPath;
    use windows::Win32::Graphics::GdiPlus::GpPen;
    use windows::Win32::Graphics::GdiPlus::GpSolidFill;
    use windows::Win32::Graphics::GdiPlus::SmoothingModeAntiAlias;
    use windows::Win32::Graphics::GdiPlus::UnitPixel;
    use windows::Win32::System::LibraryLoader::GetModuleHandleExW;
    use windows::Win32::UI::WindowsAndMessaging::DefWindowProcW;
    use windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW;
    use windows::Win32::UI::WindowsAndMessaging::PostQuitMessage;
    use windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW;
    use windows::Win32::UI::WindowsAndMessaging::SetWindowPos;
    use windows::Win32::UI::WindowsAndMessaging::UpdateLayeredWindow;
    use windows::Win32::UI::WindowsAndMessaging::CREATESTRUCTW;
    use windows::Win32::UI::WindowsAndMessaging::GWLP_USERDATA;
    use windows::Win32::UI::WindowsAndMessaging::HWND_TOP;
    use windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNOACTIVATE;
    use windows::Win32::UI::WindowsAndMessaging::ULW_ALPHA;
    use windows::Win32::UI::WindowsAndMessaging::WM_CREATE;
    use windows::Win32::UI::WindowsAndMessaging::WM_DESTROY;
    use windows::Win32::UI::WindowsAndMessaging::WM_QUIT;
    use windows::Win32::UI::WindowsAndMessaging::WM_SIZE;
    use windows::Win32::UI::WindowsAndMessaging::WM_USER;
    use windows::Win32::UI::WindowsAndMessaging::WS_EX_LAYERED;
    use windows::Win32::UI::WindowsAndMessaging::WS_EX_NOACTIVATE;
    use windows::Win32::UI::WindowsAndMessaging::WS_EX_TOOLWINDOW;
    use windows::Win32::UI::WindowsAndMessaging::WS_EX_TRANSPARENT;
    use windows::Win32::UI::WindowsAndMessaging::WS_POPUP;

    pub const WM_USER_CONFIGURE: u32 = WM_USER + 1;

    struct OverlayProcState {
        pub graphic_ctx: OverlayGraphicContext,
        pub width: i32,
        pub height: i32,
    }

    impl OverlayProcState {
        pub fn new(graphic_ctx: OverlayGraphicContext, width: i32, height: i32) -> Self {
            OverlayProcState {
                graphic_ctx,
                width,
                height,
            }
        }
    }

    #[derive(Debug, Clone)]
    struct OverlayGraphicContext {
        empty_brush: *mut GpSolidFill,
        color_pen: *mut GpPen,
        thickness: f32,
        border_radius: f32,
    }

    impl From<OverlayParams> for OverlayGraphicContext {
        fn from(params: OverlayParams) -> Self {
            let mut ctx = OverlayGraphicContext {
                empty_brush: ptr::null_mut(),
                color_pen: ptr::null_mut(),
                thickness: params.thickness as f32,
                border_radius: params.border_radius as f32,
            };

            init_gdiplus();

            unsafe {
                GdipCreateSolidFill(0x00000000, &mut ctx.empty_brush);
                GdipCreatePen1(params.color.get_argb(), ctx.thickness, UnitPixel, &mut ctx.color_pen);
            };

            ctx
        }
    }

    impl Drop for OverlayGraphicContext {
        fn drop(&mut self) {
            unsafe {
                GdipDeleteBrush(self.empty_brush as *mut _);
                GdipDeletePen(self.color_pen);
            }
        }
    }

    pub unsafe extern "system" fn overlay_win_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            WM_CREATE => {
                let create_struct = &*(lparam.0 as *const CREATESTRUCTW);
                let proc_params = create_struct.lpCreateParams as *mut OverlayParams;
                let proc_state = Box::new(OverlayProcState::new(
                    (*proc_params).into(),
                    create_struct.cx,
                    create_struct.cy,
                ));
                update_overlay(hwnd, &proc_state.graphic_ctx, proc_state.width, proc_state.height);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(proc_state) as isize);

                LRESULT(0)
            }
            WM_SIZE => {
                let width = (lparam.0 & 0xFFFF) as i32;
                let height = ((lparam.0 >> 16) & 0xFFFF) as i32;
                let proc_state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverlayProcState;
                let proc_state = &*proc_state_ptr;
                update_overlay(hwnd, &proc_state.graphic_ctx, width, height);

                let new_proc_state = Box::new(OverlayProcState::new(proc_state.graphic_ctx.clone(), width, height));
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(new_proc_state) as isize);

                LRESULT(0)
            }
            msg if msg == WM_USER_CONFIGURE => {
                let proc_state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverlayProcState;
                let proc_state = &*proc_state_ptr;
                let (last_width, last_height) = (proc_state.width, proc_state.height);
                if !proc_state_ptr.is_null() {
                    drop(Box::from_raw(proc_state_ptr));
                }
                let proc_params = lparam.0 as *mut OverlayParams;
                let proc_state = Box::new(OverlayProcState::new((*proc_params).into(), last_width, last_height));
                update_overlay(hwnd, &proc_state.graphic_ctx, last_width, last_height);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(proc_state) as isize);
                LRESULT(0)
            }
            WM_DESTROY | WM_QUIT => {
                let proc_params_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverlayProcState;
                if !proc_params_ptr.is_null() {
                    drop(Box::from_raw(proc_params_ptr));
                }
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    pub fn create(params: OverlayParams, target: Option<HWND>, class_name: &str) -> HWND {
        let mut hmod: HMODULE = unsafe { std::mem::zeroed() };
        unsafe { GetModuleHandleExW(0, None, &mut hmod).unwrap() };

        let cs_w: Vec<u16> = OsStr::new(class_name).encode_wide().chain(Some(0)).collect();
        let cs_ptr = PCWSTR(cs_w.as_ptr());

        let ex_style = WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE;
        let style = WS_POPUP;

        let b = get_box_from_target(target.unwrap_or(HWND(0)), params.thickness, params.padding);
        let b = b.unwrap_or_default().into();
        let hwnd = create_window(ex_style, cs_ptr, style, b, target, hmod, params);
        let hwnd = hwnd.unwrap_or(HWND(0));

        show_window(hwnd, SW_SHOWNOACTIVATE);

        hwnd
    }

    pub fn get_box_from_target(target: HWND, thickness: u8, padding: u8) -> Option<Area> {
        let offset = 1.5 * thickness as f32;
        let shift1 = offset.ceil() as i16 + (padding as i16) - 1;
        let shift2 = (2.0 * offset).ceil() as i16 + 2 * (padding as i16) - 2;
        let visible_area = WindowRef::new(target).get_visible_area()?;
        Some(visible_area.shift((-shift1, -shift1, shift2, shift2)))
    }

    pub fn move_to_target(overlay: HWND, target: HWND, params: &OverlayParams) {
        let target_area = get_box_from_target(target, params.thickness, params.padding);
        let (x, y, cx, cy) = match target_area {
            Some(b) => b.into(),
            None => return,
        };

        let flags = SWP_NOACTIVATE;
        let _ = unsafe { SetWindowPos(overlay, HWND_TOP, x, y, cx, cy, flags) };
    }

    fn update_overlay(hwnd: HWND, ctx: &OverlayGraphicContext, width: i32, height: i32) {
        unsafe {
            init_gdiplus();

            let hdc_screen = GetDC(hwnd);
            let hdc_mem = CreateCompatibleDC(hdc_screen);
            let bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut bits: *mut std::ffi::c_void = ptr::null_mut();
            let hbm = CreateDIBSection(hdc_screen, &bmi, DIB_RGB_COLORS, &mut bits as *mut _, None, 0).unwrap();
            let hbm_old = SelectObject(hdc_mem, hbm);

            let mut graphics: *mut GpGraphics = ptr::null_mut();

            GdipCreateFromHDC(hdc_mem, &mut graphics);
            GdipSetSmoothingMode(graphics, SmoothingModeAntiAlias); // TODO: configurable antialias?

            let (x, y, w, h) = (
                ctx.thickness,
                ctx.thickness,
                width as f32 - 2.0 * ctx.thickness,
                height as f32 - 2.0 * ctx.thickness,
            );

            if ctx.border_radius > 0.0 {
                let mut path: *mut GpPath = ptr::null_mut();
                let d = ctx.border_radius * 2.0;
                GdipCreatePath(FillModeWinding, &mut path);
                GdipAddPathArc(path, x, y, d, d, 180.0, 90.0);
                GdipAddPathArc(path, x + w - d, y, d, d, 270.0, 90.0);
                GdipAddPathArc(path, x + w - d, y + h - d, d, d, 0.0, 90.0);
                GdipAddPathArc(path, x, y + h - d, d, d, 90.0, 90.0);
                GdipClosePathFigure(path);
                GdipFillPath(graphics, ctx.empty_brush as *mut _, path);
                GdipDrawPath(graphics, ctx.color_pen, path);
                GdipDeletePath(path);
            } else {
                GdipDrawRectangle(graphics, ctx.color_pen, x, y, w, h);
            }

            let blend = BLENDFUNCTION {
                BlendOp: AC_SRC_OVER as u8,
                BlendFlags: 0,
                SourceConstantAlpha: 255,
                AlphaFormat: AC_SRC_ALPHA as u8,
            };

            let _ = UpdateLayeredWindow(
                hwnd,
                hdc_screen,
                None,
                Some(&SIZE { cx: width, cy: height }),
                hdc_mem,
                Some(&POINT { x: 0, y: 0 }),
                COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            );

            GdipDeleteGraphics(graphics);
            SelectObject(hdc_mem, hbm_old);
            let _ = DeleteObject(hbm);
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(hwnd, hdc_screen);
        }
    }
}
