#[cfg(windows)]
pub fn get_window_titles() -> Vec<String> {
    use std::{
        ffi::OsString, os::windows::prelude::*, sync::Mutex,
    };
    use winapi::{
        shared::{minwindef, windef}, um::winuser,
    };
    use once_cell::sync::Lazy;
    static TITLES: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(vec![]));
    extern "system" fn callback(hwnd: windef::HWND, _lparam: minwindef::LPARAM) -> i32 {
        if hwnd == std::ptr::null_mut() {
            0
        } else {
            let mut buf = [0; 256];
            let len: i32;
            unsafe {
                len = winuser::GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
            }
            if len < 1 {
                return 1;
            }
            let name = OsString::from_wide(&buf[..len as usize]);
            let str = name.to_str().unwrap().to_string();
            if !str.is_empty() {
                TITLES.lock().unwrap().push(str);
            }
            1
        }
    }
    TITLES.lock().unwrap().clear();
    unsafe { winuser::EnumWindows(Some(callback), 0); }
    TITLES.lock().unwrap().to_vec()
}

#[cfg(not(windows))]
pub fn get_window_titles() -> Vec<String> {
    wmctrl::get_windows()
        .iter()
        .map(|window| String::from(window.title()))
        .collect()
}
