use std::path::PathBuf;

use regex::Regex;

pub fn get_install_directory() -> Result<PathBuf, String> {
    let path = crate::install::find_install_path();
    if path.is_none() {
        return Err("failed to find install path".to_string());
    }
    return Ok(PathBuf::from(path.unwrap()));
}
fn get_patch_directory() -> Result<PathBuf, String> {
    let install_directory = get_install_directory()?;
    let patch_directory = install_directory.join("wiclive-patches");
    if !patch_directory.exists() {
        std::fs::create_dir(&patch_directory).map_err(|e| e.to_string())?;
    }
    return Ok(patch_directory);
}

pub async fn get_patch_files(patches_enabled: bool) -> Result<Vec<String>, String> {
    let patches_directory;
    if patches_enabled {
        patches_directory = get_patch_directory()?;
    } else {
        patches_directory = get_install_directory()?;
    }

    let mut result: Vec<String> = Vec::new();

    let entries = std::fs::read_dir(patches_directory).map_err(|e| e.to_string())?;
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

        // skip non patch files
        let re = Regex::new(r"^wic(\d+)\.sdf$").unwrap();
        if !re.is_match(path.file_name().unwrap().to_str().unwrap()) {
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
pub async fn get_patch_hash(filename: &str, patches_enabled: bool) -> Result<String, String> {
    let patches_directory;
    if patches_enabled {
        patches_directory = get_patch_directory()?;
    } else {
        patches_directory = get_install_directory()?;
    }

    let path = patches_directory.join(filename);

    return crate::io::get_file_hash(path).await;
}

pub async fn download_patch(window: tauri::Window, patch: &str) -> Result<(), String> {
    println!("downloading patch {}", patch);
    let patches_directory = get_patch_directory()?;

    let patch_url = format!("{}/patches/download/{}", &crate::CONFIG.API_URL, patch);

    let progress_callback = crate::io::create_progress_callback(
        window.clone(),
        "download-patch",
        Some(patch.to_string()),
    );

    crate::io::download_file(
        patch_url.as_str(),
        &format!("{}\\{}", patches_directory.display(), patch),
        progress_callback,
    )
    .await
    .map_err(|e| e.to_string())?;
    println!("done downloading patch {}", patch);

    Ok(())
}

pub async fn enable_patches() -> Result<(), String> {
    let install_directory = get_install_directory()?;
    let patch_directory = get_patch_directory()?;

    // if patch subdir does not exist, create it
    if !patch_directory.exists() {
        std::fs::create_dir(&patch_directory).map_err(|e| e.to_string())?;
    }

    let entries = std::fs::read_dir(install_directory).map_err(|e| e.to_string())?;
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

        // skip non patch files
        let re = Regex::new(r"^wic(\d+)\.sdf$").unwrap();
        if !re.is_match(path.file_name().unwrap().to_str().unwrap()) {
            println!("skipping file {}", path.to_str().unwrap());
            continue;
        }
        // if the matched number is less than 100, skip the file
        let caps = re
            .captures(path.file_name().unwrap().to_str().unwrap())
            .unwrap();
        let patch_number = caps.get(1).unwrap().as_str().parse::<i32>().unwrap();
        if patch_number <= 65 {
            println!("skipping file {}", path.to_str().unwrap());
            continue;
        }

        // move file to patch subdir
        let new_path = patch_directory.join(path.file_name().unwrap());
        std::fs::rename(&path, &new_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub async fn disable_patches() -> Result<(), String> {
    let install_directory = get_install_directory()?;
    let patch_directory = get_patch_directory()?;

    // if patch subdir does not exist, create it
    if !patch_directory.exists() {
        std::fs::create_dir(&patch_directory).map_err(|e| e.to_string())?;
    }

    let patch_subdir = install_directory.join("wiclive-patches");
    if !patch_subdir.exists() {
        return Ok(());
    }

    let entries = std::fs::read_dir(patch_subdir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?; // Properly handle the Result
        let path = entry.path(); // Bind the path to a variable

        // move file to patches directory
        let new_path = install_directory.join(path.file_name().unwrap());
        std::fs::rename(&path, &new_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}
