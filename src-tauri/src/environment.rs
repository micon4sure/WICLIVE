use std::fs;
use std::path::PathBuf;

use crate::install;

pub fn set(_handle: tauri::AppHandle, dll_dir: &str) -> Result<(), String> {
    // get install dir from install module
    let install_dir = install::find_install_path().ok_or("Failed to find install path")?;

    println!("install_dir: {}, dll_dir: {}", install_dir, dll_dir);

    // Ensure the install directory exists
    fs::create_dir_all(&install_dir)
        .map_err(|e| format!("Failed to create install directory: {}", e))?;

    // Read the contents of the DLL directory
    let dir = fs::read_dir(dll_dir).map_err(|e| format!("Failed to read directory: {}", e))?;

    println!("installing files to: {:?}", install_dir);

    for entry in dir {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if !path.is_file() {
            println!("skipping {:?}", path);
            continue;
        }
        let dest_path =
            PathBuf::from(&install_dir).join(path.file_name().ok_or("Failed to get file name")?);

        println!("copying file from {:?} to {:?}", path, dest_path);
        fs::copy(&path, &dest_path)
            .map_err(|e| format!("Failed to copy file {:?}: {}", path, e))?;
    }

    Ok(())
}
