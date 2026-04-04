use std::path::PathBuf;
use serde::Serialize;
use tauri::Emitter;
use crate::core;

// ── Serializable version for IPC ──────────────────────────────────

#[derive(Serialize)]
pub struct VersionInfo {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub build: u16,
}

impl From<core::VersionInfo> for VersionInfo {
    fn from(v: core::VersionInfo) -> Self {
        Self { major: v.major, minor: v.minor, patch: v.patch, build: v.build }
    }
}

// ── Tauri command wrappers ────────────────────────────────────────

#[tauri::command]
pub fn get_install_path() -> Option<String> {
    core::get_install_path()
}

#[tauri::command]
pub fn get_game_version() -> Result<VersionInfo, String> {
    let exe = core::require_exe_path()?;
    core::read_pe_version(exe.to_str().unwrap()).map(Into::into)
}

#[tauri::command]
pub fn get_game_version_registry() -> Option<String> {
    core::read_registry_version()
}

#[tauri::command]
pub fn get_laa_flag() -> Result<bool, String> {
    let exe = core::require_exe_path()?;
    core::check_laa(exe.to_str().unwrap())
}

#[tauri::command]
pub fn set_laa_flag() -> Result<bool, String> {
    let exe = core::require_exe_path()?;
    core::apply_laa(exe.to_str().unwrap())
}

#[tauri::command]
pub fn unset_laa_flag() -> Result<bool, String> {
    let exe = core::require_exe_path()?;
    core::unset_laa(exe.to_str().unwrap())
}

#[tauri::command]
pub fn get_cd_key() -> Result<String, String> {
    core::read_cd_key()
}

#[tauri::command]
pub fn set_cd_key(key: String) -> Result<(), String> {
    core::write_cd_key(&key)
}

#[tauri::command]
pub fn check_vcredist() -> bool {
    core::check_vcredist()
}

#[tauri::command]
pub fn check_hooks() -> Result<bool, String> {
    let dir = core::require_install_path()?;
    Ok(core::check_proxy(&dir))
}

#[tauri::command]
pub fn get_hooks_version() -> Result<String, String> {
    let dir = core::require_install_path()?;
    core::read_proxy_version(&dir)
}

#[tauri::command]
pub fn needs_hooks_update(latest: String) -> Result<bool, String> {
    let dir = core::require_install_path()?;
    core::needs_proxy_update(&dir, &latest)
}

#[tauri::command]
pub fn is_soviet_assault() -> bool {
    core::get_install_path()
        .map(|p| core::is_soviet_assault(&p))
        .unwrap_or(false)
}

#[tauri::command]
pub fn start_game() -> Result<(), String> {
    let exe = core::require_exe_path()?;
    core::launch_game(exe.to_str().unwrap())
}

#[tauri::command]
pub fn reset_game(variant: String) -> Result<(), String> {
    let dir = core::require_install_path()?;
    core::reset_exe(&dir, &variant)
}

#[tauri::command]
pub async fn install_proxy(
    app: tauri::AppHandle,
    config: tauri::State<'_, crate::ApiConfig>,
) -> Result<String, String> {
    let install_dir = core::require_install_path()?;
    let url = format!("{}/proxy/download", config.url);
    let tmp_zip = std::env::temp_dir().join("proxy.zip");

    let app_dl = app.clone();
    core::download_file(&url, &tmp_zip, move |downloaded, total| {
        let _ = app_dl.emit("proxy-progress", PatchProgress {
            stage: "downloading".into(),
            downloaded,
            total,
            detail: "proxy.zip".into(),
        });
    }).await?;

    let app_ex = app.clone();
    let count = core::extract_zip(&tmp_zip, &install_dir, move |done, total, name| {
        let _ = app_ex.emit("proxy-progress", PatchProgress {
            stage: "extracting".into(),
            downloaded: done,
            total,
            detail: name.to_string(),
        });
    })?;

    let _ = std::fs::remove_file(&tmp_zip);

    let _ = app.emit("proxy-progress", PatchProgress {
        stage: "done".into(),
        downloaded: count,
        total: count,
        detail: format!("{} files extracted", count),
    });

    let ver = core::read_proxy_version(&install_dir).unwrap_or_default();
    Ok(ver.trim().to_string())
}

#[tauri::command]
pub async fn get_latest_proxy_version(
    config: tauri::State<'_, crate::ApiConfig>,
) -> Result<String, String> {
    let url = format!("{}/proxy/version", config.url);
    let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    resp.text().await.map_err(|e| e.to_string())
}

// ── Legacy Proxy (PBE) ────────────────────────────────────────────

#[tauri::command]
pub fn check_legacy_proxy() -> Result<bool, String> {
    let dir = core::require_install_path()?;
    Ok(core::check_legacy_proxy(&dir))
}

#[tauri::command]
pub fn is_legacy_proxy_active() -> Result<bool, String> {
    let dir = core::require_install_path()?;
    Ok(core::is_legacy_proxy_active(&dir))
}

#[tauri::command]
pub async fn download_legacy_proxy(
    app: tauri::AppHandle,
) -> Result<String, String> {
    // Fetch version from wicgate.com
    let version_url = "https://www.wicgate.com/wic_cl_hook-version.txt";
    let resp = reqwest::get(version_url).await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("Failed to fetch legacy version: HTTP {}", resp.status()));
    }
    let version = resp.text().await.map_err(|e| e.to_string())?;
    let version = version.trim().to_string();

    // Download the update zip
    let url = format!("https://www.wicgate.com/wicgate_update_{}.zip", version);
    let tmp_zip = std::env::temp_dir().join("legacy-proxy.zip");

    let app_dl = app.clone();
    core::download_file(&url, &tmp_zip, move |downloaded, total| {
        let _ = app_dl.emit("legacy-proxy-progress", PatchProgress {
            stage: "downloading".into(),
            downloaded,
            total,
            detail: format!("wicgate_update_{}.zip", version),
        });
    }).await?;

    Ok(tmp_zip.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn install_legacy_proxy(
    app: tauri::AppHandle,
) -> Result<(), String> {
    let install_dir = core::require_install_path()?;
    let dir = PathBuf::from(&install_dir);
    let tmp_zip = std::env::temp_dir().join("legacy-proxy.zip");

    if !tmp_zip.exists() {
        return Err("Legacy proxy not downloaded yet".into());
    }

    // Save current dbghelp.dll as live backup before extraction overwrites it
    let active = dir.join("dbghelp.dll");
    let live = dir.join("dbghelp-live.dll");
    if active.exists() && !live.exists() {
        std::fs::copy(&active, &live)
            .map_err(|e| format!("Failed to backup live proxy: {}", e))?;
    }

    // Extract zip to install dir (overwrites dbghelp.dll with legacy version)
    let app_ex = app.clone();
    core::extract_zip(&tmp_zip, &install_dir, move |done, total, name| {
        let _ = app_ex.emit("legacy-proxy-progress", PatchProgress {
            stage: "extracting".into(),
            downloaded: done,
            total,
            detail: name.to_string(),
        });
    })?;

    // Rename extracted dbghelp.dll to dbghelp-pbe.dll
    let pbe = dir.join("dbghelp-pbe.dll");
    if active.exists() {
        std::fs::copy(&active, &pbe)
            .map_err(|e| format!("Failed to save PBE proxy: {}", e))?;
    }

    // Restore live proxy as active
    if live.exists() {
        std::fs::copy(&live, &active)
            .map_err(|e| format!("Failed to restore live proxy: {}", e))?;
    }

    // Write mode file
    std::fs::write(dir.join("wiclive-mode.txt"), "live")
        .map_err(|e| format!("Failed to write mode: {}", e))?;

    // Cleanup
    let _ = std::fs::remove_file(&tmp_zip);

    let _ = app.emit("legacy-proxy-progress", PatchProgress {
        stage: "done".into(),
        downloaded: 0,
        total: 0,
        detail: "Legacy proxy installed".into(),
    });

    Ok(())
}

#[tauri::command]
pub fn activate_legacy_proxy() -> Result<(), String> {
    let dir = core::require_install_path()?;
    core::activate_legacy_proxy(&dir)
}

#[tauri::command]
pub fn deactivate_legacy_proxy() -> Result<(), String> {
    let dir = core::require_install_path()?;
    core::deactivate_legacy_proxy(&dir)
}

// ── Proxy ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn remove_proxy() -> Result<(), String> {
    let dir = core::require_install_path()?;
    core::remove_proxy(&dir)
}

#[tauri::command]
pub async fn request_cd_key(
    config: tauri::State<'_, crate::ApiConfig>,
    source: String,
) -> Result<String, String> {
    let url = format!("{}/cdkey/generate/{}", config.url, source);
    let client = reqwest::Client::new();
    let resp = client.post(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {}: {}", status, body));
    }
    #[derive(serde::Deserialize)]
    struct KeyResponse { key: String }
    let data: KeyResponse = resp.json().await.map_err(|e| e.to_string())?;
    Ok(data.key)
}

#[tauri::command]
pub fn get_api_url(config: tauri::State<'_, crate::ApiConfig>) -> String {
    config.url.clone()
}

#[tauri::command]
pub fn is_portable() -> bool {
    cfg!(feature = "portable")
}

// ── Maps ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_map_files() -> Result<Vec<String>, String> {
    core::list_map_files()
}

#[tauri::command]
pub fn delete_all_maps() -> Result<u32, String> {
    let files = core::list_map_files()?;
    let maps_dir = core::get_maps_dir()?;
    let mut deleted = 0u32;
    for f in &files {
        let path = maps_dir.join(f);
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete {}: {}", f, e))?;
        deleted += 1;
    }
    Ok(deleted)
}

#[tauri::command]
pub async fn get_map_hash(filename: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || core::get_map_hash(&filename))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn download_map(
    app: tauri::AppHandle,
    config: tauri::State<'_, crate::ApiConfig>,
    filename: String,
) -> Result<(), String> {
    let maps_dir = core::get_maps_dir()?;
    if !maps_dir.exists() {
        std::fs::create_dir_all(&maps_dir).map_err(|e| e.to_string())?;
    }
    let dest = maps_dir.join(&filename);
    let url = format!("{}/maps/download/{}", config.url, filename);

    let app_dl = app.clone();
    let fname = filename.clone();
    core::download_file(&url, &dest, move |downloaded, total| {
        let _ = app_dl.emit("map-progress", PatchProgress {
            stage: "downloading".into(),
            downloaded,
            total,
            detail: fname.clone(),
        });
    }).await?;

    let _ = app.emit("map-progress", PatchProgress {
        stage: "done".into(),
        downloaded: 0,
        total: 0,
        detail: filename,
    });

    Ok(())
}

#[tauri::command]
pub fn has_registry_install_path() -> bool {
    core::has_registry_install_path()
}

// ── Install registry cleanup ─────────────────────────────────────

#[tauri::command]
pub fn clear_install_registry() -> Result<(), String> {
    core::clear_install_registry()
}

// ── Autoexec config ─────────────────────────────────────────────

#[tauri::command]
pub fn get_autoexec_state() -> Result<(bool, bool), String> {
    core::get_autoexec_state()
}

#[tauri::command]
pub fn set_live_settings(enabled: bool) -> Result<(), String> {
    core::set_live_settings(enabled)
}

#[tauri::command]
pub fn set_competitive_settings(enabled: bool) -> Result<(), String> {
    core::set_competitive_settings(enabled)
}

// ── Debug info ───────────────────────────────────────────────────

#[tauri::command]
pub async fn get_debug_info(
    config: tauri::State<'_, crate::ApiConfig>,
) -> Result<String, String> {
    let mut lines: Vec<String> = vec!["=== WIC LIVE Debug Info ===".into()];

    // API URL
    lines.push(format!("API URL: {}", config.url));

    // Install path
    match core::get_install_path() {
        Some(p) => lines.push(format!("Install Dir: {}", p)),
        None => lines.push("Install Dir: NOT FOUND".into()),
    }

    // Base / Maps dir
    match core::get_base_directory() {
        Ok(p) => lines.push(format!("Base Dir: {}", p.display())),
        Err(e) => lines.push(format!("Base Dir: {}", e)),
    }
    match core::get_maps_dir() {
        Ok(p) => {
            lines.push(format!("Maps Dir: {}", p.display()));
            lines.push(format!("Maps Dir Exists: {}", p.exists()));
            match core::list_map_files() {
                Ok(files) => lines.push(format!("Local Maps: {}", files.len())),
                Err(e) => lines.push(format!("Local Maps: error ({})", e)),
            }
        }
        Err(e) => lines.push(format!("Maps Dir: error ({})", e)),
    }

    // Game version
    match core::require_exe_path() {
        Ok(exe) => {
            let exe_str = exe.to_string_lossy();
            lines.push(format!("Exe Path: {}", exe_str));
            match core::read_pe_version(&exe_str) {
                Ok(v) => lines.push(format!("Game Version: {}", v)),
                Err(e) => lines.push(format!("Game Version: error ({})", e)),
            }
            match core::check_laa(&exe_str) {
                Ok(true) => lines.push("LAA: enabled".into()),
                Ok(false) => lines.push("LAA: disabled".into()),
                Err(e) => lines.push(format!("LAA: error ({})", e)),
            }
        }
        Err(e) => lines.push(format!("Exe Path: {}", e)),
    }

    // VC++
    lines.push(format!("VC++ Redist: {}", if core::check_vcredist() { "installed" } else { "missing" }));

    // CD Key
    match core::read_cd_key() {
        Ok(k) if !k.is_empty() => lines.push(format!("CD Key: {}", k)),
        Ok(_) => lines.push("CD Key: not set".into()),
        Err(e) => lines.push(format!("CD Key: error ({})", e)),
    }

    // Registry version
    match core::read_registry_version() {
        Some(v) => lines.push(format!("Registry Version: {}", v)),
        None => lines.push("Registry Version: not found".into()),
    }

    // Proxy
    match core::require_install_path() {
        Ok(dir) => {
            if core::check_proxy(&dir) {
                match core::read_proxy_version(&dir) {
                    Ok(v) => lines.push(format!("Proxy: {}", v.trim())),
                    Err(_) => lines.push("Proxy: installed (version unknown)".into()),
                }
            } else {
                lines.push("Proxy: not installed".into());
            }
            lines.push(format!("Soviet Assault: {}", core::is_soviet_assault(&dir)));
        }
        Err(_) => {}
    }

    // Env vars
    for var in &["GAME_DIR", "MAPS_DIR", "API_URL"] {
        if let Ok(val) = std::env::var(var) {
            lines.push(format!("env {}: {}", var, val));
        }
    }

    // Platform
    lines.push(format!("OS: {} {}", std::env::consts::OS, std::env::consts::ARCH));

    Ok(lines.join("\n"))
}

// ── Patch install ─────────────────────────────────────────────────

#[derive(Clone, Serialize)]
pub struct PatchProgress {
    pub stage: String,       // "downloading" | "extracting" | "done" | "error"
    pub downloaded: u64,
    pub total: u64,
    pub detail: String,
}

#[tauri::command]
pub async fn apply_patches(
    app: tauri::AppHandle,
    config: tauri::State<'_, crate::ApiConfig>,
) -> Result<String, String> {
    let exe = core::require_exe_path()?;
    let ver = core::read_pe_version(exe.to_str().unwrap())?;

    let patch_name = core::needed_patch(&ver)
        .ok_or_else(|| "Game is already up to date".to_string())?;

    let url = format!("{}/patches/{}", config.url, patch_name);
    let tmp_dir = std::env::temp_dir();
    let tmp_zip = tmp_dir.join(patch_name);

    // Download
    let app_dl = app.clone();
    core::download_file(&url, &tmp_zip, move |downloaded, total| {
        let _ = app_dl.emit("patch-progress", PatchProgress {
            stage: "downloading".into(),
            downloaded,
            total,
            detail: patch_name.into(),
        });
    }).await?;

    // Extract
    let install_dir = core::require_install_path()?;
    let app_ex = app.clone();
    let count = core::extract_zip(&tmp_zip, &install_dir, move |done, total, name| {
        let _ = app_ex.emit("patch-progress", PatchProgress {
            stage: "extracting".into(),
            downloaded: done,
            total,
            detail: name.to_string(),
        });
    })?;

    // Update registry version
    let _ = core::set_registry_version("1.0.1.1");

    // Cleanup
    let _ = std::fs::remove_file(&tmp_zip);

    let _ = app.emit("patch-progress", PatchProgress {
        stage: "done".into(),
        downloaded: count,
        total: count,
        detail: format!("{} files extracted", count),
    });

    Ok(format!("Patched with {} ({} files)", patch_name, count))
}

// ── Installer download ───────────────────────────────────────────

#[tauri::command]
pub async fn download_installer(
    app: tauri::AppHandle,
    config: tauri::State<'_, crate::ApiConfig>,
) -> Result<String, String> {
    let dest = std::env::temp_dir().join("WIC_MP_ONLY_installer.zip");
    let url = format!("{}/files/WIC_MP_ONLY_installer.zip", config.url);

    let app_dl = app.clone();
    core::download_file(&url, &dest, move |downloaded, total| {
        let _ = app_dl.emit("installer-progress", PatchProgress {
            stage: "downloading".into(),
            downloaded,
            total,
            detail: "WIC_MP_ONLY_installer.zip".into(),
        });
    }).await?;

    let _ = app.emit("installer-progress", PatchProgress {
        stage: "done".into(),
        downloaded: 0,
        total: 0,
        detail: dest.to_string_lossy().into_owned(),
    });

    Ok(dest.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn extract_installer(
    app: tauri::AppHandle,
    install_dir: String,
) -> Result<String, String> {
    let zip_path = std::env::temp_dir().join("WIC_MP_ONLY_installer.zip");
    if !zip_path.exists() {
        return Err("Installer not downloaded yet".into());
    }
    std::fs::create_dir_all(&install_dir).map_err(|e| e.to_string())?;

    let app_ex = app.clone();
    let count = core::extract_zip(&zip_path, &install_dir, move |done, total, name| {
        let _ = app_ex.emit("installer-progress", PatchProgress {
            stage: "extracting".into(),
            downloaded: done,
            total,
            detail: name.to_string(),
        });
    })?;

    let _ = std::fs::remove_file(&zip_path);

    // Register installation
    core::register_install(&install_dir)?;

    let _ = app.emit("installer-progress", PatchProgress {
        stage: "done".into(),
        downloaded: count,
        total: count,
        detail: format!("{} files extracted", count),
    });

    Ok(format!("{} files extracted to {}", count, install_dir))
}
