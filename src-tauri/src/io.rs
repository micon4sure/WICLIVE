use std::{env, path::PathBuf};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

use futures_util::stream::StreamExt;

pub async fn get_file_hash(path: PathBuf) -> Result<String, String> {
    // open file
    let mut file = File::open(path).await.map_err(|e| e.to_string())?;

    // create buffer
    let mut buffer = Vec::new();

    // read file into buffer
    file.read_to_end(&mut buffer)
        .await
        .map_err(|e| e.to_string())?;

    // compute md5
    let hash = md5::compute(&buffer);

    // return as hexa string, uppercase
    Ok(format!("{:x}", hash).to_uppercase())
}

pub async fn download_file<F: FnMut(usize, usize) + Send + 'static>(
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

pub fn get_maps_directory() -> Result<PathBuf, String> {
    let base_directory = get_base_directory()?;
    let maps_directory = base_directory.join("Downloaded\\maps");
    if !maps_directory.exists() {
        std::fs::create_dir_all(&maps_directory).map_err(|e| e.to_string())?;
    }
    Ok(maps_directory)
}

pub fn get_base_directory() -> Result<PathBuf, String> {
    // try standard user profile path
    let userprofile = env::var("USERPROFILE").map_err(|e| e.to_string())?;
    let userprofile = PathBuf::from(userprofile);
    let base_directory = userprofile.join("Documents\\World in Conflict");
    if base_directory.exists() {
        return Ok(base_directory);
    }

    // try OneDrive path
    let onedrive = env::var("OneDrive").map_err(|e| e.to_string())?;
    let onedrive = PathBuf::from(onedrive);
    let base_directory = onedrive.join("Documents\\World in Conflict");
    if base_directory.exists() {
        return Ok(base_directory);
    }

    Err("Base directory not found in standard or OneDrive locations.".to_string())
}
