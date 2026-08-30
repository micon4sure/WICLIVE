use std::path::PathBuf;

const GAME_EXE: &str = "wic_online.exe";
const BASE_EXE: &str = "wic.exe";
const LAUNCHABLE_EXES: &[&str] = &[BASE_EXE, GAME_EXE];

// ── Install detection ──────────────────────────────────────────────

/// Detect game install path from Windows registry.
pub fn get_install_path() -> Option<String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    // Standard install
    if let Ok(key) = hklm.open_subkey(r"SOFTWARE\WOW6432Node\Massive Entertainment AB\World in Conflict") {
        if let Ok(path) = key.get_value::<String, _>("InstallPath") {
            if PathBuf::from(&path).join(GAME_EXE).exists() {
                return Some(path);
            }
        }
    }

    // GOG variant
    if let Ok(key) = hklm.open_subkey(r"SOFTWARE\WOW6432Node\GOG.com\Games\1438332414") {
        if let Ok(path) = key.get_value::<String, _>("WORKINGDIR") {
            if PathBuf::from(&path).join(GAME_EXE).exists() {
                return Some(path);
            }
        }
    }

    None
}

/// Check if the registry has an install path entry (regardless of whether files exist).
pub fn has_registry_install_path() -> bool {
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    if let Ok(key) = hklm.open_subkey(r"SOFTWARE\WOW6432Node\Massive Entertainment AB\World in Conflict") {
        if key.get_value::<String, _>("InstallPath").is_ok() {
            return true;
        }
    }

    if let Ok(key) = hklm.open_subkey(r"SOFTWARE\WOW6432Node\GOG.com\Games\1438332414") {
        if key.get_value::<String, _>("WORKINGDIR").is_ok() {
            return true;
        }
    }

    false
}

pub fn require_install_path() -> Result<String, String> {
    get_install_path().ok_or_else(|| "Install path not found".into())
}

/// Remove the game install path from the Windows registry.
pub fn clear_install_registry() -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    // Standard install
    if let Ok(key) = hklm.open_subkey_with_flags(
        r"SOFTWARE\WOW6432Node\Massive Entertainment AB\World in Conflict",
        KEY_SET_VALUE,
    ) {
        let _ = key.delete_value("InstallPath");
    }

    // GOG variant
    if let Ok(key) = hklm.open_subkey_with_flags(
        r"SOFTWARE\WOW6432Node\GOG.com\Games\1438332414",
        KEY_SET_VALUE,
    ) {
        let _ = key.delete_value("WORKINGDIR");
    }

    Ok(())
}

pub fn require_exe_path() -> Result<PathBuf, String> {
    let path = require_install_path()?;
    let exe = PathBuf::from(&path).join(GAME_EXE);
    if !exe.exists() {
        return Err(format!("{} not found", GAME_EXE));
    }
    Ok(exe)
}

/// Paths of all launchable wic executables that exist in the install dir.
pub fn existing_launchable_exes(install_dir: &str) -> Vec<PathBuf> {
    let dir = PathBuf::from(install_dir);
    LAUNCHABLE_EXES
        .iter()
        .map(|name| dir.join(name))
        .filter(|p| p.exists())
        .collect()
}

/// Resolve which exe to launch. Prefers wic.exe, falls back to wic_online.exe.
pub fn resolve_launch_exe(install_dir: &str) -> Result<PathBuf, String> {
    let dir = PathBuf::from(install_dir);
    for name in LAUNCHABLE_EXES {
        let p = dir.join(name);
        if p.exists() {
            return Ok(p);
        }
    }
    Err("No wic.exe or wic_online.exe found".into())
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

/// Read game version from registry.
pub fn read_registry_version() -> Option<String> {
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

/// Check LAA across all existing launchable exes. Returns Ok(true) only if every
/// existing exe has the flag set. Errors if none exist.
pub fn check_laa_all(install_dir: &str) -> Result<bool, String> {
    let exes = existing_launchable_exes(install_dir);
    if exes.is_empty() {
        return Err("No wic.exe or wic_online.exe found".into());
    }
    for exe in &exes {
        if !check_laa(exe.to_str().unwrap())? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Apply LAA to all existing launchable exes. Errors if none exist.
pub fn apply_laa_all(install_dir: &str) -> Result<bool, String> {
    let exes = existing_launchable_exes(install_dir);
    if exes.is_empty() {
        return Err("No wic.exe or wic_online.exe found".into());
    }
    for exe in &exes {
        apply_laa(exe.to_str().unwrap())?;
    }
    Ok(true)
}

/// Clear LAA on all existing launchable exes. Errors if none exist.
pub fn unset_laa_all(install_dir: &str) -> Result<bool, String> {
    let exes = existing_launchable_exes(install_dir);
    if exes.is_empty() {
        return Err("No wic.exe or wic_online.exe found".into());
    }
    for exe in &exes {
        unset_laa(exe.to_str().unwrap())?;
    }
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

// ── Welcome launcher flag ─────────────────────────────────────────

const LAUNCHER_SHOW_OPCODE: u8 = 0x75; // JNE
const LAUNCHER_SKIP_OPCODE: u8 = 0xEB; // JMP
const LAUNCHER_SIGNATURE_LEN: usize = 42;

#[derive(Debug)]
struct LauncherFlagSite {
    path: PathBuf,
    offset: u64,
    opcode: u8,
}

/// Find the conditional branch that opens the post-1.10 welcome launcher.
///
/// The surrounding instructions are shared by `wic.exe` and
/// `wic_online.exe`, while their file offsets and absolute addresses differ.
/// Matching the complete instruction sequence keeps this safe across the
/// known 1.10 and 1.11 layouts without writing to an unrecognised binary.
fn find_launcher_flag_offset(bytes: &[u8]) -> Result<usize, String> {
    if bytes.len() < LAUNCHER_SIGNATURE_LEN {
        return Err("welcome launcher flag signature not found".into());
    }

    let mut found = None;
    for start in 0..=bytes.len() - LAUNCHER_SIGNATURE_LEN {
        let opcode = bytes[start + 6];
        let global = &bytes[start + 2..start + 6];
        let matches = bytes[start] == 0x39
            && bytes[start + 1] == 0x1D
            && matches!(opcode, LAUNCHER_SHOW_OPCODE | LAUNCHER_SKIP_OPCODE)
            && bytes[start + 7] == 0x10
            && bytes[start + 8] == 0xE8
            && bytes[start + 13..start + 18] == [0x83, 0xF8, 0x01, 0x75, 0x06]
            && bytes[start + 18] == 0x89
            && bytes[start + 19] == 0x3D
            && &bytes[start + 20..start + 24] == global
            && bytes[start + 24] == 0xE8
            && bytes[start + 29] == 0xE8
            && bytes[start + 34] == 0x39
            && bytes[start + 35] == 0x1D
            && &bytes[start + 36..start + 40] == global
            && bytes[start + 40] == 0x75
            && bytes[start + 41] == 0x39;

        if matches {
            if found.is_some() {
                return Err("multiple welcome launcher flag signatures found".into());
            }
            found = Some(start + 6);
        }
    }

    found.ok_or_else(|| "welcome launcher flag signature not found".into())
}

fn inspect_launcher_flag(path: &std::path::Path) -> Result<LauncherFlagSite, String> {
    let bytes =
        std::fs::read(path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    let offset =
        find_launcher_flag_offset(&bytes).map_err(|e| format!("{}: {}", path.display(), e))?;

    Ok(LauncherFlagSite {
        path: path.to_path_buf(),
        offset: offset as u64,
        opcode: bytes[offset],
    })
}

fn write_launcher_opcode(
    site: &LauncherFlagSite,
    expected: u8,
    replacement: u8,
) -> Result<(), String> {
    use std::fs::OpenOptions;
    use std::io::{Read, Seek, SeekFrom, Write};

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&site.path)
        .map_err(|e| format!("Failed to open {}: {}", site.path.display(), e))?;

    file.seek(SeekFrom::Start(site.offset))
        .map_err(|e| format!("Failed to seek {}: {}", site.path.display(), e))?;
    let mut current = [0u8; 1];
    file.read_exact(&mut current)
        .map_err(|e| format!("Failed to verify {}: {}", site.path.display(), e))?;
    if current[0] != expected {
        return Err(format!(
            "{} changed while updating its welcome launcher flag",
            site.path.display()
        ));
    }

    file.seek(SeekFrom::Start(site.offset))
        .map_err(|e| format!("Failed to seek {}: {}", site.path.display(), e))?;
    file.write_all(&[replacement])
        .map_err(|e| format!("Failed to update {}: {}", site.path.display(), e))?;
    file.sync_data()
        .map_err(|e| format!("Failed to flush {}: {}", site.path.display(), e))?;

    file.seek(SeekFrom::Start(site.offset))
        .map_err(|e| format!("Failed to seek {}: {}", site.path.display(), e))?;
    file.read_exact(&mut current)
        .map_err(|e| format!("Failed to verify {}: {}", site.path.display(), e))?;
    if current[0] != replacement {
        return Err(format!(
            "Failed to verify the welcome launcher flag in {}",
            site.path.display()
        ));
    }

    Ok(())
}

/// Check every launchable executable in the install. The setting is considered
/// enabled only when all existing executables skip the welcome launcher.
pub fn check_skip_launcher(install_dir: &str) -> Result<bool, String> {
    let exes = existing_launchable_exes(install_dir);
    if exes.is_empty() {
        return Err("No wic.exe or wic_online.exe found".into());
    }

    let sites = exes
        .iter()
        .map(|path| inspect_launcher_flag(path))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(sites
        .iter()
        .all(|site| site.opcode == LAUNCHER_SKIP_OPCODE))
}

/// Set the launcher behaviour on every executable that exists in the install.
/// All targets are validated before the first byte is changed. If a later write
/// fails, earlier changes are rolled back to keep the executables consistent.
pub fn set_skip_launcher(install_dir: &str, enabled: bool) -> Result<bool, String> {
    let exes = existing_launchable_exes(install_dir);
    if exes.is_empty() {
        return Err("No wic.exe or wic_online.exe found".into());
    }

    let sites = exes
        .iter()
        .map(|path| inspect_launcher_flag(path))
        .collect::<Result<Vec<_>, _>>()?;
    let replacement = if enabled {
        LAUNCHER_SKIP_OPCODE
    } else {
        LAUNCHER_SHOW_OPCODE
    };

    let mut changed: Vec<&LauncherFlagSite> = Vec::new();
    for site in &sites {
        if site.opcode == replacement {
            continue;
        }
        if let Err(error) = write_launcher_opcode(site, site.opcode, replacement) {
            let mut rollback_errors = Vec::new();
            for previous in changed.iter().rev() {
                if let Err(rollback_error) =
                    write_launcher_opcode(previous, replacement, previous.opcode)
                {
                    rollback_errors.push(rollback_error);
                }
            }
            if rollback_errors.is_empty() {
                return Err(error);
            }
            return Err(format!(
                "{}; rollback also failed: {}",
                error,
                rollback_errors.join("; ")
            ));
        }
        changed.push(site);
    }

    check_skip_launcher(install_dir)
}

// ── CD Key ─────────────────────────────────────────────────────────

/// Read CD key from HKCU registry.
pub fn read_cd_key() -> Result<String, String> {
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

/// Write CD key to HKCU registry.
pub fn write_cd_key(key: &str) -> Result<(), String> {
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

// ── VC++ Redistributable ───────────────────────────────────────────

/// Check if VC++ 2015-2022 x86 is installed.
pub fn check_vcredist() -> bool {
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    hklm.open_subkey(r"SOFTWARE\WOW6432Node\Microsoft\VisualStudio\14.0\VC\Runtimes\X86").is_ok()
}

// ── DirectX 9 Runtime ──────────────────────────────────────────────

/// Check if the DirectX 9 June 2010 runtime is installed (d3dx9_43.dll
/// present in SysWOW64 or System32). Vanilla Windows does not ship this.
pub fn check_dx9() -> bool {
    let sysroot = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    let base = PathBuf::from(&sysroot);
    base.join("SysWOW64").join("d3dx9_43.dll").exists()
        || base.join("System32").join("d3dx9_43.dll").exists()
}

/// Launch the DirectX web installer elevated and wait for it to exit.
#[cfg(windows)]
pub fn run_dx9_installer(installer_path: &std::path::Path) -> Result<(), String> {
    use std::mem;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    use windows::Win32::System::Threading::WaitForSingleObject;
    use windows::core::PCWSTR;

    let file_wide: Vec<u16> = installer_path
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let verb: Vec<u16> = "runas\0".encode_utf16().collect();
    let params: Vec<u16> = "/Q\0".encode_utf16().collect();

    unsafe {
        let mut info: SHELLEXECUTEINFOW = mem::zeroed();
        info.cbSize = mem::size_of::<SHELLEXECUTEINFOW>() as u32;
        info.fMask = SEE_MASK_NOCLOSEPROCESS;
        info.lpVerb = PCWSTR(verb.as_ptr());
        info.lpFile = PCWSTR(file_wide.as_ptr());
        info.lpParameters = PCWSTR(params.as_ptr());
        info.nShow = SW_SHOWNORMAL.0;

        ShellExecuteExW(&mut info)
            .map_err(|e| format!("Failed to launch DirectX installer: {}", e))?;

        if !info.hProcess.is_invalid() {
            WaitForSingleObject(info.hProcess, u32::MAX);
            let _ = CloseHandle(info.hProcess);
        }
    }

    Ok(())
}

#[cfg(not(windows))]
pub fn run_dx9_installer(_installer_path: &std::path::Path) -> Result<(), String> {
    Err("DirectX install is only supported on Windows".into())
}

// ── Hooks / Proxy ──────────────────────────────────────────────────

const PROXY_VERSION_FILE: &str = "wicgate-proxy-version.txt";
const COMPAT_PROXY_VERSION_FILE: &str = "wicgate-compat-proxy-version.txt";

fn proxy_version_file(compatibility: bool) -> &'static str {
    if compatibility {
        COMPAT_PROXY_VERSION_FILE
    } else {
        PROXY_VERSION_FILE
    }
}

/// Check if the selected proxy is installed (its version file exists in game dir).
pub fn check_proxy(install_dir: &str, compatibility: bool) -> bool {
    PathBuf::from(install_dir)
        .join(proxy_version_file(compatibility))
        .exists()
}

/// Return whether the compatibility proxy is the installed proxy variant.
pub fn is_compatibility_proxy(install_dir: &str) -> bool {
    check_proxy(install_dir, true)
}

/// Get the selected installed proxy's version string.
pub fn read_proxy_version(install_dir: &str, compatibility: bool) -> Result<String, String> {
    let version_file = PathBuf::from(install_dir).join(proxy_version_file(compatibility));
    std::fs::read_to_string(&version_file).map_err(|e| e.to_string())
}

/// Check if proxy needs updating by comparing installed vs latest version.
pub fn needs_proxy_update(
    install_dir: &str,
    latest: &str,
    compatibility: bool,
) -> Result<bool, String> {
    let version_file = PathBuf::from(install_dir).join(proxy_version_file(compatibility));
    let installed = std::fs::read_to_string(&version_file).unwrap_or_default();
    Ok(installed.trim() != latest.trim())
}

/// Remove the other proxy's version marker after switching variants.
pub fn remove_other_proxy_marker(
    install_dir: &str,
    compatibility: bool,
) -> Result<(), String> {
    let other = PathBuf::from(install_dir).join(proxy_version_file(!compatibility));
    if other.exists() {
        std::fs::remove_file(&other)
            .map_err(|e| format!("Failed to remove {}: {}", other.display(), e))?;
    }
    Ok(())
}

/// Remove proxy files from install directory.
pub fn remove_proxy(install_dir: &str) -> Result<(), String> {
    let dir = PathBuf::from(install_dir);
    for name in &["dbghelp.dll", PROXY_VERSION_FILE, COMPAT_PROXY_VERSION_FILE] {
        let path = dir.join(name);
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| format!("Failed to remove {}: {}", name, e))?;
        }
    }
    Ok(())
}

// ── Legacy Proxy (PBE) ─────────────────────────────────────────────

/// Check if legacy proxy is installed (dbghelp-pbe.dll exists in game dir).
pub fn check_legacy_proxy(install_dir: &str) -> bool {
    PathBuf::from(install_dir).join("dbghelp-pbe.dll").exists()
}

/// Check if legacy proxy is currently active.
pub fn is_legacy_proxy_active(install_dir: &str) -> bool {
    let mode_file = PathBuf::from(install_dir).join("wiclive-mode.txt");
    std::fs::read_to_string(&mode_file)
        .map(|s| s.trim() == "pbe")
        .unwrap_or(false)
}

/// Activate legacy proxy: copy dbghelp-pbe.dll → dbghelp.dll, write mode.
pub fn activate_legacy_proxy(install_dir: &str) -> Result<(), String> {
    let dir = PathBuf::from(install_dir);
    let pbe = dir.join("dbghelp-pbe.dll");
    let active = dir.join("dbghelp.dll");
    let mode = dir.join("wiclive-mode.txt");

    if !pbe.exists() {
        return Err("Legacy proxy not installed".into());
    }

    // Save current as live backup if not already saved
    let live = dir.join("dbghelp-live.dll");
    if active.exists() && !live.exists() {
        std::fs::copy(&active, &live)
            .map_err(|e| format!("Failed to backup live proxy: {}", e))?;
    }

    std::fs::copy(&pbe, &active)
        .map_err(|e| format!("Failed to activate legacy proxy: {}", e))?;
    std::fs::write(&mode, "pbe")
        .map_err(|e| format!("Failed to write mode: {}", e))?;
    Ok(())
}

/// Deactivate legacy proxy: copy dbghelp-live.dll → dbghelp.dll, write mode.
pub fn deactivate_legacy_proxy(install_dir: &str) -> Result<(), String> {
    let dir = PathBuf::from(install_dir);
    let live = dir.join("dbghelp-live.dll");
    let active = dir.join("dbghelp.dll");
    let mode = dir.join("wiclive-mode.txt");

    if !live.exists() {
        return Err("Live proxy backup not found".into());
    }

    std::fs::copy(&live, &active)
        .map_err(|e| format!("Failed to restore live proxy: {}", e))?;
    std::fs::write(&mode, "live")
        .map_err(|e| format!("Failed to write mode: {}", e))?;
    Ok(())
}

// ── Soviet Assault detection ───────────────────────────────────────

pub const PRODUCT_ID_WIC07_STANDARD_KEY: u8 = 1;
pub const PRODUCT_ID_WIC08_STANDARD_KEY: u8 = 3;

/// Check if Soviet Assault is installed (assault.dat exists).
pub fn is_soviet_assault(install_dir: &str) -> bool {
    PathBuf::from(install_dir).join("assault.dat").exists()
}

/// Return the Massgate CD-key product matching the installed game edition.
pub fn installed_product_id() -> Result<u8, String> {
    let install_dir = require_install_path()?;
    Ok(if is_soviet_assault(&install_dir) {
        PRODUCT_ID_WIC08_STANDARD_KEY
    } else {
        PRODUCT_ID_WIC07_STANDARD_KEY
    })
}

// ── Reset exe to variant ───────────────────────────────────────────

/// List available wic exe variants in a directory.
pub fn list_variants(install_dir: &str) -> Vec<String> {
    let dir = PathBuf::from(install_dir);
    let mut variants = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("wic") && name.ends_with(".exe") && name != GAME_EXE {
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
    let target = dir.join(GAME_EXE);

    if !source.exists() {
        return Err(format!("{} not found", variant));
    }

    std::fs::copy(&source, &target)
        .map_err(|e| format!("Failed to copy {} -> {}: {}", variant, GAME_EXE, e))?;

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

    let client = reqwest::Client::builder()
        .no_proxy()
        .build()
        .map_err(|e| format!("Client build failed: {}", e))?;

    // HEAD first to get Content-Length (GET may use chunked encoding)
    let head = client.head(url).send().await.map_err(|e| format!("HEAD failed: {}", e))?;
    let total = head.content_length()
        .filter(|&n| n > 0)
        .or_else(|| head.headers().get("content-length")?.to_str().ok()?.parse().ok())
        .unwrap_or(0);

    let response = client.get(url)
        .header("Accept-Encoding", "identity")
        .send().await.map_err(|e| format!("Request failed: {}", e))?;
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
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (key, _) = hklm.create_subkey(r"SOFTWARE\WOW6432Node\Massive Entertainment AB\World in Conflict")
        .map_err(|e| format!("Failed to open registry key: {}", e))?;
    key.set_value("Version", &version.to_string())
        .map_err(|e| format!("Failed to set version: {}", e))?;
    Ok(())
}

// ── Install registration ──────────────────────────────────────────

/// Register WiC installation: registry keys, Add/Remove Programs, desktop shortcut.
#[cfg(windows)]
pub fn register_install(install_dir: &str) -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    // Set install path (what get_install_path() reads)
    let (key, _) = hklm.create_subkey(r"SOFTWARE\WOW6432Node\Massive Entertainment AB\World in Conflict")
        .map_err(|e| format!("Failed to create registry key: {}", e))?;
    key.set_value("InstallPath", &install_dir.to_string())
        .map_err(|e| format!("Failed to set InstallPath: {}", e))?;

    // Set version
    key.set_value("Version", &"1.0.1.1".to_string())
        .map_err(|e| format!("Failed to set Version: {}", e))?;

    #[cfg(not(feature = "portable"))]
    {
        // Add/Remove Programs (Uninstall entry)
        let uninstall_key = r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\World in Conflict - Multiplayer";
        let (ukey, _) = hklm.create_subkey(uninstall_key)
            .map_err(|e| format!("Failed to create uninstall key: {}", e))?;
        ukey.set_value("DisplayName", &"World in Conflict - Multiplayer".to_string()).ok();
        ukey.set_value("InstallLocation", &install_dir.to_string()).ok();
        ukey.set_value("DisplayIcon", &format!(r"{}\{},0", install_dir, GAME_EXE)).ok();
        ukey.set_value("Publisher", &"Massive Entertainment".to_string()).ok();
        ukey.set_value("DisplayVersion", &"1.0.1.1".to_string()).ok();
        ukey.set_value("NoModify", &1u32).ok();
        ukey.set_value("NoRepair", &1u32).ok();
        if let Ok(exe) = std::env::current_exe() {
            ukey.set_value("UninstallString", &format!("\"{}\" --uninstall", exe.to_string_lossy())).ok();
        }

        // Desktop shortcut
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            let desktop = PathBuf::from(&userprofile).join("Desktop");
            let shortcut_path = desktop.join("World in Conflict.lnk");
            let exe_path = PathBuf::from(install_dir).join(GAME_EXE);
            let _ = create_shortcut(&shortcut_path, &exe_path);
        }
    }

    Ok(())
}

#[cfg(not(windows))]
pub fn register_install(_install_dir: &str) -> Result<(), String> {
    Ok(())
}

/// Create a Windows .lnk shortcut file.
#[cfg(windows)]
fn create_shortcut(shortcut_path: &std::path::Path, target: &std::path::Path) -> Result<(), String> {
    use std::process::Command;
    // Use PowerShell to create the shortcut
    let ps_script = format!(
        "$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut('{}'); $s.TargetPath = '{}'; $s.WorkingDirectory = '{}'; $s.Save()",
        shortcut_path.to_string_lossy(),
        target.to_string_lossy(),
        target.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
    );
    Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_script])
        .output()
        .map_err(|e| format!("Failed to create shortcut: {}", e))?;
    Ok(())
}

/// Uninstall: delete game files, remove registry keys, remove desktop shortcut.
#[cfg(windows)]
pub fn uninstall_game() -> Result<String, String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

    // Get install path before we delete the key
    let install_path = get_install_path();

    // Remove game registry keys
    clear_install_registry()?;

    // Remove version
    if let Ok(key) = hklm.open_subkey_with_flags(
        r"SOFTWARE\WOW6432Node\Massive Entertainment AB\World in Conflict",
        KEY_SET_VALUE,
    ) {
        let _ = key.delete_value("Version");
    }

    // Remove Add/Remove Programs entry
    let _ = hklm.delete_subkey_all(
        r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\World in Conflict - Multiplayer",
    );

    // Remove desktop shortcut
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let shortcut = PathBuf::from(&userprofile).join("Desktop").join("World in Conflict.lnk");
        let _ = std::fs::remove_file(shortcut);
    }

    // Delete game files
    if let Some(path) = install_path {
        let dir = PathBuf::from(&path);
        if dir.exists() {
            std::fs::remove_dir_all(&dir)
                .map_err(|e| format!("Failed to delete {}: {}", path, e))?;
            return Ok(format!("World in Conflict uninstalled from {}", path));
        }
    }

    Ok("World in Conflict uninstalled (registry cleaned).".into())
}

#[cfg(not(windows))]
pub fn uninstall_game() -> Result<String, String> {
    Ok("Uninstall is only supported on Windows.".into())
}

// ── Maps ──────────────────────────────────────────────────────────

/// Get the WiC base directory (Documents/World in Conflict).
/// Tries standard user profile path first, then OneDrive path.
pub fn get_base_directory() -> Result<PathBuf, String> {
    // USERPROFILE\Documents\World in Conflict
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let path = PathBuf::from(&userprofile).join(r"Documents\World in Conflict");
        if path.exists() {
            return Ok(path);
        }
    }

    // OneDrive\Documents\World in Conflict
    if let Ok(onedrive) = std::env::var("OneDrive") {
        let path = PathBuf::from(&onedrive).join(r"Documents\World in Conflict");
        if path.exists() {
            return Ok(path);
        }
    }

    Err("Base directory not found in standard or OneDrive locations.".into())
}

/// Get the maps directory (base/Downloaded/maps). Creates it if missing.
/// MAPS_DIR env var overrides for development.
pub fn get_maps_dir() -> Result<PathBuf, String> {
    let maps = if let Ok(dir) = std::env::var("MAPS_DIR") {
        PathBuf::from(dir)
    } else {
        let base = get_base_directory()?;
        base.join("Downloaded").join("maps")
    };
    if !maps.exists() {
        std::fs::create_dir_all(&maps).map_err(|e| e.to_string())?;
    }
    Ok(maps)
}

/// List all .sdf files in the maps directory.
pub fn list_map_files() -> Result<Vec<String>, String> {
    let dir = get_maps_dir()?;
    if !dir.exists() { return Ok(vec![]); }

    let mut result = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("sdf") {
            if let Some(name) = path.file_name().and_then(|f| f.to_str()) {
                result.push(name.to_string());
            }
        }
    }
    Ok(result)
}

/// Compute MD5 hash of a map file (uppercase hex).
pub fn get_map_hash(filename: &str) -> Result<String, String> {
    let path = get_maps_dir()?.join(filename);
    if !path.exists() {
        return Err(format!("Map file not found: {}", filename));
    }
    md5_file(&path)
}

fn md5_file(path: &std::path::Path) -> Result<String, String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = md5::Context::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 { break; }
        hasher.consume(&buf[..n]);
    }
    Ok(format!("{:X}", hasher.compute()))
}

// ── Autoexec config ───────────────────────────────────────────────

const AUTOEXEC_FILE: &str = "wicautoexec.txt";

const LIVE_START: &str = "// LIVE START";
const LIVE_END: &str = "// LIVE END";
const COMP_START: &str = "// COMPETITIVE START";
const COMP_END: &str = "// COMPETITIVE END";

const LIVE_BLOCK: &str = "\
// LIVE START\r\n\
CameraFreedom 1\r\n\
CameraMaxHeight 1500\r\n\
\r\n\
bind f1 AerialRecon\r\n\
alias AerialRecon tactical_aid; select_cat_1; shortcut_1\r\n\
\r\n\
bind f2 Air2Air\r\n\
alias Air2Air tactical_aid; select_cat_2; shortcut_4; shortcut_4; shortcut_4\r\n\
\r\n\
bind f3 Tankbuster\r\n\
alias Tankbuster tactical_aid; select_cat_2; shortcut_2; shortcut_2; shortcut_2\r\n\
\r\n\
bind f4 Larty\r\n\
alias Larty tactical_aid; select_cat_3; shortcut_1; shortcut_1; shortcut_1\r\n\
\r\n\
bind f5 Harty\r\n\
alias Harty tactical_aid; select_cat_3; shortcut_3; shortcut_3; shortcut_3\r\n\
\r\n\
bind 7 Jeepdrops\r\n\
alias Jeepdrops tactical_aid; select_cat_1; shortcut_3; shortcut_3; shortcut_3\r\n\
\r\n\
bind 8 Tankdrops\r\n\
alias Tankdrops tactical_aid; select_cat_1; shortcut_4; shortcut_4; shortcut_4\r\n\
\r\n\
bind 9 Airbornes\r\n\
alias Airbornes tactical_aid; select_cat_1; shortcut_2; shortcut_2; shortcut_2\r\n\
\r\n\
bind 0 Cluster\r\n\
alias Cluster tactical_aid; select_cat_3; shortcut_2; shortcut_2; shortcut_2\r\n\
// LIVE END";

const COMP_BLOCK: &str = "\
// COMPETITIVE START\r\n\
SetFogDistances 1 1 1 1\r\n\
Ex3DRenderClouds 0\r\n\
// COMPETITIVE END";

fn autoexec_path() -> Result<PathBuf, String> {
    let base = get_base_directory()?;
    Ok(base.join(AUTOEXEC_FILE))
}

fn read_autoexec() -> Result<String, String> {
    let path = autoexec_path()?;
    if !path.exists() {
        return Ok(String::new());
    }
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

fn write_autoexec(contents: &str) -> Result<(), String> {
    let path = autoexec_path()?;
    std::fs::write(&path, contents).map_err(|e| e.to_string())
}

fn has_block(contents: &str, start_marker: &str, end_marker: &str) -> bool {
    contents.contains(start_marker) && contents.contains(end_marker)
}

fn remove_block(contents: &str, start_marker: &str, end_marker: &str) -> String {
    let Some(start) = contents.find(start_marker) else {
        return contents.to_string();
    };
    let Some(end_rel) = contents[start..].find(end_marker) else {
        return contents.to_string();
    };
    let end = start + end_rel + end_marker.len();

    // Consume ONE adjacent newline: prefer trailing, fall back to leading
    let after = &contents[end..];
    let (trim_after, trim_before) = if after.starts_with("\r\n") {
        (2, 0)
    } else if after.starts_with('\n') {
        (1, 0)
    } else {
        let before = &contents[..start];
        if before.ends_with("\r\n") { (0, 2) }
        else if before.ends_with('\n') { (0, 1) }
        else { (0, 0) }
    };

    let mut result = contents[..start - trim_before].to_string();
    result.push_str(&contents[end + trim_after..]);
    result
}

pub fn get_autoexec_state() -> Result<(bool, bool), String> {
    let contents = read_autoexec()?;
    let live = has_block(&contents, LIVE_START, LIVE_END);
    let comp = has_block(&contents, COMP_START, COMP_END);
    Ok((live, comp))
}

pub fn set_live_settings(enabled: bool) -> Result<(), String> {
    let mut contents = read_autoexec()?;

    if enabled {
        if has_block(&contents, LIVE_START, LIVE_END) {
            return Ok(());
        }
        if !contents.is_empty() && !contents.ends_with('\n') && !contents.ends_with("\r\n") {
            contents.push_str("\r\n");
        }
        contents.push_str(LIVE_BLOCK);
        contents.push_str("\r\n");
    } else {
        contents = remove_block(&contents, LIVE_START, LIVE_END);
    }

    write_autoexec(&contents)
}

pub fn set_competitive_settings(enabled: bool) -> Result<(), String> {
    let mut contents = read_autoexec()?;

    if enabled {
        if has_block(&contents, COMP_START, COMP_END) {
            return Ok(());
        }
        if !contents.is_empty() && !contents.ends_with('\n') && !contents.ends_with("\r\n") {
            contents.push_str("\r\n");
        }
        contents.push_str(COMP_BLOCK);
        contents.push_str("\r\n");
    } else {
        contents = remove_block(&contents, COMP_START, COMP_END);
    }

    write_autoexec(&contents)
}

// ── WiCGate user settings ─────────────────────────────────────────

const WICGATE_CONFIG: &str = "wicgate.txt";

const WICGATE_CONFIG_DEFAULTS: &str = "\
; WiCGate Client Proxy Configuration\r\n\
; Delete this file to regenerate defaults. Changes require game restart.\r\n\
; The [launcher] section is used by WIC LIVE and ignored by the proxy.\r\n\
\r\n\
[launcher]\r\n\
; Skip publisher logos and the intro movie\r\n\
nointro=0\r\n\
; Open the multiplayer login screen after launch\r\n\
playonline=0\r\n\
\r\n\
[camera_fix]\r\n\
; Prevent camera fly-to when changing drop zone mid-game\r\n\
camera_fix=1\r\n\
\r\n\
[hilite_own]\r\n\
; Highlight your own units with a distinct color\r\n\
; Presets: amber, azure, coral, cyan, gold, lime, magenta, orange,\r\n\
;   pink, silver, white, yellow\r\n\
; Or RRGGBB hex (e.g. CFB408 = amber). Leave empty to disable.\r\n\
hilite_own_color=\r\n\
\r\n\
[ignore_alt_tab]\r\n\
; Prevent game from going idle when alt-tabbed\r\n\
ignore_alt_tab=0\r\n\
\r\n\
[no_cursor_speed]\r\n\
; Disable Windows cursor acceleration in-game\r\n\
no_cursor_speed=0\r\n\
\r\n\
[nuke_warning]\r\n\
; Show HUD notification when enemy nuke is launched\r\n\
nuke_warning=1\r\n";

fn wicgate_config_path() -> Result<PathBuf, String> {
    let base = get_base_directory()?;
    Ok(base.join(WICGATE_CONFIG))
}

fn ensure_wicgate_config() -> Result<PathBuf, String> {
    let path = wicgate_config_path()?;
    if !path.exists() {
        std::fs::write(&path, WICGATE_CONFIG_DEFAULTS).map_err(|e| e.to_string())?;
        return Ok(path);
    }

    let contents = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    if contents
        .bytes()
        .all(|byte| byte == 0 || byte.is_ascii_whitespace())
    {
        std::fs::write(&path, WICGATE_CONFIG_DEFAULTS).map_err(|e| e.to_string())?;
    } else if let Some(updated) = add_missing_launcher_settings(&contents) {
        std::fs::write(&path, updated).map_err(|e| e.to_string())?;
    }
    Ok(path)
}

fn add_missing_launcher_settings(contents: &str) -> Option<String> {
    let missing_nointro = parse_wicgate_value(contents, "nointro").is_none();
    let missing_playonline = parse_wicgate_value(contents, "playonline").is_none();
    if !missing_nointro && !missing_playonline {
        return None;
    }

    let mut result = contents.trim_end_matches(['\r', '\n']).to_string();
    if !result.is_empty() {
        result.push_str("\r\n\r\n");
    }
    result.push_str("[launcher]\r\n");
    result.push_str("; WIC LIVE launch options (ignored by the client proxy)\r\n");
    if missing_nointro {
        result.push_str("nointro=0\r\n");
    }
    if missing_playonline {
        result.push_str("playonline=0\r\n");
    }
    Some(result)
}

fn parse_wicgate_value(contents: &str, key: &str) -> Option<String> {
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(';') || trimmed.starts_with('[') || trimmed.is_empty() {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=') {
            if k.trim() == key {
                return Some(v.trim().to_string());
            }
        }
    }
    None
}

#[derive(serde::Serialize)]
pub struct WicgateSettings {
    pub nointro: bool,
    pub playonline: bool,
    pub camera_fix: bool,
    pub hilite_own_color: String,
    pub ignore_alt_tab: bool,
    pub no_cursor_speed: bool,
    pub nuke_warning: bool,
}

pub fn get_wicgate_settings() -> Result<WicgateSettings, String> {
    let path = ensure_wicgate_config()?;
    let contents = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    Ok(WicgateSettings {
        nointro: parse_wicgate_value(&contents, "nointro")
            .map(|v| v == "1")
            .unwrap_or(false),
        playonline: parse_wicgate_value(&contents, "playonline")
            .map(|v| v == "1")
            .unwrap_or(false),
        camera_fix: parse_wicgate_value(&contents, "camera_fix")
            .map(|v| v == "1")
            .unwrap_or(true),
        hilite_own_color: parse_wicgate_value(&contents, "hilite_own_color")
            .unwrap_or_default(),
        ignore_alt_tab: parse_wicgate_value(&contents, "ignore_alt_tab")
            .map(|v| v == "1")
            .unwrap_or(false),
        no_cursor_speed: parse_wicgate_value(&contents, "no_cursor_speed")
            .map(|v| v == "1")
            .unwrap_or(false),
        nuke_warning: parse_wicgate_value(&contents, "nuke_warning")
            .map(|v| v == "1")
            .unwrap_or(true),
    })
}

pub fn set_wicgate_setting(key: &str, value: &str) -> Result<(), String> {
    let path = ensure_wicgate_config()?;
    let contents = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    let mut found = false;
    let mut new_lines: Vec<String> = Vec::new();

    for line in contents.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with(';') && !trimmed.starts_with('[') && !trimmed.is_empty() {
            if let Some((k, _)) = trimmed.split_once('=') {
                if k.trim() == key {
                    new_lines.push(format!("{}={}", key, value));
                    found = true;
                    continue;
                }
            }
        }
        new_lines.push(line.to_string());
    }

    if !found {
        return Err(format!("Unknown setting: {}", key));
    }

    let mut result = new_lines.join("\r\n");
    if !result.ends_with("\r\n") {
        result.push_str("\r\n");
    }

    std::fs::write(&path, result).map_err(|e| e.to_string())
}

// ── Game launch ────────────────────────────────────────────────────

fn launch_arguments(nointro: bool, playonline: bool) -> Vec<&'static str> {
    let mut arguments = Vec::new();
    if nointro {
        arguments.push("-nointro");
    }
    if playonline {
        arguments.push("-playonline");
    }
    arguments
}

/// Launch a game executable from its installation directory.
pub fn launch_game(exe_path: &str, nointro: bool, playonline: bool) -> Result<(), String> {
    let exe = PathBuf::from(exe_path);
    let install_dir = exe.parent().ok_or_else(|| {
        format!(
            "Could not determine install directory for {}",
            exe.display()
        )
    })?;

    std::process::Command::new(&exe)
        .current_dir(install_dir)
        .args(launch_arguments(nointro, playonline))
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_variant_markers_select_and_switch_the_installed_proxy() {
        let dir =
            std::env::temp_dir().join(format!("wiclive-proxy-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        std::fs::write(dir.join(PROXY_VERSION_FILE), "standard-1").unwrap();
        assert!(check_proxy(dir.to_str().unwrap(), false));
        assert!(!check_proxy(dir.to_str().unwrap(), true));
        assert!(!is_compatibility_proxy(dir.to_str().unwrap()));

        std::fs::write(dir.join(COMPAT_PROXY_VERSION_FILE), "compat-1").unwrap();
        remove_other_proxy_marker(dir.to_str().unwrap(), true).unwrap();
        assert!(!check_proxy(dir.to_str().unwrap(), false));
        assert!(check_proxy(dir.to_str().unwrap(), true));
        assert!(is_compatibility_proxy(dir.to_str().unwrap()));
        assert_eq!(
            read_proxy_version(dir.to_str().unwrap(), true).unwrap(),
            "compat-1"
        );

        std::fs::remove_dir_all(dir).unwrap();
    }

    // ── launch executable resolution ───────────────────────────

    #[test]
    fn resolve_launch_exe_prefers_wic_and_falls_back_to_online() {
        let dir =
            std::env::temp_dir().join(format!("wiclive-launch-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let online = dir.join(GAME_EXE);
        std::fs::write(&online, []).unwrap();
        assert_eq!(resolve_launch_exe(dir.to_str().unwrap()).unwrap(), online);

        let base = dir.join(BASE_EXE);
        std::fs::write(&base, []).unwrap();
        assert_eq!(resolve_launch_exe(dir.to_str().unwrap()).unwrap(), base);

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn launch_arguments_follow_saved_options() {
        assert_eq!(launch_arguments(false, false), Vec::<&str>::new());
        assert_eq!(launch_arguments(true, false), vec!["-nointro"]);
        assert_eq!(launch_arguments(false, true), vec!["-playonline"]);
        assert_eq!(
            launch_arguments(true, true),
            vec!["-nointro", "-playonline"]
        );
    }

    #[test]
    fn launcher_settings_are_added_without_replacing_existing_config() {
        let original = "[camera_fix]\r\ncamera_fix=0\r\n";
        let updated = add_missing_launcher_settings(original).unwrap();

        assert!(updated.starts_with(original));
        assert_eq!(parse_wicgate_value(&updated, "camera_fix").unwrap(), "0");
        assert_eq!(parse_wicgate_value(&updated, "nointro").unwrap(), "0");
        assert_eq!(parse_wicgate_value(&updated, "playonline").unwrap(), "0");
        assert!(add_missing_launcher_settings(&updated).is_none());
    }

    #[test]
    fn launcher_settings_migration_adds_only_missing_keys() {
        let original = "[launcher]\r\nnointro=1\r\n";
        let updated = add_missing_launcher_settings(original).unwrap();

        assert_eq!(updated.matches("nointro=").count(), 1);
        assert_eq!(parse_wicgate_value(&updated, "nointro").unwrap(), "1");
        assert_eq!(parse_wicgate_value(&updated, "playonline").unwrap(), "0");
    }

    // ── welcome launcher flag ─────────────────────────────────

    fn launcher_fixture(opcode: u8) -> Vec<u8> {
        let mut bytes = vec![0x90; 11];
        bytes.extend_from_slice(&[
            0x39, 0x1D, 0xE8, 0x81, 0xDB, 0x00, opcode, 0x10, 0xE8, 0x10, 0x42, 0x05, 0x00, 0x83,
            0xF8, 0x01, 0x75, 0x06, 0x89, 0x3D, 0xE8, 0x81, 0xDB, 0x00, 0xE8, 0xC0, 0xEA, 0xFF,
            0xFF, 0xE8, 0x0B, 0x06, 0xFD, 0xFF, 0x39, 0x1D, 0xE8, 0x81, 0xDB, 0x00, 0x75, 0x39,
        ]);
        bytes.extend_from_slice(&[0x90; 7]);
        bytes
    }

    fn launcher_test_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("wiclive-launcher-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn finds_both_launcher_flag_states() {
        assert_eq!(
            find_launcher_flag_offset(&launcher_fixture(LAUNCHER_SHOW_OPCODE)).unwrap(),
            17
        );
        assert_eq!(
            find_launcher_flag_offset(&launcher_fixture(LAUNCHER_SKIP_OPCODE)).unwrap(),
            17
        );
    }

    #[test]
    fn rejects_missing_or_ambiguous_launcher_signatures() {
        assert!(find_launcher_flag_offset(&[0x90; 64]).is_err());

        let mut ambiguous = launcher_fixture(LAUNCHER_SHOW_OPCODE);
        ambiguous.extend_from_slice(&launcher_fixture(LAUNCHER_SKIP_OPCODE));
        assert!(find_launcher_flag_offset(&ambiguous).is_err());
    }

    #[test]
    fn launcher_flag_supports_online_only_installs() {
        let dir = launcher_test_dir("online-only");
        let online = dir.join(GAME_EXE);
        let original = launcher_fixture(LAUNCHER_SHOW_OPCODE);
        std::fs::write(&online, &original).unwrap();

        assert!(!check_skip_launcher(dir.to_str().unwrap()).unwrap());
        assert!(set_skip_launcher(dir.to_str().unwrap(), true).unwrap());

        let enabled = std::fs::read(&online).unwrap();
        let differences: Vec<_> = original
            .iter()
            .zip(&enabled)
            .enumerate()
            .filter(|(_, (before, after))| before != after)
            .collect();
        assert_eq!(differences.len(), 1);
        assert_eq!(differences[0].0, 17);
        assert_eq!(enabled[17], LAUNCHER_SKIP_OPCODE);

        assert!(!set_skip_launcher(dir.to_str().unwrap(), false).unwrap());
        assert_eq!(std::fs::read(&online).unwrap(), original);

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn launcher_flag_reads_off_if_either_exe_is_off_and_updates_both() {
        let dir = launcher_test_dir("mixed-state");
        let base = dir.join(BASE_EXE);
        let online = dir.join(GAME_EXE);
        std::fs::write(&base, launcher_fixture(LAUNCHER_SKIP_OPCODE)).unwrap();
        std::fs::write(&online, launcher_fixture(LAUNCHER_SHOW_OPCODE)).unwrap();

        assert!(!check_skip_launcher(dir.to_str().unwrap()).unwrap());
        assert!(set_skip_launcher(dir.to_str().unwrap(), true).unwrap());
        assert_eq!(std::fs::read(&base).unwrap()[17], LAUNCHER_SKIP_OPCODE);
        assert_eq!(std::fs::read(&online).unwrap()[17], LAUNCHER_SKIP_OPCODE);

        std::fs::write(&base, launcher_fixture(LAUNCHER_SHOW_OPCODE)).unwrap();
        assert!(!check_skip_launcher(dir.to_str().unwrap()).unwrap());

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn launcher_flag_validates_all_exes_before_writing() {
        let dir = launcher_test_dir("prevalidate");
        let base = dir.join(BASE_EXE);
        let online = dir.join(GAME_EXE);
        let original = launcher_fixture(LAUNCHER_SHOW_OPCODE);
        std::fs::write(&base, &original).unwrap();
        std::fs::write(&online, [0x90; 64]).unwrap();

        assert!(set_skip_launcher(dir.to_str().unwrap(), true).is_err());
        assert_eq!(std::fs::read(&base).unwrap(), original);

        std::fs::remove_dir_all(dir).unwrap();
    }

    // ── has_block ──────────────────────────────────────────────

    #[test]
    fn has_block_finds_present_block() {
        let contents = "some stuff\r\n// LIVE START\r\ndata\r\n// LIVE END\r\n";
        assert!(has_block(contents, LIVE_START, LIVE_END));
    }

    #[test]
    fn has_block_returns_false_when_missing() {
        assert!(!has_block("just some text", LIVE_START, LIVE_END));
    }

    #[test]
    fn has_block_returns_false_with_only_start() {
        assert!(!has_block("// LIVE START\r\ndata", LIVE_START, LIVE_END));
    }

    #[test]
    fn has_block_returns_false_with_only_end() {
        assert!(!has_block("data\r\n// LIVE END", LIVE_START, LIVE_END));
    }

    // ── remove_block ──────────────────────────────────────────

    #[test]
    fn remove_block_strips_block_with_crlf() {
        let input = "before\r\n// LIVE START\r\nstuff\r\n// LIVE END\r\nafter";
        let result = remove_block(input, LIVE_START, LIVE_END);
        assert_eq!(result, "before\r\nafter");
    }

    #[test]
    fn remove_block_strips_block_with_lf() {
        let input = "before\n// LIVE START\nstuff\n// LIVE END\nafter";
        let result = remove_block(input, LIVE_START, LIVE_END);
        assert_eq!(result, "before\nafter");
    }

    #[test]
    fn remove_block_at_start_of_file() {
        let input = "// LIVE START\r\nstuff\r\n// LIVE END\r\nafter";
        let result = remove_block(input, LIVE_START, LIVE_END);
        assert_eq!(result, "after");
    }

    #[test]
    fn remove_block_at_end_of_file() {
        let input = "before\r\n// LIVE START\r\nstuff\r\n// LIVE END";
        let result = remove_block(input, LIVE_START, LIVE_END);
        assert_eq!(result, "before");
    }

    #[test]
    fn remove_block_only_block() {
        let input = "// LIVE START\r\nstuff\r\n// LIVE END\r\n";
        let result = remove_block(input, LIVE_START, LIVE_END);
        assert_eq!(result, "");
    }

    #[test]
    fn remove_block_noop_when_missing() {
        let input = "just some text\r\n";
        let result = remove_block(input, LIVE_START, LIVE_END);
        assert_eq!(result, input);
    }

    #[test]
    fn remove_block_preserves_other_blocks() {
        let input = "before\r\n// LIVE START\r\ndata\r\n// LIVE END\r\n// COMPETITIVE START\r\nfog\r\n// COMPETITIVE END\r\nafter";
        let result = remove_block(input, LIVE_START, LIVE_END);
        assert_eq!(result, "before\r\n// COMPETITIVE START\r\nfog\r\n// COMPETITIVE END\r\nafter");
    }

    #[test]
    fn remove_comp_block_preserves_live() {
        let input = "// LIVE START\r\ndata\r\n// LIVE END\r\n// COMPETITIVE START\r\nfog\r\n// COMPETITIVE END\r\n";
        let result = remove_block(input, COMP_START, COMP_END);
        assert!(has_block(&result, LIVE_START, LIVE_END));
        assert!(!has_block(&result, COMP_START, COMP_END));
    }

    // ── LIVE_BLOCK / COMP_BLOCK constants ─────────────────────

    #[test]
    fn live_block_has_correct_markers() {
        assert!(LIVE_BLOCK.starts_with(LIVE_START));
        assert!(LIVE_BLOCK.ends_with(LIVE_END));
    }

    #[test]
    fn comp_block_has_correct_markers() {
        assert!(COMP_BLOCK.starts_with(COMP_START));
        assert!(COMP_BLOCK.ends_with(COMP_END));
    }

    #[test]
    fn live_block_uses_crlf() {
        assert!(LIVE_BLOCK.contains("\r\n"));
        assert!(!LIVE_BLOCK.contains("\r\n\n")); // no stray LFs
    }

    // ── Round-trip: insert then remove ────────────────────────

    #[test]
    fn insert_then_remove_live_is_clean() {
        let original = "some user config\r\n";
        let mut contents = original.to_string();
        contents.push_str(LIVE_BLOCK);
        contents.push_str("\r\n");

        let result = remove_block(&contents, LIVE_START, LIVE_END);
        assert_eq!(result, "some user config\r\n");
    }

    #[test]
    fn insert_both_then_remove_both() {
        let mut contents = String::new();
        contents.push_str(LIVE_BLOCK);
        contents.push_str("\r\n");
        contents.push_str(COMP_BLOCK);
        contents.push_str("\r\n");

        let contents = remove_block(&contents, COMP_START, COMP_END);
        let contents = remove_block(&contents, LIVE_START, LIVE_END);
        assert_eq!(contents, "");
    }

    #[test]
    fn insert_live_around_existing_content() {
        let original = "CameraMaxHeight 800\r\n";
        let mut contents = original.to_string();
        contents.push_str(LIVE_BLOCK);
        contents.push_str("\r\n");

        assert!(has_block(&contents, LIVE_START, LIVE_END));
        let result = remove_block(&contents, LIVE_START, LIVE_END);
        assert_eq!(result, "CameraMaxHeight 800\r\n");
    }
}
