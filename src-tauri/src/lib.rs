mod commands;
mod prospect;
mod server;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use commands::AppState;
use prospect::types::AppConfig;
use tauri_plugin_log::{Target, TargetKind};

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let msg = format!(
            "ICARUS Prospect Editor crashed.\n\n{info}\n\n\
             Log directory: %APPDATA%\\ICARUS Prospect Editor\\logs\\\n\
             Please attach app.log when reporting this."
        );

        if let Some(dir) = dirs::data_dir() {
            let crash_path = dir
                .join("ICARUS Prospect Editor")
                .join("logs")
                .join("crash.log");
            if let Some(parent) = crash_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&crash_path, &msg);
        }

        #[cfg(target_os = "windows")]
        unsafe {
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            let wide: Vec<u16> = OsStr::new(&msg)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let title: Vec<u16> = OsStr::new("ICARUS Prospect Editor — Startup Error")
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW(
                std::ptr::null_mut(),
                wide.as_ptr(),
                title.as_ptr(),
                0x10, // MB_ICONERROR
            );
        }
    }));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    install_panic_hook();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .targets([
                    Target::new(TargetKind::LogDir {
                        file_name: Some("app".into()),
                    }),
                    Target::new(TargetKind::Stdout),
                ])
                .build(),
        )
        .manage(Mutex::new(AppState {
            config: AppConfig::default(),
            server_config: crate::server::ServerConfig::default(),
            open_prospects: HashMap::new(),
        }))
        .manage(Arc::new(Mutex::new(crate::server::ServerState::default())))
        .invoke_handler(tauri::generate_handler![
            commands::list_prospects,
            commands::auto_detect_prospects_dir,
            commands::get_prospect_overview,
            commands::get_component_details,
            commands::get_domain_view,
            commands::update_metadata,
            commands::update_component_property,
            commands::save_prospect,
            commands::backup_prospect_cmd,
            commands::restore_prospect_cmd,
            commands::list_backups_cmd,
            commands::get_config,
            commands::set_config,
            commands::search_components,
            commands::diff_prospects,
            commands::get_inventory_view,
            commands::update_inventory_slot,
            commands::delete_inventory_slot,
            commands::add_inventory_item,
            commands::detect_server,
            commands::get_server_config,
            commands::set_server_config,
            commands::start_server,
            commands::stop_server,
            commands::get_server_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
