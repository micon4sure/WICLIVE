use powershell_script;
use std::ffi::{c_void, OsStr};
use std::fs;
use std::{env, os::windows::ffi::OsStrExt, path::PathBuf};
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winreg::RegKey;

use windows::{
    core::PCWSTR,
    Win32::Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW, VS_FIXEDFILEINFO,
    },
};

use serde::Serialize;

use crate::CONFIG;

#[derive(Serialize)]
pub struct VersionInfo {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub build: u16,
}
pub fn is_elevated() -> bool {
    use std::ptr::null_mut;
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::processthreadsapi::GetCurrentProcess;
    use winapi::um::processthreadsapi::OpenProcessToken;
    use winapi::um::securitybaseapi::GetTokenInformation;
    use winapi::um::winnt::TokenElevation;
    use winapi::um::winnt::HANDLE;
    use winapi::um::winnt::TOKEN_ELEVATION;

    use std::mem;
    use winapi::ctypes::c_void;
    use winapi::um::winnt::TOKEN_QUERY;

    let mut handle: HANDLE = null_mut();
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut handle) };

    let elevation = unsafe { libc::malloc(mem::size_of::<TOKEN_ELEVATION>()) as *mut c_void };
    let size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;
    let mut ret_size = size;
    unsafe {
        GetTokenInformation(
            handle,
            TokenElevation,
            elevation,
            size as u32,
            &mut ret_size,
        )
    };
    let elevation_struct: TOKEN_ELEVATION = unsafe { *(elevation as *mut TOKEN_ELEVATION) };

    if !handle.is_null() {
        unsafe {
            CloseHandle(handle);
        }
    }

    elevation_struct.TokenIsElevated == 1
}
pub fn elevate_permissions(handle: tauri::AppHandle) {
    if is_elevated() {
        return;
    }
    let binary = std::env::current_exe().unwrap();
    let runner = powershell_script::PsScriptBuilder::new()
        .non_interactive(true)
        .build();

    let script;
    if CONFIG.ENV == "development" {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let home_dir = path.parent().unwrap().display();

        println!("manifest_dir: {:?}", home_dir);
        script = format!(
            "Start-Process -FilePath \"powershell\" -ArgumentList \"-NoExit\", \"cd {}; bun run tauri dev\" -Verb RunAs",
            home_dir,
        );
    } else {
        script = format!(
            "Start-Process -FilePath \"{}\" -Verb RunAs",
            binary.to_str().unwrap()
        );
    }
    println!("script: {}", script);
    let output = runner.run(script.as_str()).unwrap();
    println!("output: {}", output);
    // exit the current process
    handle.exit(0)
}

pub fn find_install_path() -> Option<String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    let primary_path = r"SOFTWARE\WOW6432Node\Massive Entertainment AB\World in Conflict";
    let secondary_path = r"SOFTWARE\WOW6432Node\GOG.com\Games\1438332414";

    // Try to open the primary subkey
    let subkey = hklm.open_subkey(primary_path);
    match subkey {
        Ok(regkey) => {
            if let Ok(install_location) = regkey.get_value::<String, _>("InstallPath") {
                return Some(install_location);
            }
        }
        Err(_e) => {} // Ignore error and continue to check the secondary path
    }

    // Try to open the secondary subkey
    let subkey = hklm.open_subkey(secondary_path);
    match subkey {
        Ok(regkey) => {
            if let Ok(install_location) = regkey.get_value::<String, _>("WORKINGDIR") {
                return Some(install_location);
            }
        }
        Err(_e) => {} // Ignore error and return None
    }

    None
}

fn to_wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

pub async fn extract_game_version() -> Result<VersionInfo, String> {
    let install_path = match find_install_path() {
        Some(path) => path,
        None => return Err("install path not found".to_string()),
    };
    let path_exe = install_path.to_string() + "\\wic.exe";

    if !PathBuf::from(&path_exe).exists() {
        return Err("install path found but exe not present".to_string());
    }

    unsafe {
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
        if !ok.as_bool() {
            return Err(windows::core::Error::from_win32()).map_err(|e| e.to_string())?;
        }

        // assert that the value was read
        assert!(!info_ptr.is_null());
        assert_eq!(info_len as usize, std::mem::size_of::<VS_FIXEDFILEINFO>());

        // get the value from the pointer
        let ffi = info_ptr.read_unaligned();

        // extract version info
        let major = (ffi.dwFileVersionMS >> 16) & 0xFFFF; // Extract major version
        let minor = ffi.dwFileVersionMS & 0xFFFF; // Extract minor version
        let patch = (ffi.dwFileVersionLS >> 16) & 0xFFFF; // Extract build number
        let build = ffi.dwFileVersionLS & 0xFFFF; // Extract revision number

        // format into version string
        let version = VersionInfo {
            major: major as u16,
            minor: minor as u16,
            patch: patch as u16,
            build: build as u16,
        };

        Ok(version)
    }
}

pub fn get_cd_key() -> Result<String, String> {
    let tmpdir = std::env::temp_dir();
    println!("tmpdir: {:?}", tmpdir);

    // Define the registry key path
    let reg_path = r"Software\Massive Entertainment AB\World In Conflict";
    println!("reg_path: {}", reg_path);

    // Open the HKEY_CURRENT_USER registry hive
    let hkey_current_user = RegKey::predef(HKEY_CURRENT_USER);

    // Attempt to open the subkey for World In Conflict
    let subkey = match hkey_current_user.open_subkey(reg_path) {
        Ok(sk) => {
            println!("Opened registry path: {}", reg_path);
            sk
        }
        Err(e) => {
            println!("Failed to open registry path: {}", e);
            return Ok(String::new()); // Return empty string if the key does not exist
        }
    };

    // Attempt to retrieve the CDKEY value
    match subkey.get_value("CDKEY") {
        Ok(cd_key) => {
            println!("CDKEY: {}", cd_key);
            Ok(cd_key)
        }
        Err(e) => {
            println!("Failed to get CDKEY value: {}", e);
            Ok(String::new()) // Return empty string if the value is not set
        }
    }
}
pub fn set_cd_key(cd_key: Option<&str>) -> Result<(), String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let cd_key = cd_key.unwrap_or("3EXO-ELED-MXGY-FP5M-286R");

    println!("Setting CDKEY in the registry: {}", cd_key);

    // Define the registry key path
    let reg_path = r"Software\Massive Entertainment AB\World In Conflict";

    // Open the HKEY_CURRENT_USER registry hive
    let hkey_current_user = RegKey::predef(HKEY_CURRENT_USER);

    // Open or create the subkey for World In Conflict
    let (subkey, _) = hkey_current_user
        .create_subkey(reg_path)
        .map_err(|e| format!("Failed to open or create registry key: {}", e))?;

    // Set the CDKEY value
    subkey
        .set_value("CDKEY", &cd_key)
        .map_err(|e| format!("Failed to set CDKEY in registry: {}", e))?;

    println!("Successfully set CDKEY in the registry.");
    Ok(())
}

pub fn needs_vc_redist() -> Result<bool, String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let subkey =
        hklm.open_subkey(r"SOFTWARE\WOW6432Node\Microsoft\VisualStudio\14.0\VC\Runtimes\X86");
    match subkey {
        Ok(_) => {
            println!("VC Redist already installed");
            return Ok(false);
        }
        Err(_) => {
            println!("VC Redist not installed");
            return Ok(true);
        }
    }
}

pub fn needs_hooks() -> bool {
    let install_dir = find_install_path();
    let mut hooks_version = PathBuf::from(install_dir.unwrap());
    hooks_version.push("wic_cl_hook-version.txt");

    !hooks_version.exists()
}

pub fn get_hooks_version() -> Option<String> {
    let hook_version = resolve_path("dlls", "wic_cl_hook-version.txt");
    let hook_version = PathBuf::from(hook_version);

    println!("trying to get hooks version from: {:?}", hook_version);

    if hook_version.exists() {
        fs::read_to_string(hook_version).ok()
    } else {
        None
    }
}

pub fn needs_hooks_update() -> bool {
    // Read contents of hooks_version.txt
    let install_dir = find_install_path();
    let mut hooks_version_file_installed = PathBuf::from(install_dir.unwrap());
    hooks_version_file_installed.push("wic_cl_hook-version.txt");

    let installed_version = fs::read_to_string(hooks_version_file_installed).unwrap_or_default();

    // Compare with current version
    let hooks_version_file_current = resolve_path("dlls", "wic_cl_hook-version.txt");
    let current_version = fs::read_to_string(hooks_version_file_current).unwrap_or_default();

    println!("installed_version: {}", installed_version);
    println!("current_version: {}", current_version);

    installed_version != current_version
}

pub fn create_desktop_shortcut() -> Result<(), String> {
    let install_dir = find_install_path().unwrap();
    let exe_path = PathBuf::from(install_dir).join("wic.exe");

    let script_path = resolve_path("automation", "create_shortcut.ps1");

    std::process::Command::new("powershell")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-File")
        .arg(script_path)
        .arg(exe_path)
        .output()
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn resolve_path(dir: &str, resource: &str) -> String {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path.push("_up_");
    path.push(dir);
    path.push(resource);
    path.to_str().unwrap().to_string()
}
