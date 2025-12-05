use log::{error, info};
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

async fn check_and_install_updates(app: AppHandle) {
    info!("Checking for updates...");

    // Emit event to show loading screen
    let _ = app.emit("update-checking", ());

    let updater = match app.updater() {
        Ok(u) => u,
        Err(e) => {
            error!("Failed to get updater: {}", e);
            let _ = app.emit("update-done", ());
            return;
        }
    };

    let update = match updater.check().await {
        Ok(Some(update)) => update,
        Ok(None) => {
            info!("No update available - app is up to date");
            let _ = app.emit("update-done", ());
            return;
        }
        Err(e) => {
            error!("Failed to check for updates (offline?): {}", e);
            let _ = app.emit("update-done", ());
            return;
        }
    };

    let current_version = update.current_version.clone();
    let new_version = update.version.clone();

    info!("Update available: {} -> {}", current_version, new_version);
    let _ = app.emit("update-downloading", new_version.clone());

    // Download and install
    match update
        .download_and_install(
            |_downloaded, _total| {},
            || {
                info!("Download finished");
            },
        )
        .await
    {
        Ok(_) => {
            info!("Update installed successfully, restarting...");
            app.restart();
        }
        Err(e) => {
            error!("Failed to install update: {}", e);
            let _ = app.emit("update-done", ());
        }
    }
}

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
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                check_and_install_updates(handle).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
