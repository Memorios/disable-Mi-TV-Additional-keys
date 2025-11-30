use std::ptr;
use winapi::um::winuser::*;
use winapi::um::libloaderapi::GetModuleHandleW;

static mut MOUSE_HOOK: HHOOK = 0 as HHOOK;
static mut KEYBOARD_HOOK: HHOOK = 0 as HHOOK;
static mut BLOCK_UNTIL: u32 = 0;
const BLOCK_MS: u32 = 400;

unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        if wparam == WM_LBUTTONDOWN as usize || wparam == WM_MBUTTONDOWN as usize {
            BLOCK_UNTIL = winapi::um::winbase::GetTickCount() + BLOCK_MS;
        }
    }
    CallNextHookEx(MOUSE_HOOK, code, wparam, lparam)
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && wparam == WM_KEYDOWN as usize {
        let info = *(lparam as *const KBDLLHOOKSTRUCT);
        let now = winapi::um::winbase::GetTickCount();

        if now < BLOCK_UNTIL {
            if info.vkCode == 0x0D || info.vkCode == 0x5D {  // Enter or AppsKey
                return 1;
            }
        }

        if info.vkCode == 0x1B {  // ESC = quit
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

        println!("Xiaomi TV fake-key blocker RUNNING");
        println!("Left + middle click = clean");
        println!("Press ESC to quit");

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
