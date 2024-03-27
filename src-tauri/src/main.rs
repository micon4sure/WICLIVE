// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows::{
    core::{s, PCWSTR, PWSTR},
    Win32::Storage::FileSystem::{
        GetFileVersionInfoA, GetFileVersionInfoSizeA, GetFileVersionInfoSizeW, GetFileVersionInfoW,
        VerQueryValueA, VerQueryValueW, VS_FIXEDFILEINFO,
    },
};

use serde::Serialize;
use std::ffi::{c_void, OsStr};
use std::{env, mem::MaybeUninit, os::windows::ffi::OsStrExt, path::PathBuf};
use tauri::{Manager, PhysicalSize, Size};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

use futures_util::stream::StreamExt;

#[derive(Serialize)]
struct Config {
    API_URL: String,
    VERSION: String,
}

impl Config {
    fn new() -> Self {
        // set api url
        let env = env!("WICLIVE_ENV");
        let api_url = match env {
            "development" => "http://localhost:3243".to_string(),
            "staging" => "https://techtile.media:3243".to_string(),
            "production" => "https://techtile.media:3243".to_string(),
            _ => "http://localhost:3243".to_string(),
        };

        // get version from cargo.toml
        let version = env!("CARGO_PKG_VERSION").to_string();
        Config {
            API_URL: api_url,
            VERSION: version,
        }
    }
}

lazy_static::lazy_static! {
    static ref CONFIG: Config = Config::new();
}

#[tauri::command]
async fn get_map_hash(filename: &str) -> Result<String, String> {
    let userprofile = env::var("USERPROFILE").map_err(|e| e.to_string())?;
    let maps_directory =
        PathBuf::from(userprofile).join("Documents\\World in Conflict\\Downloaded\\maps");
    let path = maps_directory.join(filename);

    // Open the file asynchronously
    let mut file = File::open(path).await.map_err(|e| e.to_string())?;

    // Create a new, empty buffer
    let mut buffer = Vec::new();

    // Read the file's contents into the buffer asynchronously
    file.read_to_end(&mut buffer)
        .await
        .map_err(|e| e.to_string())?;

    // Compute the MD5 hash of the buffer's contents
    let hash = md5::compute(&buffer);

    // Return the hash as a hexadecimal string, uppercase
    Ok(format!("{:x}", hash).to_uppercase())
}

async fn download_file<F: FnMut(usize, usize) + Send + 'static>(
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

#[tauri::command]
async fn get_map_files() -> Result<Vec<String>, String> {
    use std::env;
    use std::path::PathBuf;

    let userprofile = env::var("USERPROFILE").map_err(|e| e.to_string())?;
    let maps_directory =
        PathBuf::from(userprofile).join("Documents\\World in Conflict\\Downloaded\\maps");

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
    let userprofile = env::var("USERPROFILE").map_err(|e| e.to_string())?;
    let maps_directory = userprofile.clone() + "\\Documents\\World in Conflict\\Downloaded\\maps";

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
        format!("{}\\{}", maps_directory, map).as_str(),
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

#[tauri::command]
fn find_install_path() -> Result<String, String> {
    println!("finding install path");

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    println!("hklm: {:?}", hklm);

    // Updated path to include WOW6432Node for 32-bit application support on 64-bit Windows
    let subkey_path = r"SOFTWARE\WOW6432Node\Massive Entertainment AB\World in Conflict";
    println!("subkey_path: {:?}", subkey_path);

    let subkey = hklm.open_subkey(subkey_path);
    match subkey {
        Ok(regkey) => {
            println!("regkey: {:?}", regkey);
            let install_location: String =
                regkey.get_value("InstallPath").map_err(|e| e.to_string())?;
            return Ok(install_location);
        }
        Err(e) => {
            println!("error: {:?}", e);
            return Err("not installed".to_string());
        }
    }
}

fn to_wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
#[tauri::command]
async fn extract_game_version(install_path: &str) -> Result<String, String> {
    unsafe {
        let path_exe = install_path.to_string() + "\\wic.exe";

        // encode to utf16 -> PCW
        let path_exe_utf: Vec<_> = path_exe.encode_utf16().chain(std::iter::once(0)).collect();
        let path_exe_pcw = PCWSTR::from_raw(path_exe_utf.as_ptr());

        // get file version info size.
        let data_len = GetFileVersionInfoSizeW(path_exe_pcw, None);
        if data_len == 0 {
            return Err(windows::core::Error::from_win32()).map_err(|e| e.to_string())?;
        }

        // convert len to usize
        let data_len_usize: usize = data_len.try_into().unwrap();

        // allocate buffer to hold the file version info
        let mut data = vec![0u8; data_len_usize];

        // fix data size
        let data = &mut data[..];

        // read file version info into data buffer
        let result =
            GetFileVersionInfoW(path_exe_pcw, 0, data_len, data.as_mut_ptr() as *mut c_void);
        result.map_err(|e| e.to_string())?;

        // create info pointer and len to be written into
        let mut info_ptr: *mut VS_FIXEDFILEINFO = std::ptr::null_mut();
        let mut info_len: u32 = 0;

        // create pcwstring
        let wide_string = to_wide_string(r"\");
        let pcwstr = PCWSTR(wide_string.as_ptr() as *mut _);

        // read value from data buffer
        let ok = VerQueryValueW(
            data.as_ptr() as *const c_void,
            pcwstr,
            (&mut info_ptr) as *mut _ as *mut *mut c_void,
            &mut info_len,
        );

        // assert that the value was read
        assert!(!info_ptr.is_null());
        assert_eq!(info_len as usize, std::mem::size_of::<VS_FIXEDFILEINFO>());

        // get the value from the pointer
        let ffi = info_ptr.read_unaligned();

        // extract version info
        let major = (ffi.dwFileVersionMS >> 16) & 0xFFFF; // Extract major version
        let minor = ffi.dwFileVersionMS & 0xFFFF; // Extract minor version
        let build = (ffi.dwFileVersionLS >> 16) & 0xFFFF; // Extract build number
        let revision = ffi.dwFileVersionLS & 0xFFFF; // Extract revision number

        // format into version string
        let version = format!("{}.{}.{}.{}", major, minor, build, revision);

        Ok(version)
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_map_files,
            get_map_hash,
            download_map,
            get_config,
            find_install_path,
            extract_game_version
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            app.get_window("main").unwrap().open_devtools();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
