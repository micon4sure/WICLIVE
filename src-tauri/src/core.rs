use std::path::PathBuf;

// ── Install detection ──────────────────────────────────────────────

/// Detect game install path from GAME_DIR env var or Windows registry.
pub fn get_install_path() -> Option<String> {
    // Check env var first (works on all platforms, set via .env)
    if let Ok(dir) = std::env::var("GAME_DIR") {
        if PathBuf::from(&dir).join("wic.exe").exists() {
            return Some(dir);
        }
    }

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
    }

    None
}

pub fn require_install_path() -> Result<String, String> {
    get_install_path().ok_or_else(|| "Install path not found".into())
}

pub fn require_exe_path() -> Result<PathBuf, String> {
    let path = require_install_path()?;
    let exe = PathBuf::from(&path).join("wic.exe");
    if !exe.exists() {
        return Err("wic.exe not found".into());
    }
    Ok(exe)
}

// ── Game version (PE header) ───────────────────────────────────────

#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub build: u16,
}

impl std::fmt::Display for VersionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}.{}", self.major, self.minor, self.patch, self.build)
    }
}

/// Read game version from a PE executable via Windows API.
pub fn read_pe_version(exe_path: &str) -> Result<VersionInfo, String> {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use windows::core::PCWSTR;
        use windows::Win32::Storage::FileSystem::{
            GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW, VS_FIXEDFILEINFO,
        };

        unsafe {
            let path_wide: Vec<u16> = OsStr::new(exe_path)
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

/// Read game version from registry.
pub fn read_registry_version() -> Option<String> {
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

/// Check if a PE executable has the Large Address Aware flag set.
pub fn check_laa(path: &str) -> Result<bool, String> {
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom};

    let mut file = File::open(path).map_err(|e| e.to_string())?;

    let mut buf = [0u8; 4];
    file.seek(SeekFrom::Start(0x3C)).map_err(|e| e.to_string())?;
    file.read_exact(&mut buf).map_err(|e| e.to_string())?;
    let pe_offset = u32::from_le_bytes(buf) as u64;

    let char_offset = pe_offset + 4 + 18;
    file.seek(SeekFrom::Start(char_offset)).map_err(|e| e.to_string())?;
    let mut char_buf = [0u8; 2];
    file.read_exact(&mut char_buf).map_err(|e| e.to_string())?;
    let characteristics = u16::from_le_bytes(char_buf);

    Ok((characteristics & 0x0020) != 0)
}

/// Set the LAA flag on a PE executable.
pub fn apply_laa(path: &str) -> Result<bool, String> {
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

/// Clear the LAA flag on a PE executable.
pub fn unset_laa(path: &str) -> Result<bool, String> {
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

    if (characteristics & 0x0020) == 0 {
        return Ok(true); // Already cleared
    }

    characteristics &= !0x0020;
    file.seek(SeekFrom::Start(char_offset)).map_err(|e| e.to_string())?;
    file.write_all(&characteristics.to_le_bytes()).map_err(|e| e.to_string())?;

    Ok(true)
}

// ── CD Key ─────────────────────────────────────────────────────────

/// Read CD key from HKCU registry.
pub fn read_cd_key() -> Result<String, String> {
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
pub fn write_cd_key(key: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let reg_path = r"Software\Massive Entertainment AB\World In Conflict";

        let (subkey, _) = hkcu.create_subkey(reg_path)
            .map_err(|e| format!("Failed to open/create registry key: {}", e))?;

        subkey.set_value("CDKEY", &key.to_string())
            .map_err(|e| format!("Failed to set CDKEY: {}", e))?;

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    Ok(())
}

// ── VC++ Redistributable ───────────────────────────────────────────

/// Check if VC++ 2015-2022 x86 is installed.
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

// ── Hooks / Proxy ──────────────────────────────────────────────────

/// Check if proxy is installed (version file exists in game dir).
pub fn check_proxy(install_dir: &str) -> bool {
    PathBuf::from(install_dir).join("wic_cl_hook-version.txt").exists()
}

/// Get installed proxy version string.
pub fn read_proxy_version(install_dir: &str) -> Result<String, String> {
    let version_file = PathBuf::from(install_dir).join("wic_cl_hook-version.txt");
    std::fs::read_to_string(&version_file).map_err(|e| e.to_string())
}

/// Check if proxy needs updating by comparing installed vs latest version.
pub fn needs_proxy_update(install_dir: &str, latest: &str) -> Result<bool, String> {
    let version_file = PathBuf::from(install_dir).join("wic_cl_hook-version.txt");
    let installed = std::fs::read_to_string(&version_file).unwrap_or_default();
    Ok(installed.trim() != latest.trim())
}

/// Remove proxy files from install directory.
pub fn remove_proxy(install_dir: &str) -> Result<(), String> {
    let dir = PathBuf::from(install_dir);
    for name in &["wic_cl_hook.dll", "wic_ds_hook.dll", "wic_cl_hook-version.txt"] {
        let path = dir.join(name);
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| format!("Failed to remove {}: {}", name, e))?;
        }
    }
    Ok(())
}

// ── Soviet Assault detection ───────────────────────────────────────

/// Check if Soviet Assault is installed (assault.dat exists).
pub fn is_soviet_assault(install_dir: &str) -> bool {
    PathBuf::from(install_dir).join("assault.dat").exists()
}

// ── Reset exe to variant ───────────────────────────────────────────

/// List available wic exe variants in a directory.
pub fn list_variants(install_dir: &str) -> Vec<String> {
    let dir = PathBuf::from(install_dir);
    let mut variants = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("wic.") && name.ends_with(".exe") && name != "wic.exe" {
                variants.push(name);
            }
        }
    }
    variants.sort();
    variants
}

/// Reset wic.exe by copying a variant over it.
pub fn reset_exe(install_dir: &str, variant: &str) -> Result<(), String> {
    let dir = PathBuf::from(install_dir);
    let source = dir.join(variant);
    let target = dir.join("wic.exe");

    if !source.exists() {
        return Err(format!("{} not found", variant));
    }

    std::fs::copy(&source, &target)
        .map_err(|e| format!("Failed to copy {} -> wic.exe: {}", variant, e))?;

    Ok(())
}

// ── Patch download & install ───────────────────────────────────────

/// Determine which patch is needed based on current game version.
/// Returns None if already up to date.
pub fn needed_patch(ver: &VersionInfo) -> Option<&'static str> {
    if ver.major == 1 && ver.minor == 0 {
        if ver.patch == 0 {
            Some("patch-full.zip")     // 1.0.0.x → needs full patch
        } else if ver.patch == 1 && ver.build == 0 {
            Some("patch-p11.zip")      // 1.0.1.0 → needs p11 only
        } else {
            None                        // 1.0.1.1+ → up to date
        }
    } else {
        None
    }
}

/// Download a file from URL to a local path, calling progress_fn with (downloaded, total).
pub async fn download_file<F>(
    url: &str,
    dest: &std::path::Path,
    progress_fn: F,
) -> Result<(), String>
where
    F: Fn(u64, u64),
{
    use futures_util::StreamExt;

    let client = reqwest::Client::new();

    // HEAD first to get Content-Length (GET may use chunked encoding)
    let head = client.head(url).send().await.map_err(|e| format!("HEAD failed: {}", e))?;
    let total = head.content_length()
        .filter(|&n| n > 0)
        .or_else(|| head.headers().get("content-length")?.to_str().ok()?.parse().ok())
        .unwrap_or(0);

    let response = client.get(url).send().await.map_err(|e| format!("Request failed: {}", e))?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let mut downloaded: u64 = 0;
    let mut file = std::fs::File::create(dest)
        .map_err(|e| format!("Failed to create file: {}", e))?;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;
        std::io::Write::write_all(&mut file, &chunk)
            .map_err(|e| format!("Write error: {}", e))?;
        downloaded += chunk.len() as u64;
        progress_fn(downloaded, total);
    }

    Ok(())
}

/// Extract a zip file into the target directory, overwriting existing files.
pub fn extract_zip<F>(zip_path: &std::path::Path, dest_dir: &str, progress_fn: F) -> Result<u64, String>
where
    F: Fn(u64, u64, &str),
{
    let file = std::fs::File::open(zip_path)
        .map_err(|e| format!("Failed to open zip: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Invalid zip: {}", e))?;

    let total = archive.len() as u64;
    let mut extracted: u64 = 0;
    let dest = PathBuf::from(dest_dir);

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| format!("Zip entry error: {}", e))?;

        let name = entry.name().to_string();
        if name.ends_with('/') {
            let dir_path = dest.join(&name);
            std::fs::create_dir_all(&dir_path)
                .map_err(|e| format!("Failed to create dir {}: {}", name, e))?;
            continue;
        }

        let out_path = dest.join(&name);
        if let Some(parent) = out_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create dir: {}", e))?;
            }
        }

        let mut out_file = std::fs::File::create(&out_path)
            .map_err(|e| format!("Failed to create {}: {}", name, e))?;
        std::io::copy(&mut entry, &mut out_file)
            .map_err(|e| format!("Failed to extract {}: {}", name, e))?;
        extracted += 1;
        progress_fn(extracted, total, &name);
    }

    Ok(extracted)
}

/// Set game version in registry after patching.
pub fn set_registry_version(version: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let (key, _) = hklm.create_subkey(r"SOFTWARE\WOW6432Node\Massive Entertainment AB\World in Conflict")
            .map_err(|e| format!("Failed to open registry key: {}", e))?;
        key.set_value("Version", &version.to_string())
            .map_err(|e| format!("Failed to set version: {}", e))?;
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    Ok(())
}

// ── Game launch ────────────────────────────────────────────────────

/// Launch wic.exe from given path.
pub fn launch_game(exe_path: &str) -> Result<(), String> {
    std::process::Command::new(exe_path)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}
