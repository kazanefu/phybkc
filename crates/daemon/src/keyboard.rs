use dsl::TriggerKey;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

pub fn resolve_trigger_key(tk: &TriggerKey) -> Option<u16> {
    match tk {
        TriggerKey::Physical(sc) => Some(*sc),
        TriggerKey::ExtendedPhysical(sc) => Some(*sc | 0xE000),
        TriggerKey::Virtual(name) => profile::get_scancode(name),
    }
}

pub unsafe fn send_unicode_char(c: char) {
    let mut inputs: [INPUT; 2] = unsafe { std::mem::zeroed() };
    let mut utf16 = [0u16; 2];
    c.encode_utf16(&mut utf16);

    for (i, &code) in utf16.iter().enumerate() {
        if code == 0 && i > 0 {
            break;
        }
        unsafe {
            inputs[0].r#type = INPUT_KEYBOARD;
            inputs[0].Anonymous.ki.wScan = code;
            inputs[0].Anonymous.ki.dwFlags = KEYEVENTF_UNICODE;

            inputs[1] = inputs[0];
            inputs[1].Anonymous.ki.dwFlags |= KEYEVENTF_KEYUP;

            SendInput(2, inputs.as_ptr(), std::mem::size_of::<INPUT>() as i32);
        }
    }
}

pub unsafe fn send_key_event(scan_code: u16, is_key_down: bool, _is_sys_key: bool) {
    let mut input: INPUT = unsafe { std::mem::zeroed() };
    unsafe {
        input.r#type = INPUT_KEYBOARD;

        let mut dw_flags = 0;
        if !is_key_down {
            dw_flags |= KEYEVENTF_KEYUP;
        }

        // For Windows keys and some others, using VK is more reliable with SendInput
        let (vk, actual_sc, use_sc) = match scan_code {
            0xE05B => (VK_LWIN, 0x5B, false),
            0xE05C => (VK_RWIN, 0x5C, false),
            0xE05D => (VK_APPS, 0x5D, false),
            _ => {
                if scan_code > 0xFF00 {
                    (0, scan_code & 0xFF, true)
                } else {
                    (0, scan_code, true)
                }
            }
        };

        if use_sc {
            input.Anonymous.ki.wScan = actual_sc;
            input.Anonymous.ki.dwFlags = dw_flags | KEYEVENTF_SCANCODE;
            if scan_code > 0xFF00 {
                input.Anonymous.ki.dwFlags |= KEYEVENTF_EXTENDEDKEY;
            }
        } else {
            input.Anonymous.ki.wVk = vk;
            input.Anonymous.ki.dwFlags = dw_flags;
            // Most VKs don't need EXTENDEDKEY but LWin/RWin/Apps technically are
            if scan_code > 0xFF00 {
                input.Anonymous.ki.dwFlags |= KEYEVENTF_EXTENDEDKEY;
            }
        }

        SendInput(1, &input, std::mem::size_of::<INPUT>() as i32);
    }
}
