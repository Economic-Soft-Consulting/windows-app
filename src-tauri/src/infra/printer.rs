//! Sending a PDF to a physical printer, via SumatraPDF.
//!
//! This block used to exist in five copies. They drifted: four dispatched with `.spawn()`,
//! the fifth blocked on `.output()` and omitted `-exit-when-done`/`-exit-on-print`, so it
//! waited forever on a process that stays resident. Four returned `Err` when SumatraPDF was
//! missing, one returned `Ok` with a message, one only logged a warning. Fixing the argument
//! list in v1.0.13 fixed one copy out of five — which is exactly why this module exists.

use log::{info, warn};
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::infra::paths;

/// Locates SumatraPDF, preferring the copy bundled with the app.
///
/// Probed once per run: this used to be up to four filesystem checks on every print.
pub fn find_sumatra() -> Option<String> {
    static SUMATRA: OnceLock<Option<String>> = OnceLock::new();
    SUMATRA.get_or_init(probe_sumatra).clone()
}

fn probe_sumatra() -> Option<String> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    // Bundled alongside the executable (tauri.conf.json "resources").
    if let Some(dir) = std::env::current_exe().ok().and_then(|exe| exe.parent().map(PathBuf::from)) {
        candidates.push(dir.join("resources").join("SumatraPDF.exe"));
    }

    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        candidates.push(
            PathBuf::from(user_profile)
                .join("AppData")
                .join("Local")
                .join("SumatraPDF")
                .join("SumatraPDF.exe"),
        );
    }

    candidates.push(PathBuf::from(r"C:\Program Files\SumatraPDF\SumatraPDF.exe"));
    candidates.push(PathBuf::from(r"C:\Program Files (x86)\SumatraPDF\SumatraPDF.exe"));

    // Portable copy under the app data dir, the last resort.
    if let Some(portable) = paths::portable_sumatra() {
        candidates.push(portable);
    }

    let found = candidates.into_iter().find(|p| p.exists());
    match &found {
        Some(path) => info!("[PRINT] Using SumatraPDF at {}", path.display()),
        None => warn!("[PRINT] SumatraPDF not found in any known location"),
    }
    found.map(|p| p.to_string_lossy().to_string())
}

/// Builds the SumatraPDF argument list for one print job.
///
/// Separated from the spawn so it can be asserted in tests — a missing
/// `-exit-when-done` here is what left a printer process resident.
pub fn build_print_args(printer: &str, pdf_path: &str) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();

    if printer.trim().is_empty() {
        args.push("-print-to-default".to_string());
    } else {
        args.push("-print-to".to_string());
        args.push(printer.trim().to_string());
    }

    args.extend([
        "-print-settings".to_string(),
        "noscale".to_string(),
        pdf_path.to_string(),
        "-silent".to_string(),
        "-exit-when-done".to_string(),
        "-exit-on-print".to_string(),
    ]);

    args
}

/// Sends `pdf_path` to `printer` (empty string means the Windows default printer).
///
/// Fire-and-forget: SumatraPDF is spawned, not waited on. Waiting gains nothing — the job is
/// already queued with the spooler — and blocking here used to freeze the UI.
pub fn print_pdf(printer: &str, pdf_path: &str, job_label: &str) -> Result<(), String> {
    let sumatra = find_sumatra().ok_or_else(|| {
        "SumatraPDF nu a fost găsit. Nu se poate tipări documentul.".to_string()
    })?;

    let args = build_print_args(printer, pdf_path);
    info!("[PRINT] {} -> printer '{}'", job_label, printer);

    std::process::Command::new(&sumatra)
        .args(&args)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("Tipărirea a eșuat ({}): {}", job_label, e))
}

/// Same as [`print_pdf`] but only logs on failure.
///
/// For secondary documents — the quality certificate printed after an invoice — where
/// failing the whole command would be worse than skipping the extra page.
pub fn print_pdf_best_effort(printer: &str, pdf_path: &str, job_label: &str) {
    if let Err(e) = print_pdf(printer, pdf_path, job_label) {
        warn!("[PRINT] {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::build_print_args;

    /// Omitting these left SumatraPDF resident, which hung the caller that waited on it.
    #[test]
    fn always_asks_sumatra_to_exit_when_done() {
        for printer in ["", "  ", "Brother QL-800"] {
            let args = build_print_args(printer, "C:\\tmp\\doc.pdf");
            assert!(args.contains(&"-exit-when-done".to_string()), "printer={:?}", printer);
            assert!(args.contains(&"-exit-on-print".to_string()), "printer={:?}", printer);
            assert!(args.contains(&"-silent".to_string()), "printer={:?}", printer);
        }
    }

    #[test]
    fn uses_default_printer_when_none_named() {
        let args = build_print_args("   ", "C:\\tmp\\doc.pdf");
        assert_eq!(args[0], "-print-to-default");
        assert!(!args.contains(&"-print-to".to_string()));
    }

    #[test]
    fn names_the_printer_when_given() {
        let args = build_print_args(" Brother QL-800 ", "C:\\tmp\\doc.pdf");
        assert_eq!(args[0], "-print-to");
        assert_eq!(args[1], "Brother QL-800", "printer name must be trimmed");
    }

    /// Thermal receipts come out wrong if the driver is allowed to scale them.
    #[test]
    fn never_scales_the_document() {
        let args = build_print_args("", "C:\\tmp\\doc.pdf");
        let i = args.iter().position(|a| a == "-print-settings").expect("-print-settings");
        assert_eq!(args[i + 1], "noscale");
    }

    #[test]
    fn includes_the_file_to_print() {
        let args = build_print_args("", "C:\\tmp\\doc.pdf");
        assert!(args.contains(&"C:\\tmp\\doc.pdf".to_string()));
    }
}
