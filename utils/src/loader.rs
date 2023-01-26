//! This module handles I/O

use cfg_if::cfg_if;

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let base = reqwest::Url::parse(&format!(
        "{}/{}/",
        location.origin().unwrap(),
        option_env!("RES_PATH").unwrap_or("res"),
    ))
    .unwrap();
    base.join(file_name).unwrap()
}

/// Loads a file relative to the res directory as a [`String`] object. This is
/// useful for loading plaintext such as yaml files.
pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            // dbg!(url.clone());
            let txt = reqwest::get(url)
                .await?
                .text()
                .await?;
        } else {
            // let path = std::path::Path::new(env!("OUT_DIR"))
            //     .join("res")
            //     .join(file_name);
            // println!("{}", path.clone().into_os_string().into_string().unwrap());
            let path = std::path::Path::new("res").join(file_name);
            let txt = std::fs::read_to_string(path)?;
        }
    }

    Ok(txt)
}

/// Loads a file relative to the res directoly as a [`Vec`] of bytes ([`u8`]).
/// This is useful for loading textures, models and such.
pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let data = reqwest::get(url)
                .await?
                .bytes()
                .await?
                .to_vec();
        } else {
            // let path = std::path::Path::new(env!("OUT_DIR"))
            //     .join("res")
            //     .join(file_name);
            let path = std::path::Path::new("res").join(file_name);
            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}

// pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
//     cfg_if! {
//         if #[cfg(target_arch = "wasm32")] {
//             let url = format_url(file_name);
//             // dbg!(url.clone());
//             let txt = reqwest::get(url)
//                 .await?
//                 .text()
//                 .await?;
//         } else {
//             // let path = std::path::Path::new(env!("OUT_DIR"))
//             //     .join("res")
//             //     .join(file_name);
//             // println!("{}", path.clone().into_os_string().into_string().unwrap());
//             let path = std::path::Path::new("res").join(file_name);
//             let txt = std::fs::read_to_string(path)?;
//         }
//     }

//     Ok(txt)
// }

// pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
//     cfg_if! {
//         if #[cfg(target_arch = "wasm32")] {
//             let url = format_url(file_name);
//             let data = reqwest::get(url)
//                 .await?
//                 .bytes()
//                 .await?
//                 .to_vec();
//         } else {
//             // let path = std::path::Path::new(env!("OUT_DIR"))
//             //     .join("res")
//             //     .join(file_name);
//             let path = std::path::Path::new("res").join(file_name);
//             let data = std::fs::read(path)?;
//         }
//     }

//     Ok(data)
// }
