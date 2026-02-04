use profile::{Config, Profile};
use std::ptr;
use std::sync::OnceLock;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

static CURRENT_PROFILE: OnceLock<Profile> = OnceLock::new();

unsafe fn send_key_event(scan_code: u16, is_key_down: bool, is_sys_key: bool) {
    let mut input: INPUT = unsafe { std::mem::zeroed() };
    input.r#type = INPUT_KEYBOARD;

    let mut flags = KEYEVENTF_SCANCODE;
    if !is_key_down {
        flags |= KEYEVENTF_KEYUP;
    }
    // We don't strictly need to distinguish SYSKEY here for SendInput usually,
    // but we can set KEYEVENTF_EXTENDEDKEY if needed.

    input.Anonymous.ki.wScan = scan_code;
    input.Anonymous.ki.dwFlags = flags;
    input.Anonymous.ki.time = 0;
    input.Anonymous.ki.dwExtraInfo = 0;

    unsafe {
        SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
    }
}

unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let kb_struct = unsafe { *(l_param as *const KBDLLHOOKSTRUCT) };

        // Skip injected events to avoid infinite loops
        if (kb_struct.flags & LLKHF_INJECTED) != 0 {
            return unsafe { CallNextHookEx(ptr::null_mut(), n_code, w_param, l_param) };
        }

        let scan_code = kb_struct.scanCode;
        let event = w_param as u32;
        let is_key_down = event == WM_KEYDOWN || event == WM_SYSKEYDOWN;
        let is_sys_key = event == WM_SYSKEYDOWN || event == WM_SYSKEYUP;

        if let Some(profile) = CURRENT_PROFILE.get() {
            let scan_code_str = format!("0x{:02X}", scan_code);
            if let Some(target_key_name) = profile.keys.get(&scan_code_str) {
                // For now, assume target_key_name is a hex string "0x.."
                if let Ok(target_scan_code) =
                    u16::from_str_radix(target_key_name.trim_start_matches("0x"), 16)
                {
                    println!(
                        "Remapping 0x{:02X} -> 0x{:02X}",
                        scan_code, target_scan_code
                    );
                    unsafe {
                        send_key_event(target_scan_code, is_key_down, is_sys_key);
                    }
                    return 1; // Block original input
                }
            }
        }

        if is_key_down {
            println!("Key Down - ScanCode: 0x{:02X}", scan_code);
        } else {
            println!("Key Up   - ScanCode: 0x{:02X}", scan_code);
        }
    }
    unsafe { CallNextHookEx(ptr::null_mut(), n_code, w_param, l_param) }
}

fn main() -> anyhow::Result<()> {
    // Load config and profile
    let config = Config::load_from_file("config.toml")?;
    let default_profile_path = config
        .profiles
        .get(&config.default_profile.default)
        .ok_or_else(|| anyhow::anyhow!("Default profile not found in config"))?;

    let profile = Profile::load_from_file(default_profile_path)?;
    println!("Loaded profile: {}", profile.name);
    CURRENT_PROFILE
        .set(profile)
        .map_err(|_| anyhow::anyhow!("Failed to set global profile"))?;

    unsafe {
        let h_hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(low_level_keyboard_proc),
            ptr::null_mut(),
            0,
        );

        if h_hook.is_null() {
            let err = GetLastError();
            return Err(anyhow::anyhow!("Failed to set hook: {}", err));
        }

        println!(
            "Hook set and remapping active! Press keys to test. Exit with Ctrl+C (in terminal)."
        );

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        UnhookWindowsHookEx(h_hook);
    }
    Ok(())
}
