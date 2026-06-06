use std::mem;
use std::process;
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::thread;
use std::time::Duration;

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, MOD_WIN, VK_OEM_3,
};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CallWindowProcW, CreatePopupMenu, DestroyMenu, GetCursorPos, GetWindowLongPtrW,
    LoadIconW, PostMessageW, PostQuitMessage, SetForegroundWindow, SetWindowLongPtrW, ShowWindow,
    TrackPopupMenu, GWLP_WNDPROC, IDI_APPLICATION, MF_SEPARATOR, MF_STRING, SW_HIDE, SW_RESTORE,
    TPM_RETURNCMD, TPM_RIGHTBUTTON, WM_CLOSE, WM_HOTKEY, WM_LBUTTONUP, WM_RBUTTONUP, WM_USER,
    WNDPROC,
};

use super::log_service;

const HOTKEY_ID: i32 = 0x5053;
const TRAY_ID: u32 = 0x5053;
const TRAY_CALLBACK_MESSAGE: u32 = WM_USER + 42;
const MENU_OPEN: usize = 1;
const MENU_EXIT: usize = 2;

static ORIGINAL_WNDPROC: AtomicIsize = AtomicIsize::new(0);
static SHOW_REQUESTED: AtomicBool = AtomicBool::new(false);
static EXIT_REQUESTED: AtomicBool = AtomicBool::new(false);
static INSTALLED_HWND: AtomicIsize = AtomicIsize::new(0);

#[derive(Debug)]
pub struct ResidentController {
    hwnd: HWND,
    tray_added: bool,
    hotkey_registered: bool,
    subclassed: bool,
}

impl ResidentController {
    pub fn install(hwnd: isize) -> Result<Self, String> {
        if hwnd == 0 {
            return Err("Window handle is unavailable.".to_owned());
        }

        let hwnd = hwnd as HWND;
        let mut controller = Self {
            hwnd,
            tray_added: false,
            hotkey_registered: false,
            subclassed: false,
        };

        controller.subclass_window()?;
        controller.add_tray_icon()?;
        controller.register_hotkey()?;
        INSTALLED_HWND.store(hwnd, Ordering::SeqCst);
        Ok(controller)
    }

    pub fn hide_window(&self) {
        unsafe {
            ShowWindow(self.hwnd, SW_HIDE);
        }
    }

    pub fn show_window(&self) {
        show_projectshell_window(self.hwnd);
    }

    fn subclass_window(&mut self) -> Result<(), String> {
        unsafe {
            let previous = GetWindowLongPtrW(self.hwnd, GWLP_WNDPROC);
            if previous == 0 {
                return Err("Failed to read window procedure.".to_owned());
            }
            ORIGINAL_WNDPROC.store(previous, Ordering::SeqCst);
            let replaced = SetWindowLongPtrW(
                self.hwnd,
                GWLP_WNDPROC,
                resident_wnd_proc as *const () as isize,
            );
            if replaced == 0 {
                return Err("Failed to install resident window procedure.".to_owned());
            }
            self.subclassed = true;
        }
        Ok(())
    }

    fn register_hotkey(&mut self) -> Result<(), String> {
        let ok = unsafe { RegisterHotKey(self.hwnd, HOTKEY_ID, MOD_WIN, VK_OEM_3 as u32) };
        if ok == 0 {
            return Err("Failed to register Win+` hotkey. It may already be reserved.".to_owned());
        }
        self.hotkey_registered = true;
        Ok(())
    }

    fn add_tray_icon(&mut self) -> Result<(), String> {
        let mut data = notify_icon_data(self.hwnd);
        data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        data.uCallbackMessage = TRAY_CALLBACK_MESSAGE;
        data.hIcon = unsafe { LoadIconW(0, IDI_APPLICATION) };
        write_wide_fixed(&mut data.szTip, "ProjectShell");

        let ok = unsafe { Shell_NotifyIconW(NIM_ADD, &data) };
        if ok == 0 {
            return Err("Failed to add ProjectShell tray icon.".to_owned());
        }
        self.tray_added = true;
        Ok(())
    }
}

impl Drop for ResidentController {
    fn drop(&mut self) {
        unsafe {
            if self.hotkey_registered {
                UnregisterHotKey(self.hwnd, HOTKEY_ID);
            }
            if self.tray_added {
                let data = notify_icon_data(self.hwnd);
                Shell_NotifyIconW(NIM_DELETE, &data);
            }
            if self.subclassed {
                let original = ORIGINAL_WNDPROC.swap(0, Ordering::SeqCst);
                if original != 0 {
                    SetWindowLongPtrW(self.hwnd, GWLP_WNDPROC, original);
                }
            }
        }
    }
}

pub fn consume_show_request() -> bool {
    SHOW_REQUESTED.swap(false, Ordering::SeqCst)
}

pub fn consume_exit_request() -> bool {
    EXIT_REQUESTED.swap(false, Ordering::SeqCst)
}

fn request_show(hwnd: HWND) {
    SHOW_REQUESTED.store(true, Ordering::SeqCst);
    show_projectshell_window(hwnd);
}

fn request_exit(hwnd: HWND) {
    EXIT_REQUESTED.store(true, Ordering::SeqCst);
    cleanup_resident_resources(hwnd);
    unsafe {
        PostMessageW(hwnd, WM_CLOSE, 0, 0);
        PostQuitMessage(0);
    }
    thread::spawn(|| {
        thread::sleep(Duration::from_millis(350));
        process::exit(0);
    });
}

fn show_projectshell_window(hwnd: HWND) {
    unsafe {
        ShowWindow(hwnd, SW_RESTORE);
        SetForegroundWindow(hwnd);
    }
}

fn cleanup_resident_resources(hwnd: HWND) {
    unsafe {
        UnregisterHotKey(hwnd, HOTKEY_ID);
        let data = notify_icon_data(hwnd);
        Shell_NotifyIconW(NIM_DELETE, &data);

        let original = ORIGINAL_WNDPROC.swap(0, Ordering::SeqCst);
        if original != 0 {
            SetWindowLongPtrW(hwnd, GWLP_WNDPROC, original);
        }
        INSTALLED_HWND.store(0, Ordering::SeqCst);
    }
}

unsafe extern "system" fn resident_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_HOTKEY if wparam as i32 == HOTKEY_ID => {
            request_show(hwnd);
            return 0;
        }
        TRAY_CALLBACK_MESSAGE => {
            match lparam as u32 {
                WM_LBUTTONUP => request_show(hwnd),
                WM_RBUTTONUP => show_tray_menu(hwnd),
                _ => {}
            }
            return 0;
        }
        _ => {}
    }

    let original = ORIGINAL_WNDPROC.load(Ordering::SeqCst);
    if original != 0 {
        let proc: WNDPROC = mem::transmute(original);
        CallWindowProcW(proc, hwnd, msg, wparam, lparam)
    } else {
        windows_sys::Win32::UI::WindowsAndMessaging::DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

fn show_tray_menu(hwnd: HWND) {
    unsafe {
        let menu = CreatePopupMenu();
        if menu == 0 {
            log_service::log_error("Failed to create tray menu.");
            return;
        }

        let open = wide_null("Open ProjectShell");
        let exit = wide_null("Exit");
        AppendMenuW(menu, MF_STRING, MENU_OPEN, open.as_ptr());
        AppendMenuW(menu, MF_SEPARATOR, 0, ptr::null());
        AppendMenuW(menu, MF_STRING, MENU_EXIT, exit.as_ptr());

        let mut point = POINT { x: 0, y: 0 };
        GetCursorPos(&mut point);
        SetForegroundWindow(hwnd);
        let selected = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_RIGHTBUTTON,
            point.x,
            point.y,
            0,
            hwnd,
            ptr::null(),
        );
        DestroyMenu(menu);

        match selected as usize {
            MENU_OPEN => request_show(hwnd),
            MENU_EXIT => request_exit(hwnd),
            _ => {}
        }
    }
}

fn notify_icon_data(hwnd: HWND) -> NOTIFYICONDATAW {
    let mut data = unsafe { mem::zeroed::<NOTIFYICONDATAW>() };
    data.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = hwnd;
    data.uID = TRAY_ID;
    data
}

fn write_wide_fixed(target: &mut [u16], value: &str) {
    let wide = value.encode_utf16().collect::<Vec<_>>();
    for (index, code) in wide.iter().take(target.len().saturating_sub(1)).enumerate() {
        target[index] = *code;
    }
}

fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}
