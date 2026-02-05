mod evaluator;
mod hook;
mod keyboard;
mod simulator;
mod state;
mod tray;

use std::collections::{BTreeSet, HashMap};
use std::ptr;
use std::sync::{Arc, Mutex};
use windows_sys::Win32::UI::WindowsAndMessaging::*;

use crate::evaluator::KeyConditionEvaluator;
use crate::hook::low_level_keyboard_proc;
use crate::keyboard::resolve_trigger_key;
use crate::simulator::WindowsInputSimulator;
use crate::state::{CURRENT_PROFILE, EXECUTOR, HELD_KEYS, SCRIPT_TRIGGERS};
use dsl::{Executor, Script};
use profile::{Config, Profile};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load Config and Profile
    let config = Config::load_from_file("sample/config.toml")?;
    let default_profile_path = config
        .profiles
        .get(&config.default_profile.default)
        .ok_or_else(|| anyhow::anyhow!("Default profile not found"))?;

    let profile = Profile::load_from_file(default_profile_path)?;
    println!("Loaded profile: {}", profile.name);

    // 2. Initialize State
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
                let mut resolved_combo = Vec::new();
                for tk in &combo.0 {
                    if let Some(sc) = resolve_trigger_key(tk) {
                        resolved_combo.push(sc);
                    }
                }
                triggers_map.insert(resolved_combo, block.clone());
            }
        }
    }

    let consolidated_script = Script {
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

    // 3. Initialize Tray
    let _tray_icon = tray::init_tray()?;

    // 4. Set Keyboard Hook
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

        // 5. Message Loop
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);

            // Poll tray events
            tray::handle_tray_events();
        }

        UnhookWindowsHookEx(h_hook);
    }
    Ok(())
}
