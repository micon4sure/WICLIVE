// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod install;
mod io;

use install::VersionInfo;
use io::download_file;
use io::get_file_hash;
use io::get_maps_directory;

use config::Config;
use tauri::Manager;

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

    let map_url = format!("{}/files/maps/{}", &CONFIG.MASSGATE_URL, map);

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
    return install::find_install_path();
}
#[tauri::command]
async fn extract_game_version() -> Result<VersionInfo, String> {
    return install::extract_game_version().await;
}

#[tauri::command]
async fn download_game(window: tauri::Window) -> Result<String, String> {
    let game_url = format!(
        "{}/files/world_in_conflict_retail_1.000_en.zip",
        &CONFIG.MASSGATE_URL
    );

    // create temp directory
    let temp_dir = std::env::temp_dir();
    let zip_path = temp_dir.join("world_in_conflict_retail_1.000_en.zip");

    let progress_callback = io::create_progress_callback(window.clone(), "download-game", None);

    download_file(
        game_url.as_str(),
        zip_path.to_str().unwrap(),
        progress_callback,
    )
    .await?;
    Ok(zip_path.to_str().unwrap().to_string())
}

#[tauri::command]
async fn unzip_game(window: tauri::Window, zip_path: &str) -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let target_path = temp_dir.join("world_in_conflict_retail_1.000_en");
    let target_path_clone = target_path.clone();

    let progress_callback = io::create_progress_callback(window.clone(), "extract-game", None);

    let result = io::extract_zip(zip_path, target_path, progress_callback).await;

    if !result.is_ok() {
        return Err(result.err().unwrap().to_string());
    }

    // delete zip
    std::fs::remove_file(zip_path).unwrap();
    Ok(target_path_clone.to_str().unwrap().to_string())
}

#[tauri::command]
async fn download_patch(window: tauri::Window, patch: u16) -> Result<String, String> {
    let base_url = format!("{}/files/patches/", &CONFIG.MASSGATE_URL);

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
    _handle: tauri::AppHandle,
    target_dir: &str,
    installer_dir: &str,
) -> Result<(), String> {
    return install::install_game(target_dir, installer_dir);
}

#[tauri::command]
async fn install_patch(_handle: tauri::AppHandle, installer_path: &str) -> Result<(), String> {
    return install::install_patch(installer_path);
}

#[tauri::command]
async fn install_vcredist(_handle: tauri::AppHandle, vcredist_exe: &str) -> Result<(), String> {
    println!("installing vcredist");
    return install::install_vcredist(vcredist_exe);
}

#[tauri::command]
async fn download_vcredist(window: tauri::Window) -> Result<String, String> {
    let vcredist_url = "https://aka.ms/vs/17/release/vc_redist.x86.exe";
    let target = "vcredist_x86_14.exe";

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
fn clean_install_directory() {
    let paths = vec![
        "world_in_conflict_retail_1.000_en.zip",
        "world_in_conflict_retail_1.000_en",
        "world_in_conflict_1.000_to_1.010_en.exe",
        "world_in_conflict_1.010_to_1.011_en.exe",
        "vcredist_x86_11.exe",
        "vcredist_x86_14.exe",
    ];
    for path in paths {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join(path);
        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }
    }
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
    return install::is_elevated();
}

#[tauri::command]
fn elevate_permissions(handle: tauri::AppHandle) {
    let elevated = install::is_elevated();
    if !elevated {
        install::elevate_permissions(handle);
    }
}

#[tauri::command]
fn needs_hosts_entries() -> Result<bool, String> {
    return Ok(install::needs_hosts_entries());
}
#[tauri::command]
fn add_hosts_entries() -> Result<(), String> {
    return install::add_hosts_entries();
}

#[tauri::command]
fn needs_multicore_fix() -> Result<bool, String> {
    return install::needs_multicore_fix();
}
#[tauri::command]
fn apply_multicore_fix() -> Result<(), String> {
    return install::apply_multicore_fix();
}

#[tauri::command]
fn has_hook_files() -> Result<bool, String> {
    return Ok(install::has_hook_files());
}
#[tauri::command]
fn remove_hook_files() -> Result<(), String> {
    return install::remove_hook_files();
}

#[tauri::command]
fn needs_massgate_fix() -> Result<bool, String> {
    return install::needs_massgate_fix();
}

#[tauri::command]
fn apply_massgate_fix() -> Result<(), String> {
    return install::apply_massgate_fix();
}

#[tauri::command]
fn get_cd_key() -> Result<String, String> {
    return install::get_cd_key();
}
#[tauri::command]
fn set_cd_key(key: Option<&str>) -> Result<(), String> {
    return install::set_cd_key(key);
}

#[tauri::command]
fn needs_vc_redist() -> Result<bool, String> {
    return install::needs_vc_redist();
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
            install_vcredist,
            clean_install_directory,
            needs_hosts_entries,
            add_hosts_entries,
            needs_multicore_fix,
            apply_multicore_fix,
            has_hook_files,
            remove_hook_files,
            needs_massgate_fix,
            apply_massgate_fix,
            get_cd_key,
            set_cd_key,
            needs_vc_redist
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            app.get_window("main").unwrap().open_devtools();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
