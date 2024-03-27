// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod io;

use io::download_file;
use io::get_file_hash;
use io::get_maps_directory;

use config::Config;

use std::{env, path::PathBuf};
use tauri::{Manager, PhysicalSize, Size};

lazy_static::lazy_static! {
    static ref CONFIG: Config = Config::new();
}

#[tauri::command]
async fn get_map_hash(filename: &str) -> Result<String, String> {
    let maps_directory = get_maps_directory()?;
    let path = maps_directory.join(filename);

    get_file_hash(path).await
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
