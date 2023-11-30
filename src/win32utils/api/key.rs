use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;

#[derive(Debug, Clone, Copy)]
pub struct KeyState {
    pub pressed: bool,
    pub toggled: bool,
}

pub fn get_key_state(key: u16) -> KeyState {
    let state = unsafe { GetKeyState(key as i32) };
    KeyState {
        pressed: (state & 0x1000) != 0,
        toggled: (state & 0x0001) != 0,
    }
}
