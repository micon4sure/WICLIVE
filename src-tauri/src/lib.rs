mod commands;
pub mod core;

use tauri::Manager;

pub mod elevation;

pub fn run() {
    dotenvy::dotenv().ok();

    let api_url = std::env::var("API_URL").expect("API_URL environment variable must be set");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(debug_assertions)]
            if let Some(window) = app.get_webview_window("main") {
                window.open_devtools();
            }
            let _ = app;
            Ok(())
        })
        .manage(ApiConfig { url: api_url })
        .invoke_handler(tauri::generate_handler![
            commands::get_install_path,
            commands::get_game_version,
            commands::get_game_version_registry,
            commands::get_laa_flag,
            commands::set_laa_flag,
            commands::unset_laa_flag,
            commands::get_cd_key,
            commands::set_cd_key,
            commands::check_vcredist,
            commands::check_hooks,
            commands::get_hooks_version,
            commands::needs_hooks_update,
            commands::is_soviet_assault,
            commands::start_game,
            commands::reset_game,
            commands::get_api_url,
            commands::install_proxy,
            commands::get_latest_proxy_version,
            commands::remove_proxy,
            commands::apply_patches,
            commands::request_cd_key,
            commands::has_registry_install_path,
            commands::clear_install_registry,
            commands::get_autoexec_state,
            commands::set_live_settings,
            commands::set_competitive_settings,
            commands::get_debug_info,
            commands::list_map_files,
            commands::delete_all_maps,
            commands::get_map_hash,
            commands::download_map,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub struct ApiConfig {
    pub url: String,
}
