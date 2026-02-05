use dsl::{Block, Executor};
use profile::Profile;
use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex, OnceLock};

pub static CURRENT_PROFILE: OnceLock<Profile> = OnceLock::new();
pub static HELD_KEYS: OnceLock<Arc<Mutex<BTreeSet<u16>>>> = OnceLock::new();
pub static EXECUTOR: OnceLock<Arc<Executor>> = OnceLock::new();
pub static SCRIPT_TRIGGERS: OnceLock<HashMap<Vec<u16>, Block>> = OnceLock::new();
