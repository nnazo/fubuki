#[cfg(windows)]
pub fn get_window_titles() -> Vec<String> {
    use std::{
        /*alloc::{Layout, alloc, dealloc}, */ ffi::OsString,
        /*slice::from_raw_parts, */ os::windows::prelude::*, sync::Mutex,
    };
    use winapi::{
        shared::{minwindef, windef},
        um::winuser,
    };
    lazy_static! {
        static ref TITLES: Mutex<Vec<String>> = Mutex::new(vec![]);
    }
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
            let str = name.to_str().unwrap().to_string(); // turns into Option<&str> then String
            if !str.is_empty() {
                // thanks windows
                TITLES.lock().unwrap().push(str);
            }
            1
        }
    }
    TITLES.lock().unwrap().clear();
    unsafe {
        winuser::EnumWindows(Some(callback), 0);
    }
    // let mut copy = Vec::new();
    // let titles: &mut Vec<String> = &mut*(TITLES.lock().unwrap());
    // titles.iter().for_each(|title| copy.push(String::from(title)));        // i have no idea if TITLES being static
    // copy                                                                   // within the function means it's created
    TITLES.lock().unwrap().to_vec() // every time it's called
}

#[cfg(not(windows))]
pub fn get_window_titles() -> Vec<String> {
    wmctrl::get_windows()
        .iter()
        .map(|window| String::from(window.title()))
        .collect()
}
