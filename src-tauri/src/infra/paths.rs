//! Where the app keeps the files it generates.
//!
//! The directory name was previously spelled out at eleven call sites, some rooted at
//! `config_dir`, some at `data_dir`, with no stated reason for the difference. Everything
//! goes through here now so the layout is decided once.

use std::path::PathBuf;

/// The vendor directory name, under whichever base directory is being used.
const APP_DIR: &str = "facturi.softconsulting.com";

/// The app directory under the user's config dir — where generated documents live.
pub fn config_app_dir() -> Result<PathBuf, String> {
    dirs::config_dir()
        .map(|p| p.join(APP_DIR))
        .ok_or_else(|| "Nu s-a putut determina directorul de configurare".to_string())
}

/// The app directory under the user's data dir — where bundled tools live.
pub fn data_app_dir() -> Result<PathBuf, String> {
    dirs::data_dir()
        .map(|p| p.join(APP_DIR))
        .ok_or_else(|| "Nu s-a putut determina directorul de date".to_string())
}

/// Directory for generated reports.
pub fn reports_dir() -> Result<PathBuf, String> {
    Ok(config_app_dir()?.join("reports"))
}

/// Candidate directories for saving a receipt, most preferred first.
///
/// Several are tried because the config dir is not always writable on locked-down tablets.
pub fn receipts_dirs() -> Vec<PathBuf> {
    let mut dirs_to_try = Vec::new();

    for base in [dirs::config_dir(), dirs::document_dir(), dirs::data_dir()] {
        if let Some(path) = base {
            dirs_to_try.push(path.join(APP_DIR).join("receipts"));
        }
    }

    if let Ok(path) = std::env::current_dir() {
        dirs_to_try.push(path.join("receipts"));
    }

    dirs_to_try
}

/// Portable SumatraPDF shipped alongside the app data, used when no install is found.
pub fn portable_sumatra() -> Option<PathBuf> {
    let path = data_app_dir().ok()?.join("tools").join("SumatraPDF.exe");
    path.exists().then_some(path)
}
