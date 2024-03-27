// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::{env, path::PathBuf};
use tauri::{Manager, PhysicalSize, Size};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

use futures_util::stream::StreamExt;

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

fn get_maps_directory() -> Result<PathBuf, String> {
    // try standard user profile path
    let userprofile = env::var("USERPROFILE").map_err(|e| e.to_string())?;
    let userprofile = PathBuf::from(userprofile);
    let base_directory = userprofile.join("Documents\\World in Conflict");
    if base_directory.exists() {
        let maps_directory = base_directory.join("Downloaded\\maps");
        if !maps_directory.exists() {
            std::fs::create_dir_all(&maps_directory).map_err(|e| e.to_string())?;
        }

        return Ok(maps_directory);
    }

    // try OneDrive path
    let onedrive = env::var("OneDrive").map_err(|e| e.to_string())?;
    let onedrive = PathBuf::from(onedrive);
    let base_directory = onedrive.join("Documents\\World in Conflict");
    if base_directory.exists() {
        let maps_directory = base_directory.join("Downloaded\\maps");
        if !maps_directory.exists() {
            std::fs::create_dir_all(&maps_directory).map_err(|e| e.to_string())?;
        }

        return Ok(maps_directory);
    }

    Err("Maps directory not found in standard or OneDrive locations.".to_string())
}

#[tauri::command]
async fn get_map_hash(filename: &str) -> Result<String, String> {
    let maps_directory = get_maps_directory()?;
    let path = maps_directory.join(filename);

    // Open the file asynchronously
    let mut file = File::open(path).await.map_err(|e| e.to_string())?;

    // Create a new, empty buffer
    let mut buffer = Vec::new();

    // Read the file's contents into the buffer asynchronously
    file.read_to_end(&mut buffer)
        .await
        .map_err(|e| e.to_string())?;

    // Compute the MD5 hash of the buffer's contents
    let hash = md5::compute(&buffer);

    // Return the hash as a hexadecimal string, uppercase
    Ok(format!("{:x}", hash).to_uppercase())
}

async fn download_file<F: FnMut(usize, usize) + Send + 'static>(
    url: &str,
    target: &str,
    mut progress_callback: F,
) -> Result<(), String> {
    println!("downloading file {} to {}", url, target);
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if response.status() != reqwest::StatusCode::OK {
        return Err(format!("Failed to download file: {}", response.status()));
    }

    let headers = response.headers();
    let content_length = headers.get("X-Filesize").unwrap();
    let total_size = content_length.to_str().unwrap().parse::<u64>().unwrap();

    let mut file = File::create(target)
        .await
        .map_err(|e| format!("Failed to create file: {}", e))?;
    let mut downloaded: usize = 0;

    let mut stream = response.bytes_stream();
    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| format!("Failed to read chunk: {}", e))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Failed to write to file: {}", e))?;
        downloaded += chunk.len();

        progress_callback(downloaded, total_size as usize);
    }

    Ok(())
}

#[tauri::command]
async fn get_map_files() -> Result<Vec<String>, String> {
    let maps_directory = get_maps_directory()?;

    let mut result: Vec<String> = Vec::new();

    let entries = std::fs::read_dir(maps_directory).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?; // Properly handle the Result
        let path = entry.path(); // Bind the path to a variable

        // skip directories
        if path.is_dir() {
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
async fn download_map(window: tauri::Window, map: &str) -> Result<(), String> {
    println!("downloading map {}", map);
    let maps_directory = get_maps_directory()?;

    let map_url = format!("{}/maps/download/{}", &CONFIG.API_URL, map);

    let map_cloned = map.to_string(); // Clone `map` here
    let progress_callback = move |current: usize, total: usize| {
        let percentage = (current as f64 / total as f64) * 100.0;
        window
            .emit(
                "download-progress",
                format!(
                    "{{\"map\": \"{}\", \"percentage\": {:.2}}}",
                    map_cloned, percentage
                ),
            )
            .expect("Failed to emit progress");
    };

    download_file(
        map_url.as_str(),
        &format!("{}\\{}", maps_directory.display(), map),
        progress_callback,
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
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
