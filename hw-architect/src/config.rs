//! Handles the configuration files for highway architect.

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use figment::{
    providers::{Format, Yaml},
    Figment,
};

/// Returns the user config stored at:
///
/// Linux: /home/Alice/.config/hw-architect/config.yml
/// Windows: C:\Users\Alice\AppData\Roaming\simaflux\hw-architect\config.yml
/// Mac: /Users/Alice/Library/Application Support/com.simaflux.hw-architect/config.yml
pub fn get_config_dir() -> std::path::PathBuf {
    let path = ProjectDirs::from("com", "simaflux", "hw_architect")
        .map(|dir| dir.config_dir().to_path_buf())
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            "no valid home directory found using the projectdirs crate, can't use/create config dir",
        )).unwrap();
    path
}

/// Configuration of the window.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct WindowConfig {
    /// Width of the window given in pixels.
    pub width: i32,
    /// Height of the window given in pixels.
    pub height: i32,
}

/// Configuration of highway architect.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Config {
    /// The window configuration
    pub window: WindowConfig,
    /// The key map to use. Default is "qwerty", and default options are "qwerty
    /// and wokmok"
    pub key_map: String,
}

impl Default for Config {
    fn default() -> Self {
        let win_config = WindowConfig {
            width: 1920,
            height: 1080,
        };

        Self {
            window: win_config,
            key_map: "qwerty".to_string(),
        }
    }
}

/// Loads the configuration for highway architect
pub fn load_config() -> anyhow::Result<Config> {
    let mut user_conf = get_config_dir();
    user_conf.push("config.yml");

    let figment = Figment::from(Yaml::file("res/config/base_config.yml"));
    let figment = figment.merge(Yaml::file(user_conf));
    #[cfg(debug_assertions)]
    let figment = figment.merge(Yaml::file("config.yml"));

    let config = figment.extract().unwrap();
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::*;
    use std::io::prelude::*;

    // Run with cargo test write_baseconfig -- --ignored --nocapture in this crate
    #[test]
    #[ignore]
    fn write_baseconfig() {
        let baseconfig = Config::default();
        let baseconfigyaml = serde_yaml::to_string(&baseconfig).unwrap().to_lowercase();
        println!("{}", baseconfigyaml);

        let mut file = File::create("../res/config/base_config.yml").unwrap();
        file.write_all(baseconfigyaml.as_bytes()).unwrap();
    }
}
