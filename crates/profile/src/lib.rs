use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub mod key_map;
pub use key_map::{get_name, get_scancode};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub profiles: HashMap<String, String>,
    pub default_profile: DefaultProfile,
    pub global_scripts: HashMap<String, String>,
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
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

impl Profile {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;
        let profile: Profile = serde_json::from_str(&content)?;
        Ok(profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let toml_content = r#"
[profiles]
profileA = "C:/User/phybkc/profiles/profileA.json"

[default_profile]
default = "profileA"

[global_scripts]
scriptA = "C:/User/phybkc/scripts/scriptA.phybkc"
"#;
        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(
            config.profiles.get("profileA").unwrap(),
            "C:/User/phybkc/profiles/profileA.json"
        );
        assert_eq!(config.default_profile.default, "profileA");
    }

    #[test]
    fn test_profile_parsing() {
        let json_content = r#"
{
    "name": "profileA",
    "keyboard": "JIS",
    "scripts": [
        "C:/User/phybkc/scripts/scriptA.phybkc"
    ],
    "keys": {
        "0x1E": "A",
        "0x30": "B"
    }
}
"#;
        let profile: Profile = serde_json::from_str(json_content).unwrap();
        assert_eq!(profile.name, "profileA");
        assert_eq!(profile.keyboard, "JIS");
        assert_eq!(profile.keys.get("0x1E").unwrap(), "A");
    }
}
