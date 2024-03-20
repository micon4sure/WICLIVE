// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::env;
use tauri::{utils::config::TauriConfig, Manager, PhysicalSize};

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
        println!("Environment: {}", env);
        let api_url = match env {
            "development" => "http://localhost:3243".to_string(),
            "staging" => "https://techtile.media:3243".to_string(),
            "production" => "https://techtile.media:3243".to_string(),
            _ => "http://localhost:3243".to_string(),
        };
        println!("API URL: {}", api_url.as_str());

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

fn get_map_md5(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let hash = md5::compute(buffer);
    Ok(format!("{:x}", hash).to_uppercase())
}

async fn download_file(url: &str, target: &str) -> Result<(), String> {
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
async fn get_map_data() -> Result<serde_json::Map<String, serde_json::Value>, String> {
    let userprofile = env::var("USERPROFILE").map_err(|e| e.to_string())?;
    // write result data to cache file
    let cache_file_path =
        userprofile + "\\Documents\\World in Conflict\\Downloaded\\maps\\_cache.json";

    // check if cache file exists
    println!("checking for cache file {}", cache_file_path);
    if std::path::Path::new(&cache_file_path).exists() {
        println!("cache file exists, reading.");
        let cache_file = std::fs::read_to_string(cache_file_path).map_err(|e| e.to_string())?;

        let cache_data: serde_json::Map<String, serde_json::Value> =
            serde_json::from_str(&cache_file)
                .map_err(|e| format!("error parsing cache file{}", e.to_string()))?;

        println!("returning cached data");
        return Ok(cache_data);
    }

    return update_map_cache().await;
}

#[tauri::command]
async fn update_map_cache() -> Result<serde_json::Map<String, serde_json::Value>, String> {
    println!("updating map cache");
    let userprofile = env::var("USERPROFILE").map_err(|e| e.to_string())?;
    let maps_directory = userprofile.clone() + "\\Documents\\World in Conflict\\Downloaded\\maps";
    // write result data to cache file
    let cache_file_path = maps_directory.clone() + "\\_cache.json";

    // create a new JSON object to store map data
    let mut result_data = serde_json::Map::new();

    // for each file in maps_directory, get the md5 hash and add it to the result_data object
    println!("reading directory contents");
    let entries = std::fs::read_dir(maps_directory).map_err(|e| e.to_string())?;
    for entry in entries {
        println!("unwrapping filename");
        // skip directories
        if entry.as_ref().unwrap().path().is_dir() {
            println!(
                "skipping directory {}",
                entry.as_ref().unwrap().path().to_str().unwrap()
            );
            continue;
        }
        // skip files not ending in .sdf
        if entry.as_ref().unwrap().path().extension().unwrap() != "sdf" {
            println!(
                "skipping file {}",
                entry.as_ref().unwrap().path().to_str().unwrap()
            );
            continue;
        }

        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap();
        println!("getting md5 hash for {}", file_name);
        let map_hash = get_map_md5(&path.to_str().unwrap()).map_err(|e| e.to_string())?;

        println!("adding {}: {}", file_name, map_hash);

        result_data.insert(file_name.to_string(), serde_json::Value::String(map_hash));
    }

    // write result_data to cache file
    println!("writing cache file");
    let cache_file = File::create(&cache_file_path)
        .map_err(|e| format!("error creating cache file{}", e.to_string()))?;
    serde_json::to_writer(cache_file, &result_data)
        .map_err(|e| format!("error writing cache file{}", e.to_string()))?;

    println!("returning new data: {:?}", result_data);
    return Ok(result_data);
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
            get_map_data,
            download_map,
            update_map_cache,
            get_config
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            app.get_window("main").unwrap().open_devtools();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
