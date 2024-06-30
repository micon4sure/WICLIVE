pub async fn get_map_files() -> Result<Vec<String>, String> {
    let maps_directory = crate::io::get_maps_directory()?;

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

pub async fn get_map_hash(filename: &str) -> Result<String, String> {
    let maps_directory = crate::io::get_maps_directory()?;
    let path = maps_directory.join(filename);

    return crate::io::get_file_hash(path).await;
}

pub async fn download_map_live(window: tauri::Window, map: &str) -> Result<(), String> {
    println!("downloading live map {}", map);
    let maps_directory = crate::io::get_maps_directory()?;

    let map_url = format!("{}/maps/download/{}", &crate::CONFIG.API_URL, map);

    let progress_callback = crate::io::create_progress_callback(
        window.clone(),
        "download-map-live",
        Some(map.to_string()),
    );

    crate::io::download_file(
        map_url.as_str(),
        &format!("{}\\{}", maps_directory.display(), map),
        progress_callback,
    )
    .await
    .map_err(|e| e.to_string())?;
    println!("done downloading live map {}", map);

    Ok(())
}

pub async fn download_map_custom(window: tauri::Window, map: &str) -> Result<(), String> {
    println!("downloading custom map {}", map);
    let maps_directory = crate::io::get_maps_directory()?;

    let map_url = format!("{}/files/maps/{}", &crate::CONFIG.MASSGATE_URL, map);

    let progress_callback = crate::io::create_progress_callback(
        window.clone(),
        "download-map-custom",
        Some(map.to_string()),
    );

    crate::io::download_file(
        map_url.as_str(),
        &format!("{}\\{}", maps_directory.display(), map),
        progress_callback,
    )
    .await
    .map_err(|e| e.to_string())?;
    println!("done downloading custom map {}", map);

    Ok(())
}
