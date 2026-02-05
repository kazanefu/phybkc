use async_trait::async_trait;
use dsl::{
    Block, Condition, ConditionEvaluator, Executor, InputSimulator, SendExpression, TriggerKey,
};
use profile::{Config, Profile};
use std::collections::{BTreeSet, HashMap};
use std::ptr;
use std::sync::{Arc, Mutex, OnceLock};
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

static CURRENT_PROFILE: OnceLock<Profile> = OnceLock::new();
static HELD_KEYS: OnceLock<Arc<Mutex<BTreeSet<u16>>>> = OnceLock::new();
static EXECUTOR: OnceLock<Arc<Executor>> = OnceLock::new();
static SCRIPT_TRIGGERS: OnceLock<HashMap<BTreeSet<u16>, Block>> = OnceLock::new();

#[derive(Debug)]
struct WindowsInputSimulator;

#[async_trait]
impl InputSimulator for WindowsInputSimulator {
    async fn send_keys(&self, expressions: &[SendExpression]) {
        for expr in expressions {
            match expr {
                SendExpression::Key(k) => {
                    if let Some(sc) = resolve_trigger_key(k) {
                        unsafe {
                            send_key_event(sc, true, false);
                        }
                        unsafe {
                            send_key_event(sc, false, false);
                        }
                    }
                }
                SendExpression::String(s) => {
                    for c in s.chars() {
                        unsafe {
                            send_unicode_char(c);
                        }
                    }
                }
                SendExpression::Combo(keys) => {
                    let mut scancodes = Vec::new();
                    for k in keys {
                        if let Some(sc) = resolve_trigger_key(k) {
                            scancodes.push(sc);
                            unsafe {
                                send_key_event(sc, true, false);
                            }
                        }
                    }
                    for sc in scancodes.into_iter().rev() {
                        unsafe {
                            send_key_event(sc, false, false);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct KeyConditionEvaluator {
    held_keys: Arc<Mutex<BTreeSet<u16>>>,
}

#[async_trait]
impl ConditionEvaluator for KeyConditionEvaluator {
    async fn evaluate(&self, condition: &Condition) -> bool {
        match condition {
            Condition::NowInput(combos) => {
                let held = self.held_keys.lock().unwrap();
                for combo in combos {
                    let mut all_match = true;
                    for tk in &combo.0 {
                        if let Some(sc) = resolve_trigger_key(tk) {
                            if !held.contains(&sc) {
                                all_match = false;
                                break;
                            }
                        } else {
                            all_match = false;
                            break;
                        }
                    }
                    if all_match {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }
}

fn resolve_trigger_key(tk: &TriggerKey) -> Option<u16> {
    match tk {
        TriggerKey::Physical(sc) => Some(*sc),
        TriggerKey::ExtendedPhysical(sc) => Some(*sc | 0xE000),
        TriggerKey::Virtual(name) => profile::get_scancode(name),
    }
}

unsafe fn send_unicode_char(c: char) {
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

unsafe fn send_key_event(scan_code: u16, is_key_down: bool, _is_sys_key: bool) {
    let mut input: INPUT = unsafe { std::mem::zeroed() };
    unsafe {
        input.r#type = INPUT_KEYBOARD;

        let mut flags = KEYEVENTF_SCANCODE;
        if !is_key_down {
            flags |= KEYEVENTF_KEYUP;
        }

        let actual_sc = if scan_code > 0xFF00 {
            flags |= KEYEVENTF_EXTENDEDKEY;
            scan_code & 0xFF
        } else {
            scan_code
        };

        input.Anonymous.ki.wScan = actual_sc;
        input.Anonymous.ki.dwFlags = flags;

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
                    if let Some(block) = triggers.get(h_ref)
                        && let Some(executor) = EXECUTOR.get()
                    {
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
            if let Some(target_key_name) = profile.keys.get(&sc_str)
                && let Some(target_sc) = profile::get_scancode(target_key_name)
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::load_from_file("sample/config.toml")?;
    let default_profile_path = config
        .profiles
        .get(&config.default_profile.default)
        .ok_or_else(|| anyhow::anyhow!("Default profile not found"))?;

    let profile = Profile::load_from_file(default_profile_path)?;
    println!("Loaded profile: {}", profile.name);

    let held_keys = Arc::new(Mutex::new(BTreeSet::new()));
    HELD_KEYS
        .set(held_keys.clone())
        .map_err(|_| anyhow::anyhow!("Failed to set held keys"))?;

    let mut all_global_settings = Vec::new();
    let mut all_macros = Vec::new();
    let mut triggers_map = HashMap::new();

    for script_path in &profile.scripts {
        println!("Loading script: {}", script_path);
        let content = std::fs::read_to_string(script_path)?;
        let mut input = content.as_str();
        let script = dsl::parse_script(&mut input)
            .map_err(|e| anyhow::anyhow!("Parse error in {}: {:?}", script_path, e))?;

        all_global_settings.extend(script.global_settings);
        all_macros.extend(script.macros);

        for block in script.blocks {
            for combo in &block.triggers {
                let mut resolved_combo = BTreeSet::new();
                for tk in &combo.0 {
                    if let Some(sc) = resolve_trigger_key(tk) {
                        resolved_combo.insert(sc);
                    }
                }
                triggers_map.insert(resolved_combo, block.clone());
            }
        }
    }

    let consolidated_script = dsl::Script {
        global_settings: all_global_settings,
        macros: all_macros,
        blocks: vec![],
    };

    let executor = Arc::new(Executor::new(
        consolidated_script,
        Arc::new(WindowsInputSimulator),
        Arc::new(KeyConditionEvaluator { held_keys }),
    ));
    EXECUTOR
        .set(executor)
        .map_err(|_| anyhow::anyhow!("Failed to set executor"))?;
    SCRIPT_TRIGGERS
        .set(triggers_map)
        .map_err(|_| anyhow::anyhow!("Failed to set triggers"))?;

    CURRENT_PROFILE
        .set(profile)
        .map_err(|_| anyhow::anyhow!("Failed to set profile"))?;

    unsafe {
        let h_hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(low_level_keyboard_proc),
            ptr::null_mut(),
            0,
        );

        if h_hook.is_null() {
            return Err(anyhow::anyhow!("Failed to set hook"));
        }

        println!("Hook active. Spawning DSL scripts as needed.");

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        UnhookWindowsHookEx(h_hook);
    }
    Ok(())
}
