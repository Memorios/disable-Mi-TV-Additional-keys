use std::ptr;
use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM, WPARAM, LRESULT};
use windows_sys::Win32::UI::WindowsAndMessaging::*;
use windows_sys::Win32::System::Threading::GetTickCount;

static mut MOUSE_HOOK: HHOOK = 0;
static mut KEYBOARD_HOOK: HHOOK = 0;
static mut BLOCK_UNTIL: u32 = 0;
const BLOCK_MS: u32 = 400;

unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        if wparam == WM_LBUTTONDOWN as usize || wparam == WM_MBUTTONDOWN as usize {
            BLOCK_UNTIL = GetTickCount() + BLOCK_MS;
        }
    }
    CallNextHookEx(MOUSE_HOOK, code, wparam, lparam)
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && wparam == WM_KEYDOWN as usize {
        let info = *(lparam as *const KBDLLHOOKSTRUCT);
        let now = GetTickCount();
        if now < BLOCK_UNTIL {
            if info.vkCode == VK_RETURN || info.vkCode == VK_APPS {
                return 1; // block fake Enter / Menu
            }
        }
        if info.vkCode == VK_ESCAPE {
            PostQuitMessage(0);
        }
    }
    CallNextHookEx(KEYBOARD_HOOK, code, wparam, lparam)
}

fn main() {
    unsafe {
        let instance = GetModuleHandleW(ptr::null());
        MOUSE_HOOK = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), instance, 0);
        KEYBOARD_HOOK = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), instance, 0);

        println!("Mi TV fake-key blocker is running!");
        println!("→ Left-click & middle-click now work cleanly in Moonlight/Artemis");
        println!("→ Press ESC to quit");

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, 0 as HWND, 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        if MOUSE_HOOK != 0 { UnhookWindowsHookEx(MOUSE_HOOK); }
        if KEYBOARD_HOOK != 0 { UnhookWindowsHookEx(KEYBOARD_HOOK); }
    }
}
