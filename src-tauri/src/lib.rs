use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_updater::{Update, UpdaterExt};

#[derive(Debug, Serialize, Clone)]
struct UpdateInfo {
    version: String,
    current_version: String,
}

#[derive(Debug, Serialize, Clone)]
struct DownloadProgress {
    downloaded: usize,
    total: Option<u64>,
}

struct PendingUpdate(Mutex<Option<Update>>);

#[tauri::command]
async fn install_update(
    app: AppHandle,
    pending: State<'_, PendingUpdate>,
) -> Result<(), String> {
    let update = {
        let mut guard = pending.0.lock().map_err(|e| e.to_string())?;
        guard.take()
    };

    if let Some(update) = update {
        let app_clone = app.clone();
        update
            .download_and_install(
                move |downloaded, total| {
                    let _ = app_clone.emit(
                        "update-download-progress",
                        DownloadProgress {
                            downloaded,
                            total,
                        },
                    );
                },
                || {},
            )
            .await
            .map_err(|e| e.to_string())?;

        app.emit("update-installed", ()).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("No pending update".to_string())
    }
}

#[tauri::command]
async fn restart_app(app: AppHandle) {
    app.restart();
}

async fn check_for_updates(app: AppHandle) {
    let updater = match app.updater() {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Failed to get updater: {}", e);
            return;
        }
    };

    let update = match updater.check().await {
        Ok(Some(update)) => update,
        Ok(None) => {
            println!("No update available");
            return;
        }
        Err(e) => {
            eprintln!("Failed to check for updates (offline?): {}", e);
            return;
        }
    };

    let current_version = update.current_version.clone();
    let new_version = update.version.clone();

    println!("Update available: {} -> {}", current_version, new_version);

    if let Some(pending) = app.try_state::<PendingUpdate>() {
        if let Ok(mut guard) = pending.0.lock() {
            *guard = Some(update);
        }
    }

    let _ = app.emit(
        "update-available",
        UpdateInfo {
            version: new_version,
            current_version,
        },
    );
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(PendingUpdate(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![install_update, restart_app])
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                check_for_updates(handle).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
