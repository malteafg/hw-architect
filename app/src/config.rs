//! Handles the configuration files for highway architect.

use std::collections::BTreeMap;
use std::str::FromStr;

use crate::input_handler;
use utils::input;
use utils::loader;

use anyhow::anyhow;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use winit::keyboard::Key;

use figment::{
    providers::{Format, Yaml},
    Figment,
};
use winit::keyboard::NamedKey;

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

// type KeyConfig = BTreeMap<String, Vec<String>>;
/// This type corresponds to the structure of the yaml files that define keybindings:
/// TODO: Right now the inner BTreeMap always only has one element, so maybe come up with a better
/// structure.
type KeyLoaderConfig = BTreeMap<String, Vec<BTreeMap<String, Vec<String>>>>;

/// Loads and returns the given keymap
///
/// # Arguments
///
/// * `key_map` - Default and ONLY (for now) options are "qwerty" "wokmok"
pub fn load_key_map(key_map: String) -> anyhow::Result<input_handler::KeyMap> {
    let key_config_path = format!("config/{}.yml", &key_map);
    #[cfg(debug_assertions)]
    {
        dbg!(key_config_path.clone());
    }
    let key_config_file = loader::load_string(&key_config_path)?;
    let key_config: KeyLoaderConfig = serde_yaml::from_str(&key_config_file)?;

    let mut group_maps: BTreeMap<String, input_handler::KeyMap> = BTreeMap::new();
    for (group, key_maps) in key_config.into_iter() {
        let mut group_map: input_handler::KeyMap = BTreeMap::new();
        for key_map in key_maps.into_iter() {
            // this loop is silly as there is only one entry in the map
            for (action, keys) in key_map.into_iter() {
                let key_code = parse_key_code(&keys[0]).unwrap();
                let mod_state =
                    keys.iter()
                        .fold(
                            input::ModifierState::default(),
                            |mod_state, key| match key.as_str() {
                                "shift" => input::ModifierState {
                                    shift: true,
                                    ..mod_state
                                },
                                "ctrl" => input::ModifierState {
                                    ctrl: true,
                                    ..mod_state
                                },
                                "alt" => input::ModifierState {
                                    alt: true,
                                    ..mod_state
                                },
                                _ => mod_state,
                            },
                        );
                let action = input::Action::from_str(&action).unwrap();
                group_map.insert((key_code, mod_state), vec![action]);
            }
        }
        group_maps.insert(group, group_map);
    }

    // Merge group maps and check for conflicting keybindings
    let mut general_key_bindings: input_handler::KeyMap = group_maps
        .remove("general")
        .ok_or(anyhow!("Could not find general key bindings"))?;

    let mut final_key_map: input_handler::KeyMap = BTreeMap::new();
    for (_, group_map) in group_maps.into_iter() {
        for (key, mut action) in group_map.into_iter() {
            if general_key_bindings.contains_key(&key) {
                dbg!(key);
                return Err(anyhow!("Duplicate key binding"));
            }
            let Some(actions) = final_key_map.get_mut(&key) else {
                final_key_map.insert(key, action);
                continue;
            };
            actions.append(&mut action);
        }
    }

    // Add general_key_bindings to final_key_map
    final_key_map.append(&mut general_key_bindings);
    Ok(final_key_map)
}

/// Translates the keycode as it is written in the keymap config to a winit
/// [`VirtualKeyCode`]
fn parse_key_code(key: &String) -> anyhow::Result<Key> {
    match key.to_lowercase().as_str() {
        "esc" => Ok(Key::Named(NamedKey::Escape)),
        "space" => Ok(Key::Named(NamedKey::Space)),
        "a" => Ok(Key::Character("a".into())),
        "b" => Ok(Key::Character("b".into())),
        "c" => Ok(Key::Character("c".into())),
        "d" => Ok(Key::Character("d".into())),
        "e" => Ok(Key::Character("e".into())),
        "f" => Ok(Key::Character("f".into())),
        "g" => Ok(Key::Character("g".into())),
        "h" => Ok(Key::Character("h".into())),
        "j" => Ok(Key::Character("j".into())),
        "k" => Ok(Key::Character("k".into())),
        "l" => Ok(Key::Character("l".into())),
        "m" => Ok(Key::Character("m".into())),
        "n" => Ok(Key::Character("n".into())),
        "o" => Ok(Key::Character("o".into())),
        "p" => Ok(Key::Character("p".into())),
        "q" => Ok(Key::Character("q".into())),
        "r" => Ok(Key::Character("r".into())),
        "s" => Ok(Key::Character("s".into())),
        "t" => Ok(Key::Character("t".into())),
        "u" => Ok(Key::Character("u".into())),
        "v" => Ok(Key::Character("v".into())),
        "w" => Ok(Key::Character("w".into())),
        "x" => Ok(Key::Character("x".into())),
        "y" => Ok(Key::Character("y".into())),
        "z" => Ok(Key::Character("z".into())),
        "1" => Ok(Key::Character("1".into())),
        "2" => Ok(Key::Character("2".into())),
        "3" => Ok(Key::Character("3".into())),
        "4" => Ok(Key::Character("4".into())),
        "5" => Ok(Key::Character("5".into())),
        "6" => Ok(Key::Character("6".into())),
        "7" => Ok(Key::Character("7".into())),
        "8" => Ok(Key::Character("8".into())),
        "9" => Ok(Key::Character("9".into())),
        _ => Err(anyhow::anyhow!(format!("could not parse key: {}", key))),
    }
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

    // Run with cargo test write_keyconfig -- --ignored --nocapture in this crate
    #[test]
    #[ignore]
    fn write_keyconfig() {
        let mut key_map = std::collections::BTreeMap::new();
        key_map.insert(
            "camera_left".to_string(),
            vec!["a".to_string(), "shift".to_string()],
        );
        key_map.insert(
            "camera_right".to_string(),
            vec!["d".to_string(), "shift".to_string(), "ctrl".to_string()],
        );
        let keyconfigyaml = serde_yaml::to_string(&key_map).unwrap();
        println!("{}", keyconfigyaml);

        let mut file = File::create("../res/config/qwerty.yml").unwrap();
        file.write_all(keyconfigyaml.as_bytes()).unwrap();
    }
}
