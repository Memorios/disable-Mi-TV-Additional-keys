// src/main.rs
// disable-keys: left+middle mouse triggered temporary blocker for ENTER + APPS
// Variant: only left and middle mouse buttons open suppression window.

use std::ptr::null_mut;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{thread};

use windows::Win32::Foundation::{LRESULT, WPARAM, LPARAM, HINSTANCE};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, TranslateMessage, DispatchMessageW, MSG, WH_KEYBOARD_LL,
    HC_ACTION, WM_KEYDOWN, WM_SYSKEYDOWN, WM_KEYUP, WM_SYSKEYUP,
    VK_RETURN, VK_APPS, VK_CONTROL, VK_MENU, VK_F12,
    UnhookWindowsHookEx, HHOOK, HWND,
    LowLevelKeyboardProc,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows::Win32::UI::WindowsAndMessaging::{VK_LBUTTON, VK_MBUTTON};

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0))
        .as_millis() as u64
}

fn main() {
    // CONFIG: suppression window after left/middle mouse-down, in ms
    let suppress_ms: u64 = 180;

    // Shared atomic state
    let suppress_until = Arc::new(AtomicU64::new(0));
    let enabled = Arc::new(AtomicU64::new(1)); // 1 = enabled, 0 = disabled

    // Clone for poll thread
    let suppress_from_poll = suppress_until.clone();
    let enabled_poll = enabled.clone();

    // Poll thread to detect mouse button DOWN transitions (left + middle only).
    thread::spawn(move || {
        let mut prev_left = 0i16;
        let mut prev_mid = 0i16;
        loop {
            if enabled_poll.load(Ordering::Relaxed) == 0 {
                thread::sleep(Duration::from_millis(50));
                continue;
            }
            unsafe {
                let left = GetAsyncKeyState(VK_LBUTTON as i32);
                let mid = GetAsyncKeyState(VK_MBUTTON as i32);
                // Detect transition: not pressed -> pressed
                if (left & 0x8000 != 0) && (prev_left & 0x8000 == 0) {
                    // left button down
                    suppress_from_poll.store(now_millis() + suppress_ms, Ordering::Relaxed);
                }
                if (mid & 0x8000 != 0) && (prev_mid & 0x8000 == 0) {
                    // middle button down
                    suppress_from_poll.store(now_millis() + suppress_ms, Ordering::Relaxed);
                }
                prev_left = left;
                prev_mid = mid;
            }
            // Poll interval: responsive yet light CPU usage
            thread::sleep(Duration::from_millis(6));
        }
    });

    // Keyboard hook in main thread (message pump needed to keep hook alive)
    unsafe {
        let hinstance: HINSTANCE = GetModuleHandleW(None).unwrap_or_default();

        extern "system" fn kb_proc(n_code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
            keyboard_callback(n_code, wparam, lparam)
        }

        static mut KEY_HOOK: HHOOK = HHOOK(0);
        KEY_HOOK = windows::Win32::UI::WindowsAndMessaging::SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(kb_proc),
            hinstance,
            0,
        )
        .unwrap_or(HHOOK(0));

        // Store pointers to atomic variables in static for callbacks
        set_global_state(suppress_until.clone(), enabled.clone());

        // Message pump to keep hook alive
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(0), 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        if KEY_HOOK.0 != 0 {
            let _ = UnhookWindowsHookEx(KEY_HOOK);
        }
    }

    // main never returns normally because message loop runs; if it does, keep threads alive
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

// ---- Global static pointers for callback access ----
static mut GLOBAL_SUPPRESS_PTR: *const AtomicU64 = std::ptr::null();
static mut GLOBAL_ENABLED_PTR: *const AtomicU64 = std::ptr::null();

fn set_global_state(suppress: Arc<AtomicU64>, enabled: Arc<AtomicU64>) {
    // Leak arcs to static pointers for lifetime of program
    let s = Arc::into_raw(suppress);
    let e = Arc::into_raw(enabled);
    unsafe {
        GLOBAL_SUPPRESS_PTR = s;
        GLOBAL_ENABLED_PTR = e;
    }
}

fn global_suppress_until() -> u64 {
    unsafe {
        if GLOBAL_SUPPRESS_PTR.is_null() {
            return 0;
        }
        (*GLOBAL_SUPPRESS_PTR).load(Ordering::Relaxed)
    }
}

fn global_enabled() -> bool {
    unsafe {
        if GLOBAL_ENABLED_PTR.is_null() {
            return true;
        }
        (*GLOBAL_ENABLED_PTR).load(Ordering::Relaxed) != 0
    }
}

// keyboard hook callback
extern "system" fn keyboard_callback(n_code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    use windows::Win32::UI::WindowsAndMessaging::{KBDLLHOOKSTRUCT, PKBDLLHOOKSTRUCT};
    if n_code != HC_ACTION {
        unsafe { return CallNextHookEx(None, n_code, wparam, lparam); }
    }

    // read keyboard struct
    let kb_struct = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
    let vk = kb_struct.vkCode as i32;

    // Detect hotkey Ctrl+Alt+F12 (toggle)
    unsafe {
        let ctrl_down = (GetAsyncKeyState(VK_CONTROL as i32) & 0x8000) != 0;
        let alt_down = (GetAsyncKeyState(VK_MENU as i32) & 0x8000) != 0;
        if ctrl_down && alt_down && vk == VK_F12.0 as i32 &&
           (wparam.0 as u32 == WM_KEYDOWN || wparam.0 as u32 == WM_SYSKEYDOWN) {
            // toggle enabled
            if global_enabled() {
                (*GLOBAL_ENABLED_PTR).store(0, Ordering::Relaxed);
            } else {
                (*GLOBAL_ENABLED_PTR).store(1, Ordering::Relaxed);
            }
            // swallow toggle key so it doesn't reach other apps
            return LRESULT(1);
        }
    }

    // If globally disabled, forward
    if !global_enabled() {
        unsafe { return CallNextHookEx(None, n_code, wparam, lparam); }
    }

    // If within suppression window, swallow Enter or Apps keys
    let until = global_suppress_until();
    let now = now_millis();
    if now <= until {
        // VK_RETURN and VK_APPS
        if vk == VK_RETURN.0 as i32 || vk == VK_APPS.0 as i32 {
            // swallow
            return LRESULT(1);
        }
    }

    // forward other keys
    unsafe { CallNextHookEx(None, n_code, wparam, lparam) }
}
