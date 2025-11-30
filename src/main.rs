use std::ptr;
use winapi::um::winuser::{
    SetWindowsHookExW, UnhookWindowsHookEx, CallNextHookEx,
    GetMessageW, TranslateMessage, DispatchMessageW,
    WH_MOUSE_LL, WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_KEYDOWN,
    VK_RETURN, VK_APPS,  // Enter and AppsKey (Menu)
    KBDLLHOOKSTRUCT, MOUSEHOOKSTRUCT, HOOKPROC,
    HHOOK, WPARAM, LPARAM, LRESULT, MSG,
};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::shared::minwindef::{DWORD, HKL};
use winapi::shared::ntdef::NULL;
use kernel32::GetTickCount;

static mut G_HOOK: HHOOK = NULL as HHOOK;
static mut BLOCK_UNTIL: DWORD = 0;
const BLOCK_MS: u32 = 400;  // 400ms window for Xiaomi injection

unsafe extern "system" fn low_level_mouse_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code >= 0 {
        let mouse_struct = &*(l_param as *const MOUSEHOOKSTRUCT);
        match w_param as u32 {
            WM_LBUTTONDOWN => {
                BLOCK_UNTIL = GetTickCount() + BLOCK_MS;
            }
            WM_MBUTTONDOWN => {
                BLOCK_UNTIL = GetTickCount() + BLOCK_MS;
            }
            _ => {}
        }
    }
    CallNextHookEx(G_HOOK, n_code, w_param, l_param)
}

unsafe extern "system" fn low_level_keyboard_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code >= 0 && w_param as u32 == WM_KEYDOWN {
        let kbd_struct = &*(l_param as *const KBDLLHOOKSTRUCT);
        let vk_code = kbd_struct.vkCode;
        let now = GetTickCount();
        if now < BLOCK_UNTIL {
            match vk_code as u32 {
                VK_RETURN => return 1,  // Block fake Enter
                VK_APPS => return 1,    // Block fake Menu/AppsKey
                _ => {}
            }
        }
    }
    CallNextHookEx(G_HOOK, n_code, w_param, l_param)
}

fn main() {
    unsafe {
        let h_instance = GetModuleHandleW(ptr::null_mut());
        // Install mouse hook
        let mouse_proc: HOOKPROC = Some(low_level_mouse_proc);
        let mouse_hook = SetWindowsHookExW(
            WH_MOUSE_LL as i32,
            mouse_proc,
            h_instance,
            0,
        );
        if mouse_hook.is_null() {
            panic!("Failed to install mouse hook");
        }
        G_HOOK = mouse_hook;

        // Install keyboard hook (for blocking keys)
        let kbd_proc: HOOKPROC = Some(low_level_keyboard_proc);
        let kbd_hook = SetWindowsHookExW(
            13i32,  // WH_KEYBOARD_LL
            kbd_proc,
            h_instance,
            0,
        );
        if kbd_hook.is_null() {
            panic!("Failed to install keyboard hook");
        }

        println!("Mi TV Key Blocker running... Press Ctrl+C to exit.");
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        UnhookWindowsHookEx(mouse_hook);
        UnhookWindowsHookEx(kbd_hook);
    }
}
