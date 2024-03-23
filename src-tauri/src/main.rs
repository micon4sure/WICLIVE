// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};
use tauri::{utils::config::TauriConfig, Manager, PhysicalSize, Size};

use std::hash::Hasher;

use std::{
    fs::File,
    io::{self, copy, Read},
};

#[derive(Serialize)]
struct Config {
    API_URL: String,
    VERSION: String,
}

impl Config {
    fn new() -> Self {
        // set api url
        let env = env!("WICLIVE_ENV");
        let api_url = match env {
            "development" => "http://localhost:3243".to_string(),
            "staging" => "https://techtile.media:3243".to_string(),
            "production" => "https://techtile.media:3243".to_string(),
            _ => "http://localhost:3243".to_string(),
        };

        // get version from cargo.toml
        let version = env!("CARGO_PKG_VERSION").to_string();
        Config {
            API_URL: api_url,
            VERSION: version,
        }
    }
}

lazy_static::lazy_static! {
    static ref CONFIG: Config = Config::new();
}

#[tauri::command]
fn get_map_hash(filename: &str) -> Result<String, String> {
    let userprofile = env::var("USERPROFILE").map_err(|e| e.to_string())?;
    let maps_directory =
        PathBuf::from(userprofile).join("Documents\\World in Conflict\\Downloaded\\maps");
    let path = maps_directory.join(filename);

    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
    let hash = md5::compute(buffer);
    Ok(format!("{:x}", hash).to_uppercase())
}

async fn download_file(url: &str, target: &str) -> Result<(), String> {
    println!("downloading file {} to {}", url, target);
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status() != 200 {
        return Err(format!("Failed to download file: {}", response.status()));
    }

    let mut file = File::create(target).map_err(|e| format!("Failed to create file: {}", e))?;

    let content = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    copy(&mut content.as_ref(), &mut file)
        .map_err(|e| format!("Failed to write to file: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn get_map_files() -> Result<Vec<String>, String> {
    use std::env;
    use std::path::PathBuf;

    let userprofile = env::var("USERPROFILE").map_err(|e| e.to_string())?;
    let maps_directory =
        PathBuf::from(userprofile).join("Documents\\World in Conflict\\Downloaded\\maps");

    let mut result: Vec<String> = Vec::new();

    let entries = std::fs::read_dir(maps_directory).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?; // Properly handle the Result
        let path = entry.path(); // Bind the path to a variable

        // skip directories
        if path.is_dir() {
            println!("skipping directory {}", path.to_str().unwrap());
            continue;
        }

        // skip non-sdf files
        if path.extension().unwrap_or_default() != "sdf" {
            println!("skipping file {}", path.to_str().unwrap());
            continue;
        }

        if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
            result.push(filename.to_string());
        } else {
            println!("failed to convert filename to string");
            continue;
        }
    }
    Ok(result)
}

#[tauri::command]
async fn download_map(map: &str) -> Result<(), String> {
    println!("downloading map {}", map);
    let userprofile = env::var("USERPROFILE").map_err(|e| e.to_string())?;
    let maps_directory = userprofile.clone() + "\\Documents\\World in Conflict\\Downloaded\\maps";

    let map_url = format!("{}/maps/download/{}", &CONFIG.API_URL, map);

    download_file(
        map_url.as_str(),
        format!("{}\\{}", maps_directory, map).as_str(),
    )
    .await
    .map_err(|e| e.to_string())?;
    println!("done downloading map {}", map);

    Ok(())
}

#[tauri::command]
fn get_config() -> Result<Config, String> {
    return Ok(Config::new());
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_map_files,
            get_map_hash,
            download_map,
            get_config
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            app.get_window("main").unwrap().open_devtools();
            app.get_window("main")
                .unwrap()
                .set_size(Size::Physical(PhysicalSize {
                    width: 1024,
                    height: 768,
                }))
                .unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
