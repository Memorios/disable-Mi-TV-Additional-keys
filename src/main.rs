use std::ptr;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;

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
            if info.vkCode == VK_RETURN.0 || info.vkCode == VK_APPS.0 {
                return LRESULT(1); // BLOCK fake Enter / Menu key
            }
        }

        if info.vkCode == VK_ESCAPE.0 {
            PostQuitMessage(0);
        }
    }
    CallNextHookEx(KEYBOARD_HOOK, code, wparam, lparam)
}

fn main() {
    unsafe {
        let instance = GetModuleHandleW(None).unwrap_or(HINSTANCE(0));

        MOUSE_HOOK = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), instance, 0).unwrap_or(HHOOK(0));
        KEYBOARD_HOOK = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), instance, 0).unwrap_or(HHOOK(0));

        println!("Xiaomi TV fake-key blocker is RUNNING!");
        println!("→ Left click & middle click now work perfectly in Moonlight/Artemis");
        println!("→ Press ESC to quit");

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(0), 0, 0).0 > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        if MOUSE_HOOK.0 != 0 { UnhookWindowsHookEx(MOUSE_HOOK); }
        if KEYBOARD_HOOK.0 != 0 { UnhookWindowsHookEx(KEYBOARD_HOOK); }
    }
}
