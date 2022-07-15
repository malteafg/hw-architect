use crate::resources;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_yaml::*;
use std::fmt::Display;
use std::fs;
use yaml_rust::{YamlEmitter, YamlLoader};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Window {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub window: Window,
}

// macro_rules! update_conf {
//     ( $( $e:ident ),* ) => {
//         $(
//             e = e;
//         )*
//     };
// }

pub async fn load_config() -> anyhow::Result<Config> {
    let file = resources::load_string("baseconfig.yml").await?;
    let base_conf: Config = serde_yaml::from_str(&file)?;

    // let docs = YamlLoader::load_from_str(&file).unwrap();
    // let doc = &docs[0]; // Multi document support, doc is a yaml::Yaml

    // println!("yaml content:\n{:?}", doc);

    // let mut sizex = doc["window"]["dimensions"]["x"].as_i64().unwrap() as i32;
    // let mut sizey = doc["window"]["dimensions"]["y"].as_i64().unwrap() as i32;

    match resources::load_string("config.yml").await {
        Ok(user_conf) => {
            // update_conf!(width);
            Ok(base_conf)
        }
        Err(_) => Ok(base_conf),
    }
    // let docs = YamlLoader::load_from_str(&file).unwrap();
    // let doc = &docs[0]; // Multi document support, doc is a yaml::Yaml

    // println!("yaml content:\n{:?}", doc);

    // sizex = if let Some(res) = doc["window"]["dimensions"]["x"].as_i64() {
    //     res as i32
    // } else {
    //     sizex
    // };
    // sizey = if let Some(res) = doc["window"]["dimensions"]["y"].as_i64() {
    //     res as i32
    // } else {
    //     sizey
    // };

    // Debug support

    // // Index access for map & array
    // assert_eq!(doc["foo"][0].as_str().unwrap(), "list1");
    // assert_eq!(doc["bar"][1].as_f64().unwrap(), 2.0);

    // // Chained key/array access is checked and won't panic,
    // // return BadValue if they are not exist.
    // assert!(doc["INVALID_KEY"][100].is_badvalue());

    // // Dump the YAML object
    // let mut out_str = String::new();
    // {
    //     let mut emitter = YamlEmitter::new(&mut out_str);
    //     emitter.dump(doc).unwrap(); // dump the YAML object to a String
    // }
    // println!("yaml output:\n{}", out_str);

    // Ok(Config { sizex, sizey })
    // Err(anyhow!("aosit"))
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
            window: Window {
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
