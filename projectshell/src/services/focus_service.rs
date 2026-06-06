#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    IsIconic, SetForegroundWindow, ShowWindow, SW_RESTORE,
};

pub fn focus_window(hwnd: isize) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if hwnd == 0 {
            return Err("Window handle is empty.".to_owned());
        }

        unsafe {
            if IsIconic(hwnd) != 0 {
                ShowWindow(hwnd, SW_RESTORE);
            }
            if SetForegroundWindow(hwnd) == 0 {
                return Err("SetForegroundWindow failed.".to_owned());
            }
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = hwnd;
        Err("Window focus is only supported on Windows.".to_owned())
    }
}
