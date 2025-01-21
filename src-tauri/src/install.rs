use powershell_script;
use std::ffi::{c_void, OsStr};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::process::Stdio;
use std::time::{Duration, SystemTime};
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
    if CONFIG.DEBUG {
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

pub fn install_vcredist(vcredist_exe: &str) -> Result<(), String> {
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
pub fn install_game(target_dir: &str, installer_dir: &str) -> Result<(), String> {
    let automate_game_exe = resolve_path("automation", "automate_game.exe");
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

pub fn install_patch(installer_path: &str) -> Result<(), String> {
    let automate_patch_exe = resolve_path("automation", "automate_patch.exe");

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

    // delete install file
    std::fs::remove_dir_all(installer_dir).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn needs_hosts_entries() -> bool {
    let hosts_path = r"C:\Windows\System32\drivers\etc\hosts";
    let file = File::open(hosts_path).unwrap();
    let reader = BufReader::new(file);
    let existing_lines: Vec<String> = reader.lines().map(|l| l.unwrap_or_default()).collect();

    // check if redirects exist for these domains
    let desired_entries = [
        "massgate.net",
        "liveaccount.massgate.net",
        "liveaccountbackup.massgate.net",
        "stats.massgate.net",
        "www.massgate.net",
    ];

    let missing_entries: Vec<&str> = desired_entries
        .iter()
        .filter(|entry| !existing_lines.iter().any(|line| line.contains(*entry)))
        .copied()
        .collect();

    println!("missing entries: {:?}", missing_entries);

    !missing_entries.is_empty()
}
pub fn add_hosts_entries() -> Result<(), String> {
    let hosts_path = r"C:\Windows\System32\drivers\etc\hosts";

    // The block of host entries you want to add
    let desired_entries = [
        "89.163.230.140 massgate.net",
        "89.163.230.140 liveaccount.massgate.net",
        "89.163.230.140 liveaccountbackup.massgate.net",
        "89.163.230.140 stats.massgate.net",
        "89.163.230.140 www.massgate.net",
    ];

    // 1. Read the existing hosts file and collect lines
    let file = File::open(hosts_path).map_err(|e| format!("Could not open hosts file: {}", e))?;
    let reader = BufReader::new(file);
    let existing_lines: Vec<String> = reader.lines().map(|l| l.unwrap_or_default()).collect();

    // 2. Filter desired entries to only those that are *not* already in hosts
    let missing_entries: Vec<&str> = desired_entries
        .iter()
        .filter(|entry| !existing_lines.iter().any(|line| line == *entry))
        .copied()
        .collect();

    // If all entries exist, nothing to do
    if missing_entries.is_empty() {
        return Ok(());
    }

    // 3. Create the text we want to append, ensuring we start with a new line and use \r\n
    // so that each entry is on its own line in Windows style.
    let to_append = format!("\r\n{}", missing_entries.join("\r\n"));

    // 4. Append to the hosts file
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(hosts_path)
        .map_err(|e| format!("Failed to open hosts file for append: {}", e))?;

    file.write_all(to_append.as_bytes())
        .map_err(|e| format!("Failed to write to hosts file: {}", e))?;

    Ok(())
}

pub fn needs_multicore_fix() -> Result<bool, String> {
    // Check CPU core count
    let thread_count = num_cpus::get();
    if thread_count <= 12 {
        return Ok(false); // No fix needed if 12 or fewer threads
    }

    // log
    println!(
        "Detected {} CPU threads, multicore fix needed.",
        thread_count
    );

    // Get the installation path
    let install_path = find_install_path().unwrap();
    let mut target_path = PathBuf::from(install_path);
    target_path.push("dbghelp.dll");

    // Check if the file exists and was modified before 2017
    if let Ok(metadata) = fs::metadata(&target_path) {
        if let Ok(modified_time) = metadata.modified() {
            let cutoff_date = SystemTime::UNIX_EPOCH + Duration::from_secs(1483228800); // January 1, 2017
            println!(
                "File modified time: {:?}, cutoff date: {:?}, fix needed: {}",
                modified_time,
                cutoff_date,
                modified_time < cutoff_date
            );
            return Ok(modified_time < cutoff_date);
        }
    }

    Ok(true) // Default to fix if file metadata can't be checked
}
pub fn apply_multicore_fix() -> Result<(), String> {
    let dll_path = resolve_path("dlls", "dbghelp.dll");

    if !PathBuf::from(&dll_path).exists() {
        return Err("DLL file not found".to_string());
    }

    // Find the install path using the `find_install_path` function
    let install_path = find_install_path().ok_or("Install path not found")?;

    // Construct the target path for the DLL
    let mut target_path = PathBuf::from(install_path);
    target_path.push("dbghelp.dll");

    // Print paths for debugging
    println!("Source DLL path: {:?}", dll_path);
    println!("Target DLL path: {:?}", target_path);

    // Copy the DLL file to the target directory, overwriting if it already exists
    std::fs::copy(&dll_path, &target_path).map_err(|e| format!("Failed to copy DLL: {}", e))?;

    println!("Successfully applied the multicore fix by copying the DLL.");

    Ok(())
}

pub fn has_hook_files() -> bool {
    let install_path = find_install_path().unwrap();
    let mut target_path = PathBuf::from(install_path.clone());
    target_path.push("wic_cl_hook.dll");
    println!("checking for hook file {:?}", target_path);
    if target_path.exists() {
        println!("wic_cl_hook.dll exists");
        return true;
    }

    let mut target_path = PathBuf::from(install_path.clone());
    target_path.push("wic_ds_hook.dll");

    println!("checking for hook file {:?}", target_path);
    if target_path.exists() {
        println!("wic_ds_hook.dll exists");
        return true;
    }

    println!("hook files not found in {}", install_path);
    false
}

pub fn remove_hook_files() -> Result<(), String> {
    let install_path = find_install_path().unwrap();
    let mut target_path = PathBuf::from(install_path.clone());
    target_path.push("wic_cl_hook.dll");
    if target_path.exists() {
        fs::remove_file(&target_path).map_err(|e| e.to_string())?;
        println!("removed wic_cl_hook.dll");
    }

    let mut target_path = PathBuf::from(install_path.clone());
    target_path.push("wic_ds_hook.dll");
    if target_path.exists() {
        fs::remove_file(&target_path).map_err(|e| e.to_string())?;
        println!("removed wic_ds_hook.dll");
    }

    Ok(())
}

pub fn needs_massgate_fix() -> Result<bool, String> {
    println!("checking for massgate fix");
    if has_hook_files() {
        println!("hook files found");
        return Ok(true);
    }

    if needs_hosts_entries() {
        println!("hosts entries missing");
        return Ok(true);
    }

    println!("no massgate fix needed");
    return Ok(false);
}

pub fn apply_massgate_fix() -> Result<(), String> {
    if has_hook_files() {
        // remove the hook files if they exist
        let install_path = find_install_path().unwrap();
        let mut target_path = PathBuf::from(install_path.clone());
        target_path.push("wic_cl_hook.dll");
        if target_path.exists() {
            fs::remove_file(&target_path).map_err(|e| e.to_string())?;
            println!("removed wic_cl_hook.dll");
        }

        let mut target_path = PathBuf::from(install_path.clone());
        target_path.push("wic_ds_hook.dll");
        if target_path.exists() {
            fs::remove_file(&target_path).map_err(|e| e.to_string())?;
            println!("removed wic_ds_hook.dll");
        }
    }

    // Apply the multicore fix
    apply_multicore_fix()?;

    if needs_hosts_entries() {
        add_hosts_entries()?;
    }

    Ok(())
}

pub fn get_cd_key() -> Result<String, String> {
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
        hklm.open_subkey(r"SOFTWARE\WOW6432Node\Microsoft\VisualStudio\14.0\VC\Runtimes\x64");
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

fn resolve_path(dir: &str, resource: &str) -> String {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path.push("_up_");
    path.push(dir);
    path.push(resource);
    path.to_str().unwrap().to_string()
}
