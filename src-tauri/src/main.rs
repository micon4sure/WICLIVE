// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod environment;
mod install;
mod io;
mod map;
mod patch;

use install::VersionInfo;

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
    return map::get_map_files().await;
}
#[tauri::command]
async fn get_map_hash(filename: &str) -> Result<String, String> {
    return map::get_map_hash(filename).await;
}

#[tauri::command]
async fn download_map_live(window: tauri::Window, map: &str) -> Result<(), String> {
    return map::download_map_live(window, map).await;
}

#[tauri::command]
async fn download_map_custom(window: tauri::Window, map: &str) -> Result<(), String> {
    return map::download_map_custom(window, map).await;
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
async fn install_vcredist(_handle: tauri::AppHandle, vcredist_exe: &str) -> Result<(), String> {
    println!("installing vcredist");
    return install::install_vcredist(vcredist_exe);
}

#[tauri::command]
async fn download_vcredist(window: tauri::Window, version: u8) -> Result<String, String> {
    return install::download_vcredist(window, version).await;
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
fn environment_set(handle: tauri::AppHandle, environment: &str) {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path.push("_up_");
    path.push("dlls");
    path.push(environment);
    let _ = environment::set(handle, path.to_str().unwrap());
}

#[tauri::command]
async fn get_patch_files(patches_enabled: bool) -> Result<Vec<String>, String> {
    return patch::get_patch_files(patches_enabled).await;
}
#[tauri::command]
async fn get_patch_hash(filename: &str, patches_enabled: bool) -> Result<String, String> {
    return patch::get_patch_hash(filename, patches_enabled).await;
}
#[tauri::command]
async fn download_patch(window: tauri::Window, name: &str) -> Result<(), String> {
    return patch::download_patch(window, name).await;
}

#[tauri::command]
async fn enable_patches() -> Result<(), String> {
    return patch::enable_patches().await;
}
#[tauri::command]
async fn disable_patches() -> Result<(), String> {
    return patch::disable_patches().await;
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
            install_vcredist,
            environment_set,
            download_patch,
            get_patch_files,
            get_patch_hash,
            enable_patches,
            disable_patches
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            app.get_window("main").unwrap().open_devtools();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
