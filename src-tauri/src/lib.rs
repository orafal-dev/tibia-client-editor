mod appearances;
mod commands;
mod config_loader;
mod edit;
mod repack;
mod win2mac;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
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
        .invoke_handler(tauri::generate_handler![
            commands::edit_client,
            commands::diagnose_client,
            commands::load_config_file,
            commands::repack_client,
            commands::win2mac_assets,
            commands::edit_appearances,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
