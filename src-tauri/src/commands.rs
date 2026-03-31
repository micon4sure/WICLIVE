use serde::Serialize;
use std::path::PathBuf;

// ── Install detection ──────────────────────────────────────────────

/// Detect game install path from Windows registry.
#[tauri::command]
pub fn get_install_path() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

        // Standard install
        if let Ok(key) = hklm.open_subkey(r"SOFTWARE\WOW6432Node\Massive Entertainment AB\World in Conflict") {
            if let Ok(path) = key.get_value::<String, _>("InstallPath") {
                if PathBuf::from(&path).join("wic.exe").exists() {
                    return Some(path);
                }
            }
        }

        // GOG variant
        if let Ok(key) = hklm.open_subkey(r"SOFTWARE\WOW6432Node\GOG.com\Games\1438332414") {
            if let Ok(path) = key.get_value::<String, _>("WORKINGDIR") {
                if PathBuf::from(&path).join("wic.exe").exists() {
                    return Some(path);
                }
            }
        }

        None
    }

    #[cfg(not(target_os = "windows"))]
    None
}

fn require_install_path() -> Result<String, String> {
    get_install_path().ok_or_else(|| "Install path not found".into())
}

fn require_exe_path() -> Result<PathBuf, String> {
    let path = require_install_path()?;
    let exe = PathBuf::from(&path).join("wic.exe");
    if !exe.exists() {
        return Err("wic.exe not found".into());
    }
    Ok(exe)
}

// ── Game version (PE header) ───────────────────────────────────────

#[derive(Serialize)]
pub struct VersionInfo {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub build: u16,
}

/// Read game version from wic.exe PE header via Windows API.
#[tauri::command]
pub fn get_game_version() -> Result<VersionInfo, String> {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use windows::core::PCWSTR;
        use windows::Win32::Storage::FileSystem::{
            GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW, VS_FIXEDFILEINFO,
        };

        let exe = require_exe_path()?;
        let path_str = exe.to_string_lossy().to_string();

        unsafe {
            let path_wide: Vec<u16> = OsStr::new(&path_str)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let path_pcw = PCWSTR::from_raw(path_wide.as_ptr());

            let data_len = GetFileVersionInfoSizeW(path_pcw, None);
            if data_len == 0 {
                return Err("Failed to get version info size".into());
            }

            let mut data = vec![0u8; data_len as usize];
            GetFileVersionInfoW(path_pcw, Some(0), data_len, data.as_mut_ptr() as *mut _)
                .map_err(|e| e.to_string())?;

            let mut info_ptr: *mut VS_FIXEDFILEINFO = std::ptr::null_mut();
            let mut info_len: u32 = 0;

            let query_wide: Vec<u16> = OsStr::new(r"\")
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            let ok = VerQueryValueW(
                data.as_ptr() as *const _,
                PCWSTR(query_wide.as_ptr()),
                (&mut info_ptr) as *mut _ as *mut *mut std::ffi::c_void,
                &mut info_len,
            );

            if !ok.as_bool() || info_ptr.is_null() {
                return Err("Failed to query version value".into());
            }

            let ffi = info_ptr.read_unaligned();
            Ok(VersionInfo {
                major: ((ffi.dwFileVersionMS >> 16) & 0xFFFF) as u16,
                minor: (ffi.dwFileVersionMS & 0xFFFF) as u16,
                patch: ((ffi.dwFileVersionLS >> 16) & 0xFFFF) as u16,
                build: (ffi.dwFileVersionLS & 0xFFFF) as u16,
            })
        }
    }

    #[cfg(not(target_os = "windows"))]
    Err("Not supported on this platform".into())
}

/// Read game version string from registry.
#[tauri::command]
pub fn get_game_version_registry() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(key) = hklm.open_subkey(r"SOFTWARE\WOW6432Node\Massive Entertainment AB\World in Conflict") {
            if let Ok(version) = key.get_value::<String, _>("Version") {
                return Some(version);
            }
        }
        None
    }

    #[cfg(not(target_os = "windows"))]
    None
}

// ── LAA flag ───────────────────────────────────────────────────────

/// Check if wic.exe has the Large Address Aware flag set.
#[tauri::command]
pub fn get_laa_flag() -> Result<bool, String> {
    let exe = require_exe_path()?;
    check_laa(exe.to_str().unwrap())
}

/// Set the LAA flag on wic.exe.
#[tauri::command]
pub fn set_laa_flag() -> Result<bool, String> {
    let exe = require_exe_path()?;
    apply_laa(exe.to_str().unwrap())
}

fn check_laa(path: &str) -> Result<bool, String> {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};

    let mut file = File::open(path).map_err(|e| e.to_string())?;

    // Read PE offset from DOS header at 0x3C
    let mut buf = [0u8; 4];
    file.seek(SeekFrom::Start(0x3C)).map_err(|e| e.to_string())?;
    file.read_exact(&mut buf).map_err(|e| e.to_string())?;
    let pe_offset = u32::from_le_bytes(buf) as u64;

    // Characteristics at PE + 4 (signature) + 18
    let char_offset = pe_offset + 4 + 18;
    file.seek(SeekFrom::Start(char_offset)).map_err(|e| e.to_string())?;
    let mut char_buf = [0u8; 2];
    file.read_exact(&mut char_buf).map_err(|e| e.to_string())?;
    let characteristics = u16::from_le_bytes(char_buf);

    // IMAGE_FILE_LARGE_ADDRESS_AWARE = 0x0020
    Ok((characteristics & 0x0020) != 0)
}

fn apply_laa(path: &str) -> Result<bool, String> {
    use std::fs::OpenOptions;
    use std::io::{Read, Seek, SeekFrom, Write};

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .map_err(|e| e.to_string())?;

    let mut buf = [0u8; 4];
    file.seek(SeekFrom::Start(0x3C)).map_err(|e| e.to_string())?;
    file.read_exact(&mut buf).map_err(|e| e.to_string())?;
    let pe_offset = u32::from_le_bytes(buf) as u64;

    let char_offset = pe_offset + 4 + 18;
    file.seek(SeekFrom::Start(char_offset)).map_err(|e| e.to_string())?;
    let mut char_buf = [0u8; 2];
    file.read_exact(&mut char_buf).map_err(|e| e.to_string())?;
    let mut characteristics = u16::from_le_bytes(char_buf);

    if (characteristics & 0x0020) != 0 {
        return Ok(true); // Already set
    }

    characteristics |= 0x0020;
    file.seek(SeekFrom::Start(char_offset)).map_err(|e| e.to_string())?;
    file.write_all(&characteristics.to_le_bytes()).map_err(|e| e.to_string())?;

    Ok(true)
}

// ── CD Key ─────────────────────────────────────────────────────────

/// Read CD key from HKCU registry.
#[tauri::command]
pub fn get_cd_key() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let reg_path = r"Software\Massive Entertainment AB\World In Conflict";

        match hkcu.open_subkey(reg_path) {
            Ok(subkey) => match subkey.get_value::<String, _>("CDKEY") {
                Ok(key) => Ok(key),
                Err(_) => Ok(String::new()),
            },
            Err(_) => Ok(String::new()),
        }
    }

    #[cfg(not(target_os = "windows"))]
    Ok(String::new())
}

/// Write CD key to HKCU registry.
#[tauri::command]
pub fn set_cd_key(key: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let reg_path = r"Software\Massive Entertainment AB\World In Conflict";

        let (subkey, _) = hkcu.create_subkey(reg_path)
            .map_err(|e| format!("Failed to open/create registry key: {}", e))?;

        subkey.set_value("CDKEY", &key)
            .map_err(|e| format!("Failed to set CDKEY: {}", e))?;

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    Ok(())
}

// ── VC++ Redistributable ───────────────────────────────────────────

/// Check if VC++ 2015-2022 x86 is installed.
#[tauri::command]
pub fn check_vcredist() -> bool {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        hklm.open_subkey(r"SOFTWARE\WOW6432Node\Microsoft\VisualStudio\14.0\VC\Runtimes\X86").is_ok()
    }

    #[cfg(not(target_os = "windows"))]
    true
}

// ── Hooks ──────────────────────────────────────────────────────────

/// Check if hooks are installed (version file exists in game dir).
#[tauri::command]
pub fn check_hooks() -> Result<bool, String> {
    let install_dir = require_install_path()?;
    let version_file = PathBuf::from(&install_dir).join("wic_cl_hook-version.txt");
    Ok(version_file.exists())
}

/// Get installed hooks version string.
#[tauri::command]
pub fn get_hooks_version() -> Result<String, String> {
    let install_dir = require_install_path()?;
    let version_file = PathBuf::from(&install_dir).join("wic_cl_hook-version.txt");
    std::fs::read_to_string(&version_file).map_err(|e| e.to_string())
}

/// Check if hooks need updating by comparing installed vs latest version.
#[tauri::command]
pub fn needs_hooks_update(latest: String) -> Result<bool, String> {
    let install_dir = require_install_path()?;
    let version_file = PathBuf::from(&install_dir).join("wic_cl_hook-version.txt");
    let installed = std::fs::read_to_string(&version_file).unwrap_or_default();
    Ok(installed.trim() != latest.trim())
}

// ── Soviet Assault detection ───────────────────────────────────────

/// Check if Soviet Assault is installed (assault.dat exists).
#[tauri::command]
pub fn is_soviet_assault() -> bool {
    if let Some(path) = get_install_path() {
        PathBuf::from(path).join("assault.dat").exists()
    } else {
        false
    }
}

// ── Game launch ────────────────────────────────────────────────────

/// Launch wic.exe from install path.
#[tauri::command]
pub fn start_game() -> Result<(), String> {
    let exe = require_exe_path()?;
    std::process::Command::new(&exe)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ── API config ─────────────────────────────────────────────────────

/// Get the configured API URL.
#[tauri::command]
pub fn get_api_url(config: tauri::State<'_, crate::ApiConfig>) -> String {
    config.url.clone()
}
