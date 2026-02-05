use crate::keyboard::send_key_event;
use crate::state::{CURRENT_PROFILE, EXECUTOR, HELD_KEYS, SCRIPT_TRIGGERS};
use std::collections::BTreeSet;
use std::ptr;
use std::sync::Arc;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

pub unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let kb_struct = unsafe { *(l_param as *const KBDLLHOOKSTRUCT) };

        if (kb_struct.flags & LLKHF_INJECTED) != 0 {
            return unsafe { CallNextHookEx(ptr::null_mut(), n_code, w_param, l_param) };
        }

        let scan_code = kb_struct.scanCode as u16;
        let is_extended = (kb_struct.flags & LLKHF_EXTENDED) != 0;
        let actual_sc = if is_extended {
            scan_code | 0xE000
        } else {
            scan_code
        };

        let event = w_param as u32;
        let is_key_down = event == WM_KEYDOWN || event == WM_SYSKEYDOWN;

        if let Some(held) = HELD_KEYS.get() {
            let mut h = held.lock().unwrap();
            if is_key_down {
                h.insert(actual_sc);

                if let Some(triggers) = SCRIPT_TRIGGERS.get() {
                    let h_ref: &BTreeSet<u16> = &h;
                    let mut best_match: Option<(&Vec<u16>, &dsl::Block)> = None;

                    // Status keys to ignore if not part of the trigger
                    let status_keys = [0x3A, 0x29, 0x7B, 0x79, 0x70];

                    for (combo, block) in triggers {
                        let is_all_held = combo.iter().all(|k| h_ref.contains(k));
                        let is_last_key = combo.last() == Some(&actual_sc);
                        let only_status_extras = h_ref
                            .iter()
                            .all(|k| combo.contains(k) || status_keys.contains(k));

                        if is_all_held && is_last_key && only_status_extras {
                            if let Some((best_combo, _)) = best_match {
                                if combo.len() > best_combo.len() {
                                    best_match = Some((combo, block));
                                }
                            } else {
                                best_match = Some((combo, block));
                            }
                        }
                    }

                    if let (Some((_, block)), Some(executor)) = (best_match, EXECUTOR.get()) {
                        let exec = Arc::clone(executor);
                        let b = block.clone();
                        tokio::spawn(async move {
                            exec.execute_block(&b).await;
                        });
                        return 1;
                    }
                }
            } else {
                h.remove(&actual_sc);
            }
        }

        if let Some(profile) = CURRENT_PROFILE.get() {
            let sc_str = format!("0x{:02X}", actual_sc);
            if let Some(target_sc) = profile
                .keys
                .get(&sc_str)
                .and_then(|name| profile::get_scancode(name))
            {
                unsafe {
                    send_key_event(target_sc, is_key_down, false);
                }
                return 1;
            }
        }
    }
    unsafe { CallNextHookEx(ptr::null_mut(), n_code, w_param, l_param) }
}
