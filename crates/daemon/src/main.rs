mod evaluator;
mod hook;
mod keyboard;
mod simulator;
mod state;
mod tray;

use std::collections::{BTreeSet, HashMap};
use std::ptr;
use std::sync::{Arc, Mutex, RwLock};
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
    // 0. Initialize State Containers
    HELD_KEYS
        .set(Arc::new(Mutex::new(BTreeSet::new())))
        .unwrap();
    CURRENT_PROFILE.set(Arc::new(RwLock::new(None))).unwrap();
    EXECUTOR.set(Arc::new(RwLock::new(None))).unwrap();
    SCRIPT_TRIGGERS
        .set(Arc::new(RwLock::new(HashMap::new())))
        .unwrap();

    // 1. Load Config
    let config = Config::load_from_file("sample/config.toml")?;

    // 2. Initial Profile Load
    load_profile(&config, &config.default_profile.default).await?;

    // 3. Initialize Tray
    let tray_icon = tray::init_tray(&config)?;

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
            match tray::handle_tray_events() {
                tray::TrayAction::Quit => break,
                tray::TrayAction::Reload => {
                    let current_name = {
                        let p_lock = CURRENT_PROFILE.get().unwrap().read().unwrap();
                        p_lock.as_ref().map(|p| p.name.clone())
                    };
                    if let Some(name) = current_name {
                        let _ = load_profile(&config, &name).await;
                    }
                }
                tray::TrayAction::SwitchProfile(name) => {
                    if load_profile(&config, &name).await.is_ok() {
                        // Update tooltip
                        let _ = tray_icon.set_tooltip(Some(format!("phybkc - {}", name)));
                    }
                }
                tray::TrayAction::None => {}
            }
        }

        UnhookWindowsHookEx(h_hook);
    }
    Ok(())
}

async fn load_profile(config: &Config, profile_name: &str) -> anyhow::Result<()> {
    let profile_path = config
        .profiles
        .get(profile_name)
        .ok_or_else(|| anyhow::anyhow!("Profile {} not found", profile_name))?;

    let profile = Profile::load_from_file(profile_path)?;
    println!("Loading profile: {}", profile.name);

    let mut all_global_settings = Vec::new();
    let mut all_macros = Vec::new();
    let mut triggers_map = HashMap::new();

    let held_keys = HELD_KEYS.get().unwrap().clone();

    for script_path in &profile.scripts {
        println!("  Loading script: {}", script_path);
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

    // Update global state
    *CURRENT_PROFILE.get().unwrap().write().unwrap() = Some(profile);
    *EXECUTOR.get().unwrap().write().unwrap() = Some(executor);
    *SCRIPT_TRIGGERS.get().unwrap().write().unwrap() = triggers_map;

    println!("Profile {} loaded successfully.", profile_name);
    Ok(())
}
