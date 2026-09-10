//! HTML -> PDF conversion with a headless Chromium engine (Edge, falling back to Chrome).
//!
//! Every print path funnels through `try_generate_pdf_from_html`. Printing raw HTML produced
//! garbage on the thermal printers, so a failed conversion must fail the print, never fall
//! back.

use log::{info, warn};

fn wait_for_file_ready(path: &str, timeout_ms: u64, stable_ms: u64) -> bool {
    const POLL_MS: u64 = 25;

    let start = std::time::Instant::now();
    let mut last_size: Option<u64> = None;
    let mut stable_for = 0u64;

    while start.elapsed().as_millis() as u64 <= timeout_ms {
        if let Ok(metadata) = std::fs::metadata(path) {
            let size = metadata.len();
            if size > 0 {
                if Some(size) == last_size {
                    if stable_for >= stable_ms {
                        return true;
                    }
                    stable_for += POLL_MS;
                } else {
                    last_size = Some(size);
                    stable_for = 0;
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(POLL_MS));
    }

    false
}

/// Locates a Chromium-based browser usable for headless HTML->PDF conversion.
/// Checks Edge (system and per-user installs) first, then Chrome as fallback.
fn find_html_to_pdf_engine() -> Option<String> {
    // Probed once per run instead of up to six filesystem checks on every single print.
    static ENGINE: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    ENGINE.get_or_init(probe_html_to_pdf_engine).clone()
}

fn probe_html_to_pdf_engine() -> Option<String> {
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();

    let mut candidates = vec![
        "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe".to_string(),
        "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe".to_string(),
        "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe".to_string(),
        "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe".to_string(),
    ];

    if !local_app_data.is_empty() {
        candidates.insert(2, format!("{}\\Microsoft\\Edge\\Application\\msedge.exe", local_app_data));
        candidates.push(format!("{}\\Google\\Chrome\\Application\\chrome.exe", local_app_data));
    }

    candidates
        .into_iter()
        .find(|path| std::path::Path::new(path).exists())
}

pub fn try_generate_pdf_from_html(html_path_str: &str, pdf_path_str: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        // Remove any stale PDF so a failed conversion can't silently print an old document
        let _ = std::fs::remove_file(pdf_path_str);

        let engine_path = match find_html_to_pdf_engine() {
            Some(path) => path,
            None => {
                warn!("[PDF] No Edge/Chrome installation found for HTML->PDF conversion");
                return false;
            }
        };

        let file_url = format!(
            "file:///{}",
            html_path_str.replace('\\', "/").replace(' ', "%20")
        );

        let temp_dir = std::env::temp_dir().join("esoft_edge_pdf");
        let _ = std::fs::create_dir_all(&temp_dir);
        let user_data_arg = format!("--user-data-dir={}", temp_dir.to_string_lossy());
        let print_arg = format!("--print-to-pdf={}", pdf_path_str);
        info!("[PDF] Generating PDF with {}: {}", engine_path, pdf_path_str);

        let started = std::time::Instant::now();
        let mut child = match std::process::Command::new(&engine_path)
            .args(&[
                "--headless",
                "--disable-gpu",
                "--no-sandbox",
                "--disable-dev-shm-usage",
                "--no-pdf-header-footer",
                // Every asset in our templates is inlined, so nothing needs the network or
                // the profile services. Skipping them saves most of the cold-start cost.
                "--no-first-run",
                "--no-default-browser-check",
                "--disable-extensions",
                "--disable-background-networking",
                "--disable-component-update",
                "--disable-sync",
                "--disable-crash-reporter",
                "--disable-breakpad",
                "--disable-default-apps",
                "--mute-audio",
                // Bounds rendering: without it the engine can idle waiting on timers.
                "--virtual-time-budget=2000",
                &user_data_arg,
                &print_arg,
                &file_url,
            ])
            // Not inherited: .output() waits for the pipes to reach EOF, so any surviving
            // helper process (crashpad) would keep us blocked after the browser exited.
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                warn!("[PDF] Failed to launch {}: {}", engine_path, e);
                return false;
            }
        };

        // Bounded wait. std::process::Command::output() cannot time out, so a stale profile
        // lock in the user-data-dir used to hang the print forever — and, because callers
        // held the SQLite mutex, the whole app with it.
        const ENGINE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(25);
        let mut exited = false;
        while started.elapsed() < ENGINE_TIMEOUT {
            match child.try_wait() {
                Ok(Some(_)) => {
                    exited = true;
                    break;
                }
                Ok(None) => {
                    // The PDF may already be complete even if the process lingers.
                    if std::path::Path::new(pdf_path_str).exists()
                        && wait_for_file_ready(pdf_path_str, 300, 150)
                    {
                        exited = true;
                        break;
                    }
                }
                Err(e) => {
                    warn!("[PDF] Could not poll engine: {}", e);
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }

        if !exited {
            warn!(
                "[PDF] Engine did not finish in {} ms — killing it. If this repeats, the \
                 profile at {} is probably locked by a stray browser process.",
                started.elapsed().as_millis(),
                temp_dir.display()
            );
            let _ = child.kill();
            let _ = child.wait();
            // Drop the profile so the next print starts from a clean, unlocked one.
            let _ = std::fs::remove_dir_all(&temp_dir);
            return false;
        }

        // The engine has exited (or the PDF is already complete), so one bounded check
        // is enough. This used to be a retry loop that added only 100 to `waited` per
        // iteration while each iteration actually cost ~1400 ms, giving a real timeout of
        // ~140 s that was logged as "10s" — that was the ~90 s print.
        if wait_for_file_ready(pdf_path_str, 2000, 150) {
            info!("[PDF] PDF generated OK in {} ms", started.elapsed().as_millis());
            return true;
        }

        warn!("[PDF] PDF not ready after {} ms", started.elapsed().as_millis());
    }

    false
}
