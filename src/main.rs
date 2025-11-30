use std::ptr;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

static mut MOUSE_HOOK: HHOOK = HHOOK(0);
static mut KEYBOARD_HOOK: HHOOK = HHOOK(0);
static mut BLOCK_UNTIL: u32 = 0;
const BLOCK_MS: u32 = 400;

unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        if wparam.0 == WM_LBUTTONDOWN.0 || wparam.0 == WM_MBUTTONDOWN.0 {
            BLOCK_UNTIL = GetTickCount() + BLOCK_MS;
        }
    }
    CallNextHookEx(MOUSE_HOOK, code, wparam, lparam)
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && wparam.0 == WM_KEYDOWN.0 {
        let info = *(lparam.0 as *const KBDLLHOOKSTRUCT);
        let now = GetTickCount();

        if now < BLOCK_UNTIL {
            if info.vkCode == 0x0D || info.vkCode == 0x5D {  // 0x0D = Enter, 0x5D = AppsKey
                return LRESULT(1);
            }
        }

        if info.vkCode == 0x1B {  // ESC to quit
            PostQuitMessage(0);
        }
    }
    CallNextHookEx(KEYBOARD_HOOK, code, wparam, lparam)
}

fn main() {
    unsafe {
        let instance = GetModuleHandleW(None).unwrap_or(HINSTANCE(ptr::null_mut()));

        MOUSE_HOOK = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), instance, 0)
            .unwrap_or(HHOOK(ptr::null_mut()));
        KEYBOARD_HOOK = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), instance, 0)
            .unwrap_or(HHOOK(ptr::null_mut()));

        println!("Xiaomi TV fake-key blocker is RUNNING");
        println!("Left & middle click now work perfectly");
        println!("Press ESC to quit");

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(ptr::null_mut()), 0, 0).0 > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        if MOUSE_HOOK.0 != ptr::null_mut() { UnhookWindowsHookEx(MOUSE_HOOK); }
        if KEYBOARD_HOOK.0 != ptr::null_mut() { UnhookWindowsHookEx(KEYBOARD_HOOK); }
    }
}
