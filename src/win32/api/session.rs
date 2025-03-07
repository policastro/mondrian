use std::ptr;
use windows::core::PWSTR;
use windows::Win32::System::RemoteDesktop::WTSFreeMemory;
use windows::Win32::System::RemoteDesktop::WTSQuerySessionInformationW;
use windows::Win32::System::RemoteDesktop::WTSSessionInfoEx;
use windows::Win32::System::RemoteDesktop::WTSINFOEXW;
use windows::Win32::System::RemoteDesktop::WTS_CURRENT_SERVER_HANDLE;
use windows::Win32::System::RemoteDesktop::WTS_CURRENT_SESSION;
use windows::Win32::System::RemoteDesktop::WTS_SESSIONSTATE_UNLOCK;

pub fn is_user_logged_in() -> bool {
    let mut buffer: *mut std::ffi::c_void = ptr::null_mut();
    let mut bytes_returned: u32 = 0;

    unsafe {
        let res = WTSQuerySessionInformationW(
            WTS_CURRENT_SERVER_HANDLE,
            WTS_CURRENT_SESSION,
            WTSSessionInfoEx,
            &mut buffer as *mut _ as *mut PWSTR,
            &mut bytes_returned,
        );

        if res.is_ok() && !buffer.is_null() {
            let session_info = *(buffer as *const WTSINFOEXW);
            WTSFreeMemory(buffer);

            return session_info.Data.WTSInfoExLevel1.SessionFlags == WTS_SESSIONSTATE_UNLOCK as i32;
        }
    }
    false
}
