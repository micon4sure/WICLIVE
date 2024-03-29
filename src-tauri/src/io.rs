use std::{env, path::PathBuf};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

use futures_util::TryFutureExt;

use std::io::{self, BufReader, Read};
use zip::ZipArchive;

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

    let content_length;

    // X-Filesize comes from API, Content-Length comes from other sources
    let x_filesize = headers.get("X-Filesize");
    if x_filesize.is_some() {
        content_length = x_filesize.unwrap();
    } else {
        let header_content_length = headers.get("Content-Length");
        if header_content_length.is_none() {
            return Err("Failed to get content length".to_string());
        }
        content_length = header_content_length.unwrap();
    }

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

pub fn file_exists(path: &str) -> bool {
    let config = get_base_directory();
    let mut file_path = PathBuf::from(config.unwrap());
    file_path.push(path);
    file_path.exists()
}

pub fn get_file_contents(path: String) -> Result<String, String> {
    let config = get_base_directory();
    let mut file_path = PathBuf::from(config?);
    file_path.push(path);
    let contents = std::fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    Ok(contents)
}

pub fn set_file_contents(path: &str, contents: &str) -> Result<(), String> {
    let config = get_base_directory();
    let mut file_path = PathBuf::from(config?);
    file_path.push(path);

    println!("writing to file {}", file_path.display());
    println!("contents: {}", contents);
    std::fs::write(file_path, contents).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn remove_file(path: &str) -> Result<(), String> {
    let config = get_base_directory();
    let mut file_path = PathBuf::from(config?);
    file_path.push(path);
    std::fs::remove_file(file_path).map_err(|e| e.to_string())?;
    Ok(())
}

// Define a function to generate a progress callback
pub fn create_progress_callback(
    window: tauri::Window,
    download_type: &'static str,
    identifier: Option<String>,
) -> impl FnMut(usize, usize) {
    move |current: usize, total: usize| {
        let percentage = (current as f64 / total as f64) * 100.0;
        let message = match identifier.as_ref() {
            Some(id) => format!(
                "{{\"type\": \"{}\", \"id\": \"{}\", \"percentage\": {:.2}}}",
                download_type, id, percentage
            ),
            None => format!(
                "{{\"type\": \"{}\", \"percentage\": {:.2}}}",
                download_type, percentage
            ),
        };

        window
            .emit("download-progress", message)
            .expect("Failed to emit progress");
    }
}

pub async fn extract_zip<F>(
    zip_path: &str,
    target_path: PathBuf,
    mut progress_callback: F,
) -> Result<(), String>
where
    F: FnMut(usize, usize) + Send + 'static,
{
    // delete target directory if it exists
    if target_path.exists() {
        std::fs::remove_dir_all(&target_path).map_err(|e| e.to_string())?;
    }

    println!("extracting to {}", target_path.display());
    let zip_path = zip_path.to_owned();
    tokio::task::spawn_blocking(move || {
        let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let mut archive = ZipArchive::new(reader).map_err(|e| e.to_string())?;

        let total_files = archive.len();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let outpath = match file.enclosed_name() {
                Some(path) => target_path.join(path),
                None => continue,
            };

            if (*file.name()).ends_with('/') {
                println!("creating directory {}", outpath.display());
                std::fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        println!("creating parent directory {}", p.display());
                        std::fs::create_dir_all(&p).map_err(|e| e.to_string())?;
                    }
                }
                let mut outfile = std::fs::File::create(&outpath).map_err(|e| e.to_string())?;
                println!("copying file {}", outpath.display());
                io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
            }

            progress_callback(i + 1, total_files);
        }
        Result::<(), String>::Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}
