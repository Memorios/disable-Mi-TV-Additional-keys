use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use anyhow::Result;
use winapi::um::winuser::{
    SetWindowsHookExW, UnhookWindowsHookEx, CallNextHookEx,
    GetMessageW, TranslateMessage, DispatchMessageW,
    WH_MOUSE_LL, WH_KEYBOARD_LL, WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_KEYDOWN,
    VK_RETURN, VK_APPS,  // Enter and AppsKey (Menu)
    KBDLLHOOKSTRUCT, MOUSEHOOKSTRUCT, HOOKPROC,
    HHOOK, WPARAM, LPARAM, LRESULT, MSG,
};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::handleapi::CloseHandle;
use winapi::shared::minwindef::{DWORD, TRUE};
use winapi::shared::ntdef::NULL;
use kernel32::{GetTickCount, OpenConsoleW, AllocConsole, FreeConsole};

static mut G_MOUSE_HOOK: HHOOK = NULL as HHOOK;
static mut G_KBD_HOOK: HHOOK = NULL as HHOOK;
static mut BLOCK_UNTIL: DWORD = 0;
const BLOCK_MS: u32 = 400;  // 400ms window for Xiaomi injection

fn main() -> Result<()> {
    unsafe {
        AllocConsole();
        OpenConsoleW(ptr::null_mut());

        let h_instance = GetModuleHandleW(ptr::null_mut());
        if h_instance.is_null() {
            anyhow::bail!("Failed to get module handle");
        }

        // Install low-level mouse hook
        let mouse_proc: HOOKPROC = Some(low_level_mouse_proc);
        let mouse_hook = SetWindowsHookExW(
            WH_MOUSE_LL as i32,
            mouse_proc,
            h_instance,
            0,
        );
        if mouse_hook.is_null() {
            anyhow::bail!("Failed to install mouse hook. Run as administrator.");
        }
        G_MOUSE_HOOK = mouse_hook;

        // Install low-level keyboard hook
        let kbd_proc: HOOKPROC = Some(low_level_keyboard_proc);
        let kbd_hook = SetWindowsHookExW(
            WH_KEYBOARD_LL as i32,  // 13i32 == WH_KEYBOARD_LL
            kbd_proc,
            h_instance,
            0,
        );
        if kbd_hook.is_null() {
            anyhow::bail!("Failed to install keyboard hook. Run as administrator.");
        }
        G_KBD_HOOK = kbd_hook;

        println!("Mi TV Key Blocker running... Press Ctrl+C to exit.");
        println!("Tip: Run as administrator for hooks to work.");

        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();
        ctrlc::set_handler(move || {
            r.store(false, Ordering::SeqCst);
        })?;

        let mut msg: MSG = std::mem::zeroed();
        while running.load(Ordering::SeqCst) && GetMessageW(&mut msg, ptr::null_mut(), 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Cleanup
        if !G_MOUSE_HOOK.is_null() {
            UnhookWindowsHookEx(G_MOUSE_HOOK);
        }
        if !G_KBD_HOOK.is_null() {
            UnhookWindowsHookEx(G_KBD_HOOK);
        }
        FreeConsole();
    }
    Ok(())
}

unsafe extern "system" fn low_level_mouse_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code >= 0 {
        let _mouse_struct = &*(l_param as *const MOUSEHOOKSTRUCT);
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
    CallNextHookEx(G_MOUSE_HOOK, n_code, w_param, l_param)
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
    CallNextHookEx(G_KBD_HOOK, n_code, w_param, l_param)
}
