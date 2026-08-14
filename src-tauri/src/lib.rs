mod commands;
pub mod core;

use tauri::Manager;

pub mod elevation;

pub fn run() {
    let api_url = api_url();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
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
            commands::get_skip_launcher_flag,
            commands::set_skip_launcher_flag,
            commands::get_cd_key,
            commands::set_cd_key,
            commands::check_vcredist,
            commands::check_dx9,
            commands::install_dx9,
            commands::check_proxy,
            commands::get_proxy_version,
            commands::needs_proxy_update,
            commands::is_soviet_assault,
            commands::start_game,
            commands::reset_game,
            commands::get_api_url,
            commands::is_portable,
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
            commands::get_wicgate_settings,
            commands::set_wicgate_setting,
            commands::get_debug_info,
            commands::list_map_files,
            commands::delete_all_maps,
            commands::get_map_hash,
            commands::download_map,
            commands::download_installer,
            commands::extract_installer,
            commands::check_legacy_proxy,
            commands::is_legacy_proxy_active,
            commands::download_legacy_proxy,
            commands::install_legacy_proxy,
            commands::activate_legacy_proxy,
            commands::deactivate_legacy_proxy,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub struct ApiConfig {
    pub url: String,
}

pub fn api_url() -> String {
    match std::env::var("API_URL") {
        Ok(url) => url,
        Err(_) => {
            if cfg!(debug_assertions) {
                panic!("API_URL not set. Use `bun goes.ts run development` instead of `bun run tauri dev`");
            }
            "https://wiclive.wicgate.org".into()
        }
    }
}

pub fn uninstall() {
    match core::uninstall_game() {
        Ok(msg) => {
            #[cfg(windows)]
            {
                use windows::Win32::UI::WindowsAndMessaging::*;
                let text: Vec<u16> = format!("{}\0", msg).encode_utf16().collect();
                let title: Vec<u16> = "WIC LIVE\0".encode_utf16().collect();
                unsafe {
                    MessageBoxW(
                        None,
                        windows::core::PCWSTR(text.as_ptr()),
                        windows::core::PCWSTR(title.as_ptr()),
                        MB_OK | MB_ICONINFORMATION,
                    );
                }
            }
            #[cfg(not(windows))]
            println!("{}", msg);
        }
        Err(e) => {
            eprintln!("Uninstall failed: {}", e);
            std::process::exit(1);
        }
    }
}
