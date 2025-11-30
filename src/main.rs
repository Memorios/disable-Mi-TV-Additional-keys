use windows::{
    Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    Win32::UI::Input::KeyboardAndMouse::*,
    Win32::UI::WindowsAndMessaging::*,
};

unsafe extern "system" fn wnd_proc(
    _hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    _lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_INPUT => LRESULT(0),

        WM_APPCOMMAND => {
            let cmd = GET_APPCOMMAND_LPARAM(_lparam);

            match cmd {
                APPCOMMAND_BROWSER_BACKWARD |
                APPCOMMAND_BROWSER_FORWARD |
                APPCOMMAND_VOLUME_MUTE |
                APPCOMMAND_VOLUME_DOWN |
                APPCOMMAND_VOLUME_UP |
                APPCOMMAND_MEDIA_PLAY_PAUSE |
                APPCOMMAND_MEDIA_NEXTTRACK |
                APPCOMMAND_MEDIA_PREVIOUSTRACK => {
                    println!("Blocked key: {}", cmd);
                    return LRESULT(1);
                }
                _ => {}
            }
        }

        WM_DESTROY => {
            PostQuitMessage(0);
        }

        _ => {}
    }

    DefWindowProcW(_hwnd, msg, wparam, _lparam)
}

fn main() {
    unsafe {
        let class_name = w!("DisableMiTVKeysClass");

        let wc = WNDCLASSW {
            lpfnWndProc: Some(wnd_proc),
            lpszClassName: class_name,
            ..Default::default()
        };

        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            Default::default(),
            class_name,
            w!("Disable Mi TV Extra Keys"),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            300,
            200,
            None,
            None,
            None,
            None,
        );

        ShowWindow(hwnd, SW_HIDE);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(0), 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

/// Extract APPCOMMAND from lparam
fn GET_APPCOMMAND_LPARAM(lparam: LPARAM) -> i32 {
    ((lparam.0 >> 16) & !FAPPCOMMAND_MASK.0) as i32
}
