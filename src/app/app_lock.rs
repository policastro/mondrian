use windows::core::PCWSTR;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::CreateFileW;
use windows::Win32::Storage::FileSystem::FILE_FLAG_BACKUP_SEMANTICS;
use windows::Win32::Storage::FileSystem::FILE_GENERIC_WRITE;
use windows::Win32::Storage::FileSystem::FILE_SHARE_MODE;
use windows::Win32::Storage::FileSystem::OPEN_ALWAYS;

const LOCK_FILE_PATH: &str = "mondrian.lock";

pub struct AppLock {
    file_lock: HANDLE,
}

impl AppLock {
    pub fn init() -> Result<Self, String> {
        Ok(Self {
            file_lock: get_lock_file().map_err(|e| e.to_string())?,
        })
    }
}

impl Drop for AppLock {
    fn drop(&mut self) {
        if unsafe { CloseHandle(self.file_lock) }.is_ok() {
            std::fs::remove_file(LOCK_FILE_PATH).ok();
        }
    }
}

fn get_lock_file() -> Result<HANDLE, windows::core::Error> {
    let lock_file_w: Vec<u16> = LOCK_FILE_PATH.encode_utf16().chain(Some(0)).collect();
    unsafe {
        CreateFileW(
            PCWSTR(lock_file_w.as_ptr()),
            FILE_GENERIC_WRITE.0,
            FILE_SHARE_MODE(0),
            None,
            OPEN_ALWAYS,
            FILE_FLAG_BACKUP_SEMANTICS,
            None,
        )
    }
}
