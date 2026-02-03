use std::ptr;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let kb_struct = unsafe { *(l_param as *const KBDLLHOOKSTRUCT) };
        let scan_code = kb_struct.scanCode;

        let event = w_param as u32;
        if event == WM_KEYDOWN || event == WM_SYSKEYDOWN {
            println!("Key Down - ScanCode: 0x{:02X}", scan_code);
        } else if event == WM_KEYUP || event == WM_SYSKEYUP {
            println!("Key Up   - ScanCode: 0x{:02X}", scan_code);
        }
    }
    unsafe { CallNextHookEx(ptr::null_mut(), n_code, w_param, l_param) }
}

fn main() -> anyhow::Result<()> {
    unsafe {
        let h_hook = unsafe {
            SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(low_level_keyboard_proc),
                ptr::null_mut(),
                0,
            )
        };

        if h_hook == ptr::null_mut() {
            let err = GetLastError();
            return Err(anyhow::anyhow!("Failed to set hook: {}", err));
        }

        println!("Hook set! Press keys to see ScanCodes. Exit with Ctrl+C (in terminal).");

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        UnhookWindowsHookEx(h_hook);
    }
    Ok(())
}
