use log::info;

#[cfg(not(debug_assertions))]
mod updater;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir { file_name: Some("app".into()) },
                ))
                .build(),
        )
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            info!("App started, version: {}", app.package_info().version);

            #[cfg(not(debug_assertions))]
            {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    updater::check_and_install_updates(handle).await;
                });
            }

            #[cfg(debug_assertions)]
            info!("Dev mode - skipping auto-updater");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
