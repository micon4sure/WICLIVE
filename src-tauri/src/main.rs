// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod install;
mod io;

use install::VersionInfo;
use io::download_file;
use io::get_file_hash;
use io::get_maps_directory;

use dotenv::dotenv;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Stdio;

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

    if patch_path.exists() {
        return Ok(patch_path.to_str().unwrap().to_string());
    }

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
    let automate_game_exe = install::resolve_path("automation", "automate_game.exe");
    let mut setup_exe = PathBuf::from(installer_dir);
    setup_exe.push("Installer");

    let setup_path = setup_exe.clone();

    setup_exe.push("setup.exe");

    // run automate in the background
    println!("running automate: {:?}", automate_game_exe);
    std::process::Command::new(automate_game_exe)
        .arg(target_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start automate game process");

    println!("running installer: {:?}", setup_exe.display());
    // run installer
    let output = std::process::Command::new(setup_exe)
        .current_dir(setup_path)
        .output()
        .map_err(|e| e.to_string())?;

    println!("installer output: {:?}", output);

    // delete install file
    std::fs::remove_dir_all(installer_dir).map_err(|e| e.to_string())?;
    return Ok(());
}

#[tauri::command]
async fn install_patch(_handle: tauri::AppHandle, installer_path: &str) -> Result<(), String> {
    let automate_patch_exe = install::resolve_path("automation", "automate_patch.exe");

    // run accept_eula in the background
    println!("running automate: {:?}", automate_patch_exe);
    std::process::Command::new(automate_patch_exe)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start automate patch process");

    // get directory of installer.exe
    let mut installer_dir = PathBuf::from(installer_path);
    installer_dir.pop();

    println!("running installer: {:?}", installer_path);
    // run installer
    let output = std::process::Command::new(installer_path)
        .current_dir(installer_dir.clone())
        .output()
        .map_err(|e| e.to_string())?;

    println!("installer output: {:?}", output);

    Ok(())
}

#[tauri::command]
async fn install_vcredist(_handle: tauri::AppHandle, vcredist_exe: &str) -> Result<(), String> {
    println!("installing vcredist");
    println!("installing vcredist: {:?}", vcredist_exe);

    let output = std::process::Command::new(vcredist_exe)
        .arg("/install")
        .arg("/quiet")
        .arg("/norestart")
        .output()
        .map_err(|e| e.to_string())?;

    println!("installer output: {:?}", output);

    // delete install file
    std::fs::remove_file(vcredist_exe).map_err(|e| e.to_string())?;

    return Ok(());
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

#[tauri::command]
fn get_secret(secret: &str) -> Option<String> {
    dotenv().ok(); // Load environment variables from .env file
    println!("getting secret {}", secret);
    env::var(secret).ok()
}

#[tauri::command]
fn needs_hooks() -> bool {
    return install::needs_hooks();
}

#[tauri::command]
fn needs_hooks_update() -> bool {
    return install::needs_hooks_update();
}

#[tauri::command]
fn get_hooks_version() -> Option<String> {
    return install::get_hooks_version();
}

#[tauri::command]
async fn download_hooks(window: tauri::Window) -> Result<String, String> {
    let version = install::get_hooks_version();
    if version.is_none() {
        return Err("failed to get hooks version".to_string());
    }
    let hooks_url = format!(
        "https://www.wicgate.com/wicgate_update_{}.zip",
        version.unwrap()
    );

    println!("downloading hooks from {}", hooks_url);

    let temp_dir = std::env::temp_dir();
    let zip_path = temp_dir.join("hooks.zip");

    let progress_callback = io::create_progress_callback(window.clone(), "download-hooks", None);

    download_file(
        hooks_url.as_str(),
        zip_path.to_str().unwrap(),
        progress_callback,
    )
    .await?;
    Ok(zip_path.to_str().unwrap().to_string())
}

#[tauri::command]
async fn unzip_hooks(window: tauri::Window, zip_path: &str) -> Result<(), String> {
    // Find the install directory.
    let install_dir = install::find_install_path().unwrap();

    let temp_subdir = std::env::temp_dir();
    let temp_subdir = temp_subdir.join("hooks_unzipped");
    println!("checking temp subdir {}", temp_subdir.display());
    if !temp_subdir.exists() {
        println!("creating temp subdir");
        fs::create_dir(&temp_subdir).map_err(|e| e.to_string())?;
    }

    // Extract zip into the subdirectory.
    let progress_callback = io::create_progress_callback(window.clone(), "extract-hooks", None);
    println!("extracting zip");
    let result = io::extract_zip(zip_path, temp_subdir.clone(), progress_callback).await;
    if let Err(e) = result {
        println!("failed to extract zip: {}", e);
        return Err(e.to_string());
    }
    println!("extracted zip");

    for entry in fs::read_dir(&temp_subdir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let mut target = install_dir.clone();
        target.push_str(path.file_name().unwrap().to_str().unwrap());
        println!("copying {}", path.display());
        fs::copy(&path, &target).map_err(|e| e.to_string())?;
    }

    println!("done copying files");

    // Delete the original zip file.
    fs::remove_file(zip_path).map_err(|e| e.to_string())?;

    println!("done deleting zip");

    // Delete the temporary subdirectory.
    fs::remove_dir_all(&temp_subdir).map_err(|e| e.to_string())?;

    println!("done deleting temp subdir");
    Ok(())
}

#[tauri::command]
fn create_desktop_shortcut() -> Result<(), String> {
    return install::create_desktop_shortcut();
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
            get_cd_key,
            set_cd_key,
            needs_vc_redist,
            needs_hooks,
            needs_hooks_update,
            get_hooks_version,
            download_hooks,
            unzip_hooks,
            create_desktop_shortcut,
            get_secret,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            app.get_window("main").unwrap().open_devtools();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
