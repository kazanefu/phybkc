use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;

const SCANCODE_NONE: u16 = 0xFFFF;

/// Start a polling thread that monitors keyboard state changes
/// and detects physical scan codes by directly reading the keyboard controller
pub fn start_hook_thread() -> Arc<AtomicU16> {
    let slot = Arc::new(AtomicU16::new(SCANCODE_NONE));
    let slot_clone = Arc::clone(&slot);

    std::thread::spawn(move || {
        let mut prev_states = [false; 256];

        loop {
            for vk in 1u32..256 {
                // Skip generic modifier VKs - use specific ones instead
                if vk == 0x10 || vk == 0x11 || vk == 0x12 {
                    continue;
                }

                let is_down = unsafe { (GetAsyncKeyState(vk as i32) as u16 & 0x8000) != 0 };
                let prev_down = prev_states[vk as usize];

                if is_down && !prev_down {
                    // Key was just pressed - get scan code
                    // MAPVK_VK_TO_VSC_EX (4) returns the hardware scan code with extended prefix
                    let sc = unsafe { MapVirtualKeyW(vk, MAPVK_VK_TO_VSC_EX) };
                    if sc != 0 {
                        // Extended scan codes have 0xE0 or 0xE1 prefix in high byte
                        let sc16 = sc as u16;
                        slot_clone.store(sc16, Ordering::Relaxed);
                    }
                }
                prev_states[vk as usize] = is_down;
            }

            // Poll at ~100Hz
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    slot
}
