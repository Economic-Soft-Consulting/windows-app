use log::{error, info};
use serde::Serialize;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

/// How long to wait for the update server before giving up and letting the user in.
///
/// The check runs behind a full-screen overlay, so an unbounded request means the agent
/// cannot start work at all. The case that never recovers on its own is a captive-portal
/// hotspot, which answers slowly instead of failing outright.
///
/// This bounds only the check: the plugin builds every `Update` with `timeout: None`, so the
/// download is deliberately left unbounded — a 39 MB installer over mobile data must not be
/// cut off after a few seconds.
const CHECK_TIMEOUT: Duration = Duration::from_secs(15);

/// Minimum gap between two progress events.
///
/// `download_and_install` calls back on every chunk, which is several thousand times for a
/// 39 MB installer. Emitting each one would flood the IPC channel to say almost nothing.
const PROGRESS_INTERVAL: Duration = Duration::from_millis(200);

#[derive(Clone, Serialize)]
struct DownloadProgress {
    downloaded: u64,
    /// `None` when the server does not send a content length, in which case the UI can only
    /// show bytes downloaded so far.
    total: Option<u64>,
}

#[derive(Clone, Serialize)]
struct UpdateFailure {
    /// Already phrased for the user, in Romanian.
    message: String,
    /// The version they stay on, so the message can say what they are actually running.
    current_version: String,
}

pub async fn check_and_install_updates(app: AppHandle) {
    let current_version = app.package_info().version.to_string();
    info!("Checking for updates (current version {})...", current_version);

    let _ = app.emit("update-checking", ());

    // A failure to check is not a failure to report: being offline is the normal case in the
    // field, and telling the agent about it every morning would train them to ignore it.
    // Only a check that succeeded and then broke while installing is worth surfacing.
    let updater = match app.updater_builder().timeout(CHECK_TIMEOUT).build() {
        Ok(updater) => updater,
        Err(e) => {
            error!("Failed to build updater: {}", e);
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

    let new_version = update.version.clone();
    info!("Update available: {} -> {}", current_version, new_version);
    let _ = app.emit("update-downloading", new_version.clone());

    // The progress callback exists so the screen can prove the download is alive. Without it
    // a 39 MB transfer looks identical to a hang, and closing the app mid-install is the one
    // thing the user must not do.
    let progress_app = app.clone();
    let mut downloaded: u64 = 0;
    let mut last_emit = Instant::now() - PROGRESS_INTERVAL;

    let outcome = update
        .download_and_install(
            move |chunk_length, content_length| {
                downloaded += chunk_length as u64;

                let complete = Some(downloaded) == content_length;
                if complete || last_emit.elapsed() >= PROGRESS_INTERVAL {
                    last_emit = Instant::now();
                    let _ = progress_app.emit(
                        "update-progress",
                        DownloadProgress {
                            downloaded,
                            total: content_length,
                        },
                    );
                }
            },
            || info!("Download finished, installing..."),
        )
        .await;

    match outcome {
        Ok(_) => {
            info!("Update installed successfully, restarting...");
            app.restart();
        }
        Err(e) => {
            // Previously this only reached the log file, which nobody reads, so a device could
            // sit on an old version for months while every launch silently failed.
            error!("Failed to install update {}: {}", new_version, e);
            let _ = app.emit(
                "update-failed",
                UpdateFailure {
                    message: format!(
                        "Actualizarea la versiunea {} nu a reușit. Aplicația continuă pe versiunea {}.",
                        new_version, current_version
                    ),
                    current_version: current_version.clone(),
                },
            );
        }
    }
}
