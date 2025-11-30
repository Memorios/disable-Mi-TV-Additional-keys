// disable_mi_tv_keys — block ENTER and APPS key injection after mouse click

use std::ptr::null_mut;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::thread;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_MBUTTON, VK_RETURN, VK_APPS};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SetWindowsHookExW, UnhookWindowsHookEx, CallNextHookEx,
    WH_KEYBOARD_LL, WH_MOUSE_LL, KBDLLHOOKSTRUCT, MSLLHOOKSTRUCT,
    WM_KEYDOWN, WM_SYSKEYDOWN,
    GetMessageW, TranslateMessage, DispatchMessageW, MSG,
    GetModuleHandleW, DefWindowProcW
};

fn now_millis() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}

fn main() {
    let suppress_ms: u64 = 200;
    let suppress_until = Arc::new(AtomicU64::new(0));

    // Poll thread to detect left or middle mouse down
    {
        let suppress = suppress_until.clone();
        thread::spawn(move || {
            let mut prev_left = 0i16;
            let mut prev_mid = 0i16;
            loop {
                let left = unsafe { GetAsyncKeyState(VK_LBUTTON as i32) };
                let mid = unsafe { GetAsyncKeyState(VK_MBUTTON as i32) };
                if (left & 0x8000 != 0) && (prev_left & 0x8000 == 0) {
                    suppress.store(now_millis() + suppress_ms, Ordering::Relaxed);
                }
                if (mid & 0x8000 != 0) && (prev_mid & 0x8000 == 0) {
                    suppress.store(now_millis() + suppress_ms, Ordering::Relaxed);
                }
                prev_left = left;
                prev_mid = mid;
                thread::sleep(std::time::Duration::from_millis(5));
            }
        });
    }

    unsafe {
        let hinstance = GetModuleHandleW(null_mut());

        extern "system" fn keyboard_proc(n_code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
            if n_code >= 0 && (wparam as u32 == WM_KEYDOWN || wparam as u32 == WM_SYSKEYDOWN) {
                let kb = *(lparam as *const KBDLLHOOKSTRUCT);
                let vk = kb.vkCode as i32;
                let until = unsafe { SUPPRESS_UNTIL.load(Ordering::Relaxed) };
                if now_millis() <= until {
                    if vk == VK_RETURN as i32 || vk == VK_APPS as i32 {
                        return 1; // consume
                    }
                }
            }
            CallNextHookEx(0, n_code, wparam, lparam)
        }

        static mut SUPPRESS_UNTIL: AtomicU64 = AtomicU64::new(0);
        SUPPRESS_UNTIL = (*(&*suppress_until));

        let _hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), hinstance, 0);
        let mut msg = MSG { hwnd: 0, message: 0, wParam: 0, lParam: 0, time: 0, pt: Default::default() };
        while GetMessageW(&mut msg, 0, 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        UnhookWindowsHookEx(_hook);
    }
}
