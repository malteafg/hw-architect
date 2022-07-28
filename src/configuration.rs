use crate::resources;
use anyhow::anyhow;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use yaml_rust::{Yaml, YamlLoader};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub window: WindowConfig,
}

impl Config {
    fn update_from_yaml(self: Self, yaml: Vec<Yaml>) -> Config {
        let doc = &yaml[0]; // Multi document support, doc is a yaml::Yaml

        let width = doc["window"]["width"]
            .as_i64()
            .unwrap_or(self.window.width as i64) as i32;
        let height = doc["window"]["height"]
            .as_i64()
            .unwrap_or(self.window.height as i64) as i32;

        let window = WindowConfig { width, height };

        log::warn!("Some info");
        Config { window }
    }
}

fn load_user_config_to_yaml() -> anyhow::Result<Vec<Yaml>> {
    ProjectDirs::from("com", "simaflux", "hw-architect")
        .and_then(|proj_dirs| {
            let config_dir = proj_dirs.config_dir();
            let config_file = fs::read_to_string(config_dir.join("config.yml")).ok()?;
            dbg!(config_dir);
            let docs = YamlLoader::load_from_str(&config_file).ok()?;
            Some(docs)
        })
        .ok_or(anyhow!("failed to update with user config"))
}

#[cfg(debug_assertions)]
fn load_dev_config_to_yaml() -> anyhow::Result<Vec<Yaml>> {
    let config_file = fs::read_to_string("config.yml")?;
    let docs = YamlLoader::load_from_str(&config_file)?;
    Ok(docs)
}

pub async fn load_config() -> anyhow::Result<Config> {
    let file = resources::load_string("baseconfig.yml").await?;
    let base_config: Config = serde_yaml::from_str(&file)?;

    log::warn!("Some info");
    let config = match load_user_config_to_yaml() {
        Ok(yaml) => base_config.update_from_yaml(yaml),
        _ => base_config,
    };
    #[cfg(debug_assertions)]
    let config = match load_dev_config_to_yaml() {
        Ok(yaml) => config.update_from_yaml(yaml),
        _ => config,
    };
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
        let baseconfig = Config {
            window: WindowConfig {
                width: 1920,
                height: 1080,
            },
        };

        let baseconfigyaml = serde_yaml::to_string(&baseconfig).unwrap();
        println!("{}", baseconfigyaml);

        let mut file = File::create("res/baseconfig.yml").unwrap();
        file.write_all(&baseconfigyaml.as_bytes()).unwrap();
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
