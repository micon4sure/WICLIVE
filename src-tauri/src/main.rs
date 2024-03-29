// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod init;
mod io;

use init::VersionInfo;
use io::download_file;
use io::get_file_hash;
use io::get_maps_directory;

use config::Config;
use tauri::Manager;
use winapi::shared::rpcndr::boolean;

lazy_static::lazy_static! {
    static ref CONFIG: Config = Config::new();
}

#[tauri::command]
fn get_config() -> Result<Config, String> {
    return Ok(Config::new());
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
async fn get_map_hash(filename: &str) -> Result<String, String> {
    let maps_directory = get_maps_directory()?;
    let path = maps_directory.join(filename);

    return get_file_hash(path).await;
}

#[tauri::command]
async fn download_map_live(window: tauri::Window, map: &str) -> Result<(), String> {
    println!("downloading live map {}", map);
    let maps_directory = get_maps_directory()?;

    let map_url = format!("{}/maps/download/{}", &CONFIG.API_URL, map);

    let progress_callback =
        io::create_progress_callback(window.clone(), "download-map-live", Some(map.to_string()));

    download_file(
        map_url.as_str(),
        &format!("{}\\{}", maps_directory.display(), map),
        progress_callback,
    )
    .await
    .map_err(|e| e.to_string())?;
    println!("done downloading live map {}", map);

    Ok(())
}

#[tauri::command]
async fn download_map_custom(window: tauri::Window, map: &str) -> Result<(), String> {
    println!("downloading custom map {}", map);
    let maps_directory = get_maps_directory()?;

    let map_url = format!("https://www.massgate.org/files/maps/{}", map);

    let progress_callback =
        io::create_progress_callback(window.clone(), "download-map-custom", Some(map.to_string()));

    download_file(
        map_url.as_str(),
        &format!("{}\\{}", maps_directory.display(), map),
        progress_callback,
    )
    .await
    .map_err(|e| e.to_string())?;
    println!("done downloading custom map {}", map);

    Ok(())
}

#[tauri::command]
fn get_install_path() -> Option<String> {
    return init::find_install_path();
}
#[tauri::command]
async fn extract_game_version() -> Result<VersionInfo, String> {
    return init::extract_game_version().await;
}

#[tauri::command]
async fn download_game(window: tauri::Window) -> Result<String, String> {
    let game_url = "https://www.massgate.org/files/world_in_conflict_retail_1.000_en.zip";

    // create temp directory
    let temp_dir = std::env::temp_dir();
    let zip_path = temp_dir.join("world_in_conflict_retail_1.000_en.zip");

    let progress_callback = io::create_progress_callback(window.clone(), "download-game", None);

    download_file(game_url, zip_path.to_str().unwrap(), progress_callback).await?;
    Ok(zip_path.to_str().unwrap().to_string())
}

#[tauri::command]
async fn unzip_game(window: tauri::Window, zip_path: &str) -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let target_path = temp_dir.join("world_in_conflict_retail_1.000_en");
    let target_path_clone = target_path.clone();

    let progress_callback = io::create_progress_callback(window.clone(), "extract-game", None);

    let result = io::extract_zip(zip_path, target_path, progress_callback).await;
    match result {
        Ok(_) => Ok(target_path_clone.to_str().unwrap().to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn download_patch(window: tauri::Window, patch: u16) -> Result<String, String> {
    let base_url = "https://www.massgate.org/files/patches/";

    let filename;
    match patch {
        10 => {
            filename = "world_in_conflict_1.000_to_1.010_en.exe";
        }
        11 => {
            filename = "world_in_conflict_1.010_to_1.011_en.exe";
        }
        _ => return Err("invalid patch version".to_string()),
    }

    let patch_url = format!("{}/{}", base_url, filename);

    // create temp directory
    let temp_dir = std::env::temp_dir();
    let patch_path = temp_dir.join(filename);

    let progress_callback = io::create_progress_callback(window.clone(), "download-patch", None);

    download_file(
        patch_url.as_str(),
        patch_path.to_str().unwrap(),
        progress_callback,
    )
    .await?;
    Ok(patch_path.to_str().unwrap().to_string())
}

#[tauri::command]
async fn install_game(
    window: tauri::Window,
    _handle: tauri::AppHandle,
    target_dir: &str,
    installer_dir: &str,
) -> Result<(), String> {
    let resolver = |resource: &str| -> String {
        // ! path resolver is broken, temporary fix
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path.push("_up_");
        path.push("automation");
        path.push(resource);
        return path.to_str().unwrap().to_string();
    };

    return init::install_game(target_dir, installer_dir, resolver);
}

#[tauri::command]
async fn install_patch(_handle: tauri::AppHandle, installer_path: &str) -> Result<(), String> {
    let resolver = |resource: &str| -> String {
        // ! path resolver is broken, temporary fix
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path.push("_up_");
        path.push("automation");
        path.push(resource);
        return path.to_str().unwrap().to_string();
    };
    return init::install_patch(installer_path, resolver);
}

#[tauri::command]
async fn install_vcredist(_handle: tauri::AppHandle, vcredist_exe: &str) -> Result<(), String> {
    println!("installing vcredist");
    return init::install_vcredist(vcredist_exe);
}

#[tauri::command]
async fn download_vcredist(window: tauri::Window, version: u8) -> Result<String, String> {
    let vcredist_url;
    let target;
    match version {
        11 => {
            vcredist_url = "https://download.microsoft.com/download/1/6/B/16B06F60-3B20-4FF2-B699-5E9B7962F9AE/VSU_4/vcredist_x86.exe";
            target = "vcredist_x86_11.exe";
        }
        14 => {
            vcredist_url = "https://aka.ms/vs/17/release/vc_redist.x86.exe";
            target = "vcredist_x86_14.exe";
        }
        _ => {
            return Err("invalid vcredist version".to_string());
        }
    }

    // create temp directory
    let temp_dir = std::env::temp_dir();
    let vcredist_path = temp_dir.join(target);

    let progress_callback = io::create_progress_callback(window.clone(), "download-vcredist", None);

    download_file(
        vcredist_url,
        vcredist_path.to_str().unwrap(),
        progress_callback,
    )
    .await?;
    Ok(vcredist_path.to_str().unwrap().to_string())
}

#[tauri::command]
fn file_exists(path: &str) -> bool {
    return io::file_exists(path);
}
#[tauri::command]
async fn get_file_contents(path: String) -> Result<String, String> {
    return io::get_file_contents(path);
}
#[tauri::command]
async fn set_file_contents(path: &str, contents: &str) -> Result<(), String> {
    return io::set_file_contents(path, contents);
}
#[tauri::command]
async fn remove_file(path: &str) -> Result<(), String> {
    return io::remove_file(path);
}

#[tauri::command]
fn is_elevated() -> bool {
    return init::is_elevated();
}

#[tauri::command]
fn elevate_permissions(handle: tauri::AppHandle) {
    let elevated = init::is_elevated();
    if !elevated {
        init::elevate_permissions(handle);
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_map_files,
            get_map_hash,
            download_map_live,
            download_map_custom,
            get_config,
            get_install_path,
            file_exists,
            get_file_contents,
            set_file_contents,
            remove_file,
            is_elevated,
            elevate_permissions,
            extract_game_version,
            download_vcredist,
            download_game,
            unzip_game,
            download_patch,
            install_game,
            install_patch,
            install_vcredist
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            app.get_window("main").unwrap().open_devtools();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
