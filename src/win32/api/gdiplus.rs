use std::ptr;
use std::sync::OnceLock;
use windows::Win32::Foundation::FALSE;
use windows::Win32::Graphics::GdiPlus::GdiplusShutdown;
use windows::Win32::Graphics::GdiPlus::GdiplusStartup;
use windows::Win32::Graphics::GdiPlus::GdiplusStartupInput;

static GDIPLUS_TOKEN: OnceLock<usize> = OnceLock::new();

pub fn init_gdiplus() {
    GDIPLUS_TOKEN.get_or_init(|| {
        let mut token: usize = 0;
        let startup_input = GdiplusStartupInput {
            GdiplusVersion: 1,
            DebugEventCallback: 0,
            SuppressBackgroundThread: FALSE,
            SuppressExternalCodecs: FALSE,
        };

        unsafe { GdiplusStartup(&mut token, &startup_input, ptr::null_mut()) };

        token
    });
}

pub fn shutdown_gdiplus() {
    if let Some(token) = GDIPLUS_TOKEN.get() {
        unsafe { GdiplusShutdown(*token) };
    }
}
