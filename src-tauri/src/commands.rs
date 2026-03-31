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
    config: tauri::State<'_, crate::ApiConfig>,
) -> Result<String, String> {
    let install_dir = core::require_install_path()?;
    let url = format!("{}/proxy/download", config.url);
    let tmp_zip = std::env::temp_dir().join("proxy.zip");

    core::download_file(&url, &tmp_zip, |_, _| {}).await?;
    let count = core::extract_zip(&tmp_zip, &install_dir, |_, _, _| {})?;
    let _ = std::fs::remove_file(&tmp_zip);

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

#[tauri::command]
pub fn remove_proxy() -> Result<(), String> {
    let dir = core::require_install_path()?;
    core::remove_proxy(&dir)
}

#[tauri::command]
pub async fn request_cd_key(
    config: tauri::State<'_, crate::ApiConfig>,
) -> Result<String, String> {
    let url = format!("{}/cdkey/generate", config.url);
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
