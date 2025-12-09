use log::{error, info};
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

pub async fn check_and_install_updates(app: AppHandle) {
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
