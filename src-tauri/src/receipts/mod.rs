//! Writing the receipt (chitanta) HTML and PDF to disk.
//!
//! Kept separate from sending: send_collection used to generate these files as a side effect
//! of a successful POST, which made retrying a rejected receipt rewrite files it had already
//! written.

use log::{info, warn};

use crate::infra::paths;
use crate::commands::read_logo_to_base64;
use crate::infra::pdf::try_generate_pdf_from_html;
use crate::models::*;
use crate::print_receipt;

pub fn save_receipt_html_file(
    collection: &Collection,
    doc_series: &str,
    doc_number: &str,
    issue_date: &str,
    agent_name: Option<&str>,
    nume_casa: &str,
    partner_address: Option<&str>,
    partner_localitate: Option<&str>,
    partner_judet: Option<&str>,
    partner_cui: Option<&str>,
    partner_reg_com: Option<&str>,
    file_id: &str,
) -> Result<(String, String, bool), String> {
    let logo_base64 = read_logo_to_base64();
    let html = print_receipt::generate_receipt_html(
        collection,
        logo_base64.as_deref(),
        doc_series,
        doc_number,
        issue_date,
        agent_name,
        nume_casa,
        partner_address,
        partner_localitate,
        partner_judet,
        partner_cui,
        partner_reg_com,
    );

    let mut failures = Vec::new();

    for dir in paths::receipts_dirs() {
        if let Err(e) = std::fs::create_dir_all(&dir) {
            failures.push(format!("create_dir_all {}: {}", dir.display(), e));
            continue;
        }

        let html_file_path = dir.join(format!("chitanta_{}.html", file_id));
        let pdf_file_path = dir.join(format!("chitanta_{}.pdf", file_id));

        match std::fs::write(&html_file_path, &html) {
            Ok(_) => {
                let html_path = html_file_path.to_string_lossy().to_string();
                let pdf_path = pdf_file_path.to_string_lossy().to_string();
                let pdf_generated = try_generate_pdf_from_html(&html_path, &pdf_path);
                info!("[CHITANTE][SAVE] Saved receipt HTML at {}", html_path);
                if pdf_generated {
                    info!("[CHITANTE][SAVE] Saved receipt PDF at {}", pdf_path);
                } else {
                    warn!("[CHITANTE][SAVE] Could not generate receipt PDF, HTML is available at {}", html_path);
                }
                return Ok((html_path, pdf_path, pdf_generated));
            }
            Err(e) => {
                failures.push(format!("write {}: {}", html_file_path.display(), e));
            }
        }
    }

    Err(format!(
        "Nu am putut salva chitanța local. Erori: {}",
        failures.join(" | ")
    ))
}

pub fn get_partner_receipt_info(
    conn: &rusqlite::Connection,
    partner_id: &str,
) -> (Option<String>, Option<String>, Option<String>, Option<String>, Option<String>) {
    conn.query_row(
        r#"
        SELECT
            p.cif,
            p.reg_com,
            l.address,
            l.localitate,
            l.judet
        FROM partners p
        LEFT JOIN locations l ON l.partner_id = p.id
        WHERE p.id = ?1
        ORDER BY
            CASE
                WHEN IFNULL(l.inactiv, 'NU') IN ('DA', '1', 'true', 'TRUE') THEN 1
                ELSE 0
            END,
            l.id
        LIMIT 1
        "#,
        [partner_id],
        |row| {
            Ok((
                row.get::<_, Option<String>>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        },
    )
    .unwrap_or((None, None, None, None, None))
}

// ==================== SYNC COMMANDS ====================

