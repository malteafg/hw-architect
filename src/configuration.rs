use crate::resources;
use yaml_rust::{YamlEmitter, YamlLoader};

pub struct Config {
    pub sizex: i32,
    pub sizey: i32,
}

pub async fn load_config() -> anyhow::Result<Config> {
    let file = resources::load_string("baseconfig.yaml").await?;
    let docs = YamlLoader::load_from_str(&file).unwrap();
    let doc = &docs[0]; // Multi document support, doc is a yaml::Yaml

    println!("yaml content:\n{:?}", doc);

    let sizex = doc["window"]["dimensions"]["x"].as_i64().unwrap() as i32;
    let sizey = doc["window"]["dimensions"]["y"].as_i64().unwrap() as i32;

    // Debug support

    // // Index access for map & array
    // assert_eq!(doc["foo"][0].as_str().unwrap(), "list1");
    // assert_eq!(doc["bar"][1].as_f64().unwrap(), 2.0);

    // // Chained key/array access is checked and won't panic,
    // // return BadValue if they are not exist.
    // assert!(doc["INVALID_KEY"][100].is_badvalue());

    // Dump the YAML object
    let mut out_str = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(doc).unwrap(); // dump the YAML object to a String
    }
    println!("yaml output:\n{}", out_str);

    Ok(Config { sizex, sizey })
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
