use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub profiles: HashMap<String, String>,
    pub default_profile: DefaultProfile,
    pub global_scripts: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultProfile {
    pub default: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub keyboard: String,
    pub scripts: Vec<String>,
    pub keys: HashMap<String, String>,
}
