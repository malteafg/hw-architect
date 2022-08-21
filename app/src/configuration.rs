use crate::input_handler;
use anyhow::anyhow;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::str::FromStr;
use utils::input;
use utils::loader;
use yaml_rust::{Yaml, YamlLoader};

fn load_user_config_to_yaml(file: &str) -> anyhow::Result<Yaml> {
    ProjectDirs::from("com", "simaflux", "hw-architect")
        .and_then(|proj_dirs| {
            let config_dir = proj_dirs.config_dir();
            let config_file = fs::read_to_string(config_dir.join(file)).ok()?;
            let docs = YamlLoader::load_from_str(&config_file).ok()?;
            if docs.len() < 1 {
                None
            } else {
                Some(docs[0].clone())
            }
        })
        .ok_or(anyhow!("failed to update with user config"))
}

#[cfg(debug_assertions)]
fn load_dev_config_to_yaml(file: &str) -> anyhow::Result<Yaml> {
    let config_file = fs::read_to_string(file)?;
    let docs = YamlLoader::load_from_str(&config_file)?;
    if docs.len() < 1 {
        Err(anyhow!("no contents in dev config"))
    } else {
        Ok(docs[0].clone())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct WindowConfig {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Config {
    pub window: WindowConfig,
    pub key_map: String,
}

impl Config {
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

pub async fn load_config() -> anyhow::Result<Config> {
    let config_file = loader::load_string("config/base_config.yml").await?;
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

// let mut map = BTreeMap::new();
// map.insert("x".to_string(), 1.0);
type KeyConfig = BTreeMap<String, Vec<String>>;

pub async fn load_key_map(key_map: String) -> anyhow::Result<input_handler::KeyMap> {
    let key_config_path = format!("config/{}.yml", &key_map);
    dbg!(key_config_path.clone());
    let key_config_file = loader::load_string(&key_config_path).await?;
    let key_config: KeyConfig = serde_yaml::from_str(&key_config_file)?;

    let key_map = key_config
        .iter()
        .map(|(action, keys)| {
            let key_code = input_handler::parse_key_code(&keys[0]).unwrap();
            let mod_state = keys
                .iter()
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
            let action = input::Action::from_str(action).unwrap();
            ((key_code, mod_state), action)
        })
        .collect();

    // let key_map = match load_user_config_to_yaml("keymap.yml") {
    //     Ok(yaml) => key_map.update_from_yaml(yaml),
    //     _ => key_map,
    // };
    // #[cfg(debug_assertions)]
    // let key_map = match load_dev_config_to_yaml("keymap.yml") {
    //     Ok(yaml) => key_map.update_from_yaml(yaml),
    //     _ => key_map,
    // };

    Ok(key_map)
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
        file.write_all(&baseconfigyaml.as_bytes()).unwrap();
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
        file.write_all(&keyconfigyaml.as_bytes()).unwrap();
    }
}
//     let s = "
// foo:
//     - list1
//     - list2
// bar:
//     - 1
//     - 2.0
// window:
//   dimensions:
//     columns: 0
//     lines: 0

//   decorations: full
//   startup_mode: Windowed
//   opacity: 0.9
// ";
