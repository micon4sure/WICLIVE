use powershell_script;
use std::borrow::BorrowMut;
use std::ffi::{c_void, OsStr};
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread::{self, sleep};
use std::time::Duration;
use std::{env, mem::MaybeUninit, os::windows::ffi::OsStrExt, path::PathBuf};
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

use windows::{
    core::{s, PCWSTR, PWSTR},
    Win32::Storage::FileSystem::{
        GetFileVersionInfoA, GetFileVersionInfoSizeA, GetFileVersionInfoSizeW, GetFileVersionInfoW,
        VerQueryValueA, VerQueryValueW, VS_FIXEDFILEINFO,
    },
};

use serde::Serialize;

use crate::{init, io};

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
    // check for debug profile
    let debug = env::var("DEBUG").is_ok();

    println!("elevating permissions, debug: {:?}", debug);

    let binary = std::env::current_exe().unwrap();
    let runner = powershell_script::PsScriptBuilder::new()
        .non_interactive(true)
        .build();

    let script;
    if cfg!(debug_assertions) {
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
            let install_location: String = regkey.get_value("InstallPath").unwrap();
            return Some(install_location);
        }
        Err(e) => {
            return None;
        }
    }
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
        None => return Err("not installed".to_string()),
    };
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

pub fn check_vcredist_installed() -> bool {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let subkey_path = r"SOFTWARE\WOW6432Node\Microsoft\VisualStudio\14.0\VC\Runtimes\x86";
    let subkey = hklm.open_subkey(subkey_path);
    match subkey {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn install_game<F>(target_dir: &str, installer_dir: &str, resolver: F) -> Result<(), String>
where
    F: Fn(&str) -> String,
{
    let automate_game_exe = resolver("automate_game.exe");
    let mut setup_exe = PathBuf::from(installer_dir);
    setup_exe.push("Installer");
    setup_exe.push("setup.exe");

    // run automate in the background
    println!("running automate: {:?}", automate_game_exe);
    std::process::Command::new(automate_game_exe)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start automate game process");

    println!("running installer: {:?}", setup_exe.display());
    // run installer
    let output = std::process::Command::new(setup_exe)
        .output()
        .map_err(|e| e.to_string())?;

    println!("installer output: {:?}", output);

    return Ok(());
}

pub fn install_patch(installer_path: &str, resolver: fn(&str) -> String) -> Result<(), String> {
    let automate_patch_exe = resolver("automate_patch.exe");

    // run accept_eula in the background
    println!("running automate: {:?}", automate_patch_exe);
    std::process::Command::new(automate_patch_exe)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start automate patch process");

    println!("running installer: {:?}", installer_path);
    // run installer
    let output = std::process::Command::new(installer_path)
        .output()
        .map_err(|e| e.to_string())?;

    println!("installer output: {:?}", output);

    Ok(())
}
pub fn install_vcredist(installer_dir: &str) -> Result<(), String> {
    let mut vcredist_exe = PathBuf::from(installer_dir);
    vcredist_exe.push("vc_redist.x86.exe");

    println!("running vcredist: {:?}", vcredist_exe.display());

    let output = std::process::Command::new(vcredist_exe)
        .arg("/install")
        .arg("/quiet")
        .arg("/norestart")
        .output()
        .map_err(|e| e.to_string())?;

    println!("installer output: {:?}", output);

    return Ok(());
}
