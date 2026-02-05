use dsl::{Block, Executor};
use profile::Profile;
use std::collections::{BTreeSet, HashMap};
use std::sync::OnceLock;
use std::sync::{Arc, Mutex, RwLock};

pub static CURRENT_PROFILE: OnceLock<Arc<RwLock<Option<Profile>>>> = OnceLock::new();
pub static HELD_KEYS: OnceLock<Arc<Mutex<BTreeSet<u16>>>> = OnceLock::new();
pub static EXECUTOR: OnceLock<Arc<RwLock<Option<Arc<Executor>>>>> = OnceLock::new();
pub static SCRIPT_TRIGGERS: OnceLock<Arc<RwLock<HashMap<Vec<u16>, Block>>>> = OnceLock::new();
