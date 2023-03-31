//! Handles the configuration files for highway architect.

use std::collections::BTreeMap;
use std::fs;
use std::str::FromStr;

use crate::input_handler;
use utils::input;
use utils::loader;

use anyhow::anyhow;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use winit::event::VirtualKeyCode;
use yaml_rust::{Yaml, YamlLoader};

/// Returns the user config stored at:
///
/// Linux: /home/Alice/.config/hw-architect/config.yml
///
/// Windows: C:\Users\Alice\AppData\Roaming\simaflux\hw-architect\config.yml
///
/// Mac: /Users/Alice/Library/Application Support/com.simaflux.hw-architect/config.yml
fn load_user_config_to_yaml(file: &str) -> anyhow::Result<Yaml> {
    dbg!("loading user config");
    ProjectDirs::from("com", "simaflux", "hw-architect")
        .and_then(|proj_dirs| {
            let config_dir = proj_dirs.config_dir();
            let config_file = fs::read_to_string(config_dir.join(file)).ok()?;
            let docs = YamlLoader::load_from_str(&config_file).ok()?;
            if docs.is_empty() {
                None
            } else {
                Some(docs[0].clone())
            }
        })
        .ok_or_else(|| anyhow!("failed to update with user config"))
}

/// Returns the user config (dev config) stored at project_root/config.yml.
#[cfg(debug_assertions)]
fn load_dev_config_to_yaml(file: &str) -> anyhow::Result<Yaml> {
    dbg!("loading dev config");
    let config_file = fs::read_to_string(file)?;
    let docs = YamlLoader::load_from_str(&config_file)?;
    if docs.is_empty() {
        Err(anyhow!("no contents in dev config"))
    } else {
        Ok(docs[0].clone())
    }
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

impl Config {
    /// Updates self with values of the variables that are set in another yaml.
    /// If some variables are not set in the other yaml, the values of self are
    /// used again.
    fn update_from_yaml(self, yaml: Yaml) -> Self {
        let width = yaml["window"]["width"]
            .as_i64()
            .unwrap_or(self.window.width as i64) as i32;
        let height = yaml["window"]["height"]
            .as_i64()
            .unwrap_or(self.window.height as i64) as i32;
        let key_map = match yaml["key_map"].as_str() {
            Some("qwerty") | Some("wokmok") => "wokmok".to_string(),
            _ => self.key_map,
        };

        let window = WindowConfig { width, height };

        Config { window, key_map }
    }
}

/// Loads the configuration file for highway architect
pub fn load_config() -> anyhow::Result<Config> {
    let config_file = loader::load_string("config/base_config.yml")?;
    let base_config: Config = serde_yaml::from_str(&config_file)?;

    let config = match load_user_config_to_yaml("config.yml") {
        Ok(yaml) => base_config.update_from_yaml(yaml),
        _ => base_config,
    };
    #[cfg(debug_assertions)]
    let config = match load_dev_config_to_yaml("config.yml") {
        Ok(yaml) => config.update_from_yaml(yaml),
        _ => config,
    };
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
fn parse_key_code(key: &String) -> anyhow::Result<VirtualKeyCode> {
    match key.to_lowercase().as_str() {
        "a" => Ok(VirtualKeyCode::A),
        "b" => Ok(VirtualKeyCode::B),
        "c" => Ok(VirtualKeyCode::C),
        "d" => Ok(VirtualKeyCode::D),
        "e" => Ok(VirtualKeyCode::E),
        "f" => Ok(VirtualKeyCode::F),
        "g" => Ok(VirtualKeyCode::G),
        "h" => Ok(VirtualKeyCode::H),
        "j" => Ok(VirtualKeyCode::J),
        "k" => Ok(VirtualKeyCode::K),
        "l" => Ok(VirtualKeyCode::L),
        "m" => Ok(VirtualKeyCode::M),
        "n" => Ok(VirtualKeyCode::N),
        "o" => Ok(VirtualKeyCode::O),
        "p" => Ok(VirtualKeyCode::P),
        "q" => Ok(VirtualKeyCode::Q),
        "r" => Ok(VirtualKeyCode::R),
        "s" => Ok(VirtualKeyCode::S),
        "t" => Ok(VirtualKeyCode::T),
        "u" => Ok(VirtualKeyCode::U),
        "v" => Ok(VirtualKeyCode::V),
        "w" => Ok(VirtualKeyCode::W),
        "x" => Ok(VirtualKeyCode::X),
        "y" => Ok(VirtualKeyCode::Y),
        "z" => Ok(VirtualKeyCode::Z),
        "1" => Ok(VirtualKeyCode::Key1),
        "2" => Ok(VirtualKeyCode::Key2),
        "3" => Ok(VirtualKeyCode::Key3),
        "4" => Ok(VirtualKeyCode::Key4),
        "5" => Ok(VirtualKeyCode::Key5),
        "6" => Ok(VirtualKeyCode::Key6),
        "7" => Ok(VirtualKeyCode::Key7),
        "8" => Ok(VirtualKeyCode::Key8),
        "9" => Ok(VirtualKeyCode::Key9),
        "esc" => Ok(VirtualKeyCode::Escape),
        "space" => Ok(VirtualKeyCode::Space),
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
        let baseconfig = Config {
            window: WindowConfig {
                width: 1920,
                height: 1080,
            },
            key_map: "qwerty".to_string(),
        };

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
