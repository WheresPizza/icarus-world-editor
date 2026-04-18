mod commands;
mod prospect;

use std::collections::HashMap;
use std::sync::Mutex;

use commands::AppState;
use prospect::types::AppConfig;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .manage(Mutex::new(AppState {
            config: AppConfig::default(),
            open_prospects: HashMap::new(),
        }))
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
