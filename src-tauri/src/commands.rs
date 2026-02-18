use crate::api_client;
use crate::database::Database;
use crate::models::*;
use crate::print_invoice;
use crate::print_daily_report;
use crate::print_receipt;
use chrono::{Utc, Datelike, Local};
use log::{info, warn};
use tauri::State;
use uuid::Uuid;
use rusqlite::params;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

// Helper function to read logo and convert to base64
fn read_logo_to_base64() -> Option<String> {
    // Embed logo at compile time so it works in packaged builds
    let logo_data: &[u8] = include_bytes!("../../public/logo.png");
    if logo_data.is_empty() {
        return None;
    }

    use base64::{Engine as _, engine::general_purpose};
    let base64_string = general_purpose::STANDARD.encode(logo_data);
    Some(format!("data:image/png;base64,{}", base64_string))
}

fn parse_price(value: &Option<String>) -> Option<f64> {
    value.as_ref().and_then(|s| s.replace(',', ".").parse::<f64>().ok())
}

fn compute_due_date(created_at_rfc3339: &str, payment_term_days: Option<&str>) -> Result<String, String> {
    let created_at = chrono::DateTime::parse_from_rfc3339(created_at_rfc3339)
        .map_err(|e| format!("Failed to parse invoice date: {}", e))?;

    let days = payment_term_days
        .and_then(|value| value.trim().parse::<i64>().ok())
        .filter(|value| *value >= 0)
        .unwrap_or(30);

    Ok((created_at + chrono::Duration::days(days))
        .format("%d.%m.%Y")
        .to_string())
}

fn normalize_opt_key(value: &Option<String>) -> String {
    value
        .as_ref()
        .map(|v| v.trim().to_string())
        .unwrap_or_default()
}

fn build_invoice_key(id_partener: &str, serie_factura: &Option<String>, numar_factura: &Option<String>, cod_document: &Option<String>) -> String {
    format!(
        "{}|{}|{}|{}",
        id_partener.trim(),
        normalize_opt_key(serie_factura),
        normalize_opt_key(numar_factura),
        normalize_opt_key(cod_document)
    )
}

fn get_receipt_series(conn: &rusqlite::Connection) -> Result<String, String> {
    let (receipt_series_opt, carnet_series_opt): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT receipt_series, carnet_series FROM agent_settings WHERE id = 1",
            [],
            |row| Ok((row.get(0).ok(), row.get(1).ok()))
        )
        .map_err(|e| e.to_string())?;

    Ok(receipt_series_opt
        .filter(|s| !s.trim().is_empty())
        .or(carnet_series_opt)
        .unwrap_or_else(|| "CH".to_string()))
}

fn generate_receipt_number(conn: &rusqlite::Connection) -> Result<String, String> {
    let (current, end): (Option<i64>, Option<i64>) = conn.query_row(
        "SELECT receipt_number_current, receipt_number_end FROM agent_settings WHERE id = 1",
        [],
        |row| Ok((row.get(0).ok(), row.get(1).ok()))
    ).map_err(|e| e.to_string())?;

    if let Some(val) = current {
        // Check end limit if set
        if let Some(limit) = end {
            if val > limit {
                 return Err(format!("S-a atins limita de numere pentru chitanțe ({})", limit));
            }
        }
        
        // Update DB with next value
        let next_val = val + 1;
        conn.execute("UPDATE agent_settings SET receipt_number_current = ?1 WHERE id = 1", [next_val])
            .map_err(|e| e.to_string())?;
            
        Ok(val.to_string())
    } else {
        // Fallback to timestamp if not configured
        Ok(chrono::Local::now().format("%Y%m%d%H%M%S").to_string())
    }
}

fn map_product_row(row: &rusqlite::Row) -> rusqlite::Result<Product> {
    // Parse TVA percentage from TEXT to f64
    let tva_percent: Option<f64> = match row.get::<_, Option<String>>(5)? {
        Some(s) => s.parse::<f64>().ok(),
        None => None,
    };
    
    Ok(Product {
        id: row.get(0)?,
        name: row.get(1)?,
        unit_of_measure: row.get(2)?,
        price: row.get(3)?,
        class: row.get(4)?,
        tva_percent,
    })
}

fn wait_for_file_ready(path: &str, timeout_ms: u64, stable_ms: u64) -> bool {
    let start = std::time::Instant::now();
    let mut last_size: Option<u64> = None;
    let mut stable_for = 0u64;

    while start.elapsed().as_millis() as u64 <= timeout_ms {
        if let Ok(metadata) = std::fs::metadata(path) {
            let size = metadata.len();
            if size > 0 {
                if Some(size) == last_size {
                    stable_for += 100;
                    if stable_for >= stable_ms {
                        return true;
                    }
                } else {
                    last_size = Some(size);
                    stable_for = 0;
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    false
}

fn try_generate_pdf_from_html(html_path_str: &str, pdf_path_str: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        let edge_paths = vec![
            "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
            "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe",
        ];

        for edge_path in edge_paths {
            if std::path::Path::new(edge_path).exists() {
                let file_url = format!(
                    "file:///{}",
                    html_path_str.replace('\\', "/").replace(' ', "%20")
                );

                let temp_dir = std::env::temp_dir().join("esoft_edge_pdf");
                let _ = std::fs::create_dir_all(&temp_dir);
                let user_data_arg = format!("--user-data-dir={}", temp_dir.to_string_lossy());
                let print_arg = format!("--print-to-pdf={}", pdf_path_str);
                info!("[CERT][PDF] Generating PDF: {}", pdf_path_str);

                let output = std::process::Command::new(edge_path)
                    .args(&[
                        "--headless",
                        "--disable-gpu",
                        "--no-sandbox",
                        "--disable-dev-shm-usage",
                        &user_data_arg,
                        &print_arg,
                        &file_url,
                    ])
                    .output();

                if let Ok(result) = output {
                    info!("[CERT][PDF] Edge status: {}, stderr: {}", result.status, String::from_utf8_lossy(&result.stderr));
                    let mut waited = 0;
                    while waited < 6000 {
                        if wait_for_file_ready(pdf_path_str, 1200, 400) {
                            info!("[CERT][PDF] PDF generated OK");
                            return true;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        waited += 100;
                    }
                    info!("[CERT][PDF] PDF not ready after 6s");
                } else {
                    info!("[CERT][PDF] Edge exec failed");
                }
            }
        }
    }

    false
}

fn get_receipts_dirs_to_try() -> Vec<PathBuf> {
    let mut dirs_to_try = Vec::new();

    if let Some(path) = dirs::config_dir() {
        dirs_to_try.push(path.join("facturi.softconsulting.com").join("receipts"));
    }

    if let Some(path) = dirs::document_dir() {
        dirs_to_try.push(path.join("facturi.softconsulting.com").join("receipts"));
    }

    if let Some(path) = dirs::data_dir() {
        dirs_to_try.push(path.join("facturi.softconsulting.com").join("receipts"));
    }

    if let Ok(path) = std::env::current_dir() {
        dirs_to_try.push(path.join("receipts"));
    }

    dirs_to_try
}

fn save_receipt_html_file(
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
) -> Result<(String, String), String> {
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

    for dir in get_receipts_dirs_to_try() {
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
                return Ok((html_path, pdf_path));
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

fn generate_quality_certificate_html() -> String {
    use base64::{engine::general_purpose, Engine as _};

    let epc_img = general_purpose::STANDARD.encode(include_bytes!("../../public/EPC 16 EC.png"));
    let iso_img = general_purpose::STANDARD.encode(include_bytes!("../../public/KARIN-ISO.png"));
    let stamp_img = general_purpose::STANDARD.encode(include_bytes!("../../public/STAMPILA.png"));
    let cert_date = Local::now().format("%d.%m.%Y").to_string();

    format!(
        r#"<!DOCTYPE html>
<html lang="ro">
<head>
    <meta charset="UTF-8" />
    <title>Certificat de calitate - Declarație de conformitate</title>
    <style>
        @page {{ size: 80mm 297mm; margin: 2.5mm; }}
        body {{ font-family: Arial, Helvetica, sans-serif; margin: 0; padding: 0; color: #000; font-weight: 700; }}
        .page {{ width: 68mm; margin-left: 0.5mm; box-sizing: border-box; padding: 0.5mm; }}
        .header {{ margin-bottom: 3px; text-align: center; border-bottom: 2px solid #000; padding-bottom: 2px; }}
        .header-line {{ font-size: 10.5px; font-weight: 800; line-height: 1.2; }}
        .header-line + .header-line {{ margin-top: 0; }}
        .logos {{ display: flex; justify-content: center; align-items: center; gap: 8px; margin: 3px 0; }}
        .logo-box {{ width: calc(50% - 4px); display: flex; align-items: center; justify-content: center; }}
        .logo-img {{ max-width: 100%; height: auto; object-fit: contain; }}
        .title {{ text-align: center; font-size: 13px; font-weight: 900; margin: 3px 0 1px; text-decoration: underline; text-transform: uppercase; }}
        .date {{ text-align: center; font-size: 10.5px; font-weight: 800; margin-top: 1px; }}
        .cert-subtitle {{ text-align: center; font-size: 12px; font-weight: 800; margin-top: 1px; }}
        .cert-intro {{ margin-top: 3px; font-size: 10.5px; line-height: 1.2; text-align: left; font-weight: 800; }}
        .cat-grid {{ display: grid; grid-template-columns: 1fr; gap: 0; margin-top: 3px; }}
        .cat-group {{ width: 100%; text-align: left; margin: 0 0 4px 0; }}
        .cat-group:last-child {{ margin-bottom: 2px; }}
        .cat-line {{ font-size: 10px; line-height: 1.2; font-weight: 700; white-space: normal; overflow-wrap: anywhere; margin: 0; text-align: left; }}
        .cert-body {{ margin-top: 3px; font-size: 9.5px; line-height: 1.2; text-align: left; font-weight: 700; }}
        .cert-body p {{ margin: 0 0 1px 0; }}
        .cert-footer {{ display: flex; justify-content: space-between; align-items: flex-start; margin-top: 4px; font-size: 9.5px; font-weight: 700; }}
        .footer-col {{ width: 48%; }}
        .footer-right {{ text-align: right; padding-right: 5mm; }}
        .stamp-section {{ text-align: center; margin-top: 4px; }}
        .footer-stamp {{ width: 100px; height: auto; object-fit: contain; }}
    </style>
</head>
<body>
    <div class="page">
        <div class="header">
            <div class="header-line">PO 7.5-03-F01Rev. 8/12012021</div>
            <div class="header-line">SC KARIN SRL</div>
            <div class="header-line">J24/380/1994, SEINI, N.BALCESCU, 43</div>
            <div class="header-line">Jud. MM, Tel. 0262-491317</div>
        </div>

        <div class="logos">
            <div class="logo-box">
                <img src="data:image/png;base64,{}" class="logo-img" alt="EPC 16 EC" />
            </div>
            <div class="logo-box">
                <img src="data:image/png;base64,{}" class="logo-img" alt="KARIN ISO" />
            </div>
        </div>

        <div class="title">Certificat de calitate - Declarație de conformitate</div>
        <div class="date">Data: {}</div>
        <div class="cert-subtitle">Nr.033 din data de 05.02.2026</div>

        <div class="cert-intro">
            În conformitate cu prevederile legale privind răspunderea,
            se atestă calitatea produselor livrate: ouă consum categoria A,
            cu data ouatului:
        </div>

        <div class="cat-grid">
            <div class="cat-group">
                <div class="cat-line">Cat. S (&lt;53g) 04.02.26 ddm 04.03.26 Lot 035 S</div>
                <div class="cat-line">Cat. S (&lt;53g) ________ ddm ________ Lot ____ S</div>
            </div>
            <div class="cat-group">
                <div class="cat-line">Cat. L (63-73g) 02.02.26 ddm 02.03.26 Lot 033 L</div>
                <div class="cat-line">Cat. L (63-73g) 04.02.26 ddm 04.03.26 Lot 035 L</div>
                <div class="cat-line">Cat. L (63-73g) ________ ddm ________ Lot ____ L</div>
            </div>
            <div class="cat-group">
                <div class="cat-line">Cat. M (53-63g) 02.02.26 ddm 02.03.26 Lot 033 M</div>
                <div class="cat-line">Cat. M (53-63g) 04.02.26 ddm 04.03.26 Lot 035 M</div>
                <div class="cat-line">Cat. M (53-63g) ________ ddm ________ Lot ____ M</div>
            </div>
            <div class="cat-group">
                <div class="cat-line">Cat. XL (&gt;73g) 02.02.26 ddm 02.03.26 Lot 033 XL</div>
                <div class="cat-line">Cat. XL (&gt;73g) 04.02.26 ddm 04.03.26 Lot 035 XL</div>
                <div class="cat-line">Cat. XL (&gt;73g) ________ ddm ________ Lot ____ XL</div>
            </div>
        </div>

        <div class="cert-body">
            <p>Ambalate la data de 05.02.2026. Livrate beneficiarului: Rețea Magazine. Conform facturii/avizului nr. ______ din 05.02.2026.</p>
            <p>Transport auto: MM44KRN, MM66KRN, MM56KAR, MM99KRN sau ______ indeplinesc parametri de calitate specificati conform BAnr.77/29.01.2026(salmonella negativ).</p>
            <p>Caracteristici tehnice de livrare: SALUBRE; Rasa LOHMANN BROWN, LOHMANN SANDY; Aspectul cojii intactă, curată de formă normală, uscată;</p> 
            <p>Camera de aer: imobilă, cu înălțimea maximă 5 mm. Albușul: clar, translucid, consistență gelatinoasă si lipsit de corpuri străine de orice natura.</p> 
            <p>Gălbenuș vizibil, în fascicol de lumina sub formă de umbră. Mirosul și gust caracteristic oului proaspăt, fără miros și gust străin.</p>
            <p>Data durabilității minime este de 28 zile iar data recomandata pentru vanzare este de 28 de zile de la momentul ouatului.</p>
            <p>Temperatura de păstrare: 5-18 grade Celsius,În magazine, ferite de razele soarelui si sursa de caldura.</p>
            <p>In magazinele de desfacere, ouale se pastreaza in locuri racoroase, curate, ferite de alte produse ale caror miros le pot imprumuta.</p>
            <p>Produs fragil! A se manipula cu atenție la transport și depozitare.</p> 
            <p>Prezentul certificat întocmit conform Reg.(CE) nr.1234/22.10.2007 de instituire a unei organizari comune a pietelor agricole si privind</p>
            <p>dispozitii specifice referitoare la anumite produse agricole ("Regulamentul unic OCP"). Regulamentul (CE)NR.589/2008 al Comisiei din 23.06.2008</p>
            <p>de stabilire a noremlor de aplicare a Reg.(CE)nr.1234/2007 al Consiliului privind standardele de comercializare a oualelor, modificat de Regulamentul </p>
            <p>CE 598/2008. Mentionam ca ouale produse de noi cu cod pro.3RO MM 013 provin de la gaini crescute in custi imbunatatie si cu cod producator</p>
            <p>2RO MM 040 provin de la gaini cresute in sistem volier. conform standardelor U.E. in vigoare. </p>
        </div>

        <div class="cert-footer">
            <div class="footer-col">
                Administrator,<br>
                Dr. Meseșan Dan
            </div>
            <div class="footer-col footer-right">
                Țara de origine:România<br>
                Cod stație sortare RO MM 023<br>
                Cod producător 3RO MM 013<br>
                Cod producător 2RO MM 040<br>
                Șef compartiment
            </div>
        </div>

        <div class="stamp-section">
            <img src="data:image/png;base64,{}" class="footer-stamp" alt="Ștampilă" />
        </div>
    </div>
</body>
</html>"#,
        epc_img,
        iso_img,
        cert_date,
        stamp_img,
    )
}

fn save_invoice_certificate_file(invoice_id: &str) -> Result<(String, String, String), String> {
    let html = generate_quality_certificate_html();

    let app_data_dir = dirs::config_dir()
        .ok_or("Could not find app data directory")?
        .join("facturi.softconsulting.com")
        .join("invoices")
        .join("certificates");

    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create certificates directory: {}", e))?;

    let html_file_path = app_data_dir.join(format!("certificat_{}.html", invoice_id));
    let pdf_file_path = app_data_dir.join(format!("certificat_{}.pdf", invoice_id));

    std::fs::write(&html_file_path, &html)
        .map_err(|e| format!("Failed to write certificate HTML file: {}", e))?;

    let html_path = html_file_path.to_string_lossy().to_string();
    let pdf_path = pdf_file_path.to_string_lossy().to_string();
    let pdf_generated = try_generate_pdf_from_html(&html_path, &pdf_path);
    let print_target = if pdf_generated { pdf_path.clone() } else { html_path.clone() };

    Ok((html_path, pdf_path, print_target))
}

fn get_partner_receipt_info(
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

#[tauri::command]
pub fn clear_database(db: State<'_, Database>) -> Result<(), String> {
    db.clear_sync_data().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_partners_and_locations(db: State<'_, Database>) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute("PRAGMA foreign_keys = ON", [])
        .map_err(|e| e.to_string())?;

    let total_partners: i64 = conn
        .query_row("SELECT COUNT(*) FROM partners", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let total_locations: i64 = conn
        .query_row("SELECT COUNT(*) FROM locations", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let protected_partners: i64 = conn
        .query_row(
            r#"
            SELECT COUNT(DISTINCT p.id)
            FROM partners p
            WHERE EXISTS (
                SELECT 1 FROM invoices i WHERE i.partner_id = p.id
            )
            OR EXISTS (
                SELECT 1
                FROM locations l
                JOIN invoices i ON i.location_id = l.id
                WHERE l.partner_id = p.id
            )
            "#,
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let deleted_partners = conn
        .execute(
            r#"
            DELETE FROM partners
            WHERE id IN (
                SELECT p.id
                FROM partners p
                WHERE NOT EXISTS (
                    SELECT 1 FROM invoices i WHERE i.partner_id = p.id
                )
                AND NOT EXISTS (
                    SELECT 1
                    FROM locations l
                    JOIN invoices i ON i.location_id = l.id
                    WHERE l.partner_id = p.id
                )
            )
            "#,
            [],
        )
        .map_err(|e| e.to_string())? as i64;

    let remaining_locations: i64 = conn
        .query_row("SELECT COUNT(*) FROM locations", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let deleted_locations = total_locations - remaining_locations;

    Ok(format!(
        "Ștergere finalizată: {} parteneri și {} sedii șterse. {} parteneri păstrați deoarece au facturi asociate.",
        deleted_partners,
        deleted_locations,
        protected_partners.max(total_partners - deleted_partners)
    ))
}

#[tauri::command]
pub fn check_first_run(db: State<'_, Database>) -> Result<bool, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM partners", [], |row| row.get(0))
        .unwrap_or(0);

    Ok(count == 0)
}

#[tauri::command]
pub fn get_sync_status(db: State<'_, Database>) -> Result<SyncStatus, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let partners_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM partners", [], |row| row.get(0))
        .unwrap_or(0);

    let is_first_run = partners_count == 0;

    let partners_synced_at: Option<String> = conn
        .query_row(
            "SELECT last_synced_at FROM sync_metadata WHERE entity_type = 'partners'",
            [],
            |row| row.get(0),
        )
        .ok();

    let products_synced_at: Option<String> = conn
        .query_row(
            "SELECT last_synced_at FROM sync_metadata WHERE entity_type = 'products'",
            [],
            |row| row.get(0),
        )
        .ok();

    Ok(SyncStatus {
        is_first_run,
        partners_synced_at,
        products_synced_at,
        is_syncing: false,
    })
}

#[tauri::command]
pub async fn sync_all_data(db: State<'_, Database>) -> Result<SyncStatus, String> {
    // Try real API first
    // Try real API first - Strict mode for Release (No Mock Data)
    let api = api_client::ApiClient::from_default().map_err(|e| format!("Failed to initialize API: {}", e))?;

    let agent_settings = get_agent_settings(db.clone())?;
    let marca_agent = agent_settings
        .marca_agent
        .and_then(|value| {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });

    if let Some(marca) = &marca_agent {
        info!("Sync partners with MarcaAgent filter: {}", marca);
    } else {
        info!("MarcaAgent not set; syncing all AGENTI partners");
    }

    // Get full partners list via GET, then apply all filters locally
    let api_partners = api.get_partners_full_get().await.map_err(|e| format!("Failed to fetch partners: {}", e))?;
                    
    // Try to get articles from API
    let api_articles = api.get_all_articles().await.map_err(|e| format!("Failed to fetch products: {}", e))?;

    // Try to get offers from API (active for today)
    let today = Utc::now().format("%d.%m.%Y").to_string();
    let offers_list = api
        .get_offers(api_client::OfferFilter {
            data_referinta: None,
            data_analiza: Some(today),
            cod_partener: None,
            furnizori: None,
            cod_subunit: None,
        })
        .await
        .ok()
        .map(|resp| resp.info_oferte)
        .unwrap_or_default();

    // Convert API data to our models
    let partners = convert_api_partners_to_model(api_partners, marca_agent.clone());
    let products = convert_api_articles_to_model(api_articles);
    let offers = Some(offers_list);

    // Now do all database operations synchronously
    let now = Utc::now().to_rfc3339();

    // Use inner scope to ensure MutexGuard is dropped before any potential await
    let result = {
        let conn = db.conn.lock().map_err(|e| format!("Failed to lock database: {}", e))?;

        // Temporarily disable foreign key checks during sync
        // This is needed because INSERT OR REPLACE does DELETE + INSERT which can violate FK constraints
        conn.execute("PRAGMA foreign_keys = OFF", [])
            .map_err(|e| format!("Failed to disable foreign keys: {}", e))?;

        // Save partners
        for partner in &partners {
            conn.execute(
                "INSERT OR REPLACE INTO partners (id, name, cif, reg_com, cod, blocat, tva_la_incasare, persoana_fizica, cod_extern, cod_intern, observatii, data_adaugarii, created_at, updated_at, clasa, simbol_clasa, cod_clasa, categorie_pret_implicita, simbol_categorie_pret, scadenta_la_vanzare, scadenta_la_cumparare, discount_fix, tip_partener, mod_aplicare_discount, moneda, data_nastere, caracterizare_contabila_denumire, caracterizare_contabila_simbol) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28)",
                params![
                    &partner.id, 
                    &partner.name, 
                    &partner.cif, 
                    &partner.reg_com, 
                    &partner.cod,
                    &partner.blocat,
                    &partner.tva_la_incasare,
                    &partner.persoana_fizica,
                    &partner.cod_extern,
                    &partner.cod_intern,
                    &partner.observatii,
                    &partner.data_adaugarii,
                    &partner.created_at, 
                    &partner.updated_at,
                    &partner.clasa,
                    &partner.simbol_clasa,
                    &partner.cod_clasa,
                    &partner.categorie_pret_implicita,
                    &partner.simbol_categorie_pret,
                    &partner.scadenta_la_vanzare,
                    &partner.scadenta_la_cumparare,
                    &partner.discount_fix,
                    &partner.tip_partener,
                    &partner.mod_aplicare_discount,
                    &partner.moneda,
                    &partner.data_nastere,
                    &partner.caracterizare_contabila_denumire,
                    &partner.caracterizare_contabila_simbol,
                ],
            )
            .map_err(|e| format!("Failed to save partner: {}", e))?;

            conn.execute(
                "DELETE FROM locations WHERE partner_id = ?1",
                params![&partner.id],
            )
            .map_err(|e| format!("Failed to clear partner locations: {}", e))?;

            // Save locations
            for location in &partner.locations {
                conn.execute(
                    "INSERT OR REPLACE INTO locations (id, partner_id, name, address, cod_sediu, localitate, strada, numar, judet, tara, cod_postal, telefon, email, inactiv) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                    (
                        &location.id, 
                        &location.partner_id, 
                        &location.name, 
                        &location.address,
                        &location.cod_sediu,
                        &location.localitate,
                        &location.strada,
                        &location.numar,
                        &location.judet,
                        &location.tara,
                        &location.cod_postal,
                        &location.telefon,
                        &location.email,
                        &location.inactiv,
                    ),
                )
                .map_err(|e| format!("Failed to save location: {}", e))?;
            }
        }

        // Save products
        for product in &products {
            // Convert Option<f64> to Option<String> for database storage
            let tva_str = product.tva_percent.map(|t| t.to_string());
            
            conn.execute(
                "INSERT INTO products (id, name, unit_of_measure, price, class, procent_tva) VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
                 ON CONFLICT(id) DO UPDATE SET name = excluded.name, unit_of_measure = excluded.unit_of_measure, price = excluded.price, class = excluded.class, procent_tva = excluded.procent_tva",
                (&product.id, &product.name, &product.unit_of_measure, product.price, &product.class, &tva_str),
            )
            .map_err(|e| format!("Failed to save product: {}", e))?;
        }

        // Save offers (only if fetched)
        if let Some(offers) = &offers {
            conn.execute("DELETE FROM offer_items", [])
                .map_err(|e| format!("Failed to clear offer items: {}", e))?;
            conn.execute("DELETE FROM offers", [])
                .map_err(|e| format!("Failed to clear offers: {}", e))?;

            for offer in offers {
                let id_client = offer.id_client.clone().unwrap_or_default();
                let numar = offer.numar.clone().unwrap_or_default();
                let offer_id = format!("{}-{}", id_client, numar);

                conn.execute(
                    "INSERT OR REPLACE INTO offers (id, id_client, numar, data_inceput, data_sfarsit, anulata, client, tip_oferta, furnizor, id_furnizor, cod_fiscal, simbol_clasa, moneda, observatii, extensie_document) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                    params![
                        &offer_id,
                        &id_client,
                        &offer.numar,
                        &offer.data_inceput,
                        &offer.data_sfarsit,
                        &offer.anulata,
                        &offer.client,
                        &offer.tip_oferta,
                        &offer.furnizor,
                        &offer.id_furnizor,
                        &offer.cod_fiscal,
                        &offer.simbol_clasa,
                        &offer.moneda,
                        &offer.observatii,
                        &offer.extensie_document,
                    ],
                )
                .map_err(|e| format!("Failed to save offer: {}", e))?;

                if let Some(items) = &offer.items {
                    for item in items {
                        let price = parse_price(&item.pret);
                        conn.execute(
                            "INSERT INTO offer_items (offer_id, id_client, product_id, denumire, um, cant_minima, cant_maxima, cant_optima, pret, discount, proc_adaos, pret_ref, pret_cu_proc_adaos, observatii, cod_oferta1, extensie_linie) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                            params![
                                &offer_id,
                                &id_client,
                                &item.id,
                                &item.denumire,
                                &item.um,
                                &item.cant_minima,
                                &item.cant_maxima,
                                &item.cant_optima,
                                price,
                                &item.discount,
                                &item.proc_adaos,
                                &item.pret_ref,
                                &item.pret_cu_proc_adaos,
                                &item.observatii,
                                &item.cod_oferta1,
                                &item.extensie_linie,
                            ],
                        )
                        .map_err(|e| format!("Failed to save offer item: {}", e))?;
                    }
                }
            }
        }

        // Update sync metadata
        conn.execute(
            "INSERT OR REPLACE INTO sync_metadata (entity_type, last_synced_at) VALUES ('partners', ?1)",
            [&now],
        )
        .map_err(|e| format!("Failed to update sync metadata: {}", e))?;

        conn.execute(
            "INSERT OR REPLACE INTO sync_metadata (entity_type, last_synced_at) VALUES ('products', ?1)",
            [&now],
        )
        .map_err(|e| format!("Failed to update sync metadata: {}", e))?;

        // Re-enable foreign key checks
        conn.execute("PRAGMA foreign_keys = ON", [])
            .map_err(|e| format!("Failed to re-enable foreign keys: {}", e))?;

        info!(
            "Sync completed: {} partners, {} products",
            partners.len(),
            products.len()
        );

        // Get fresh status (same lock, no need to re-acquire)
        let partners_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM partners", [], |row| row.get(0))
            .unwrap_or(0);

        let partners_synced_at: Option<String> = conn
            .query_row(
                "SELECT last_synced_at FROM sync_metadata WHERE entity_type = 'partners'",
                [],
                |row| row.get(0),
            )
            .ok();

        let products_synced_at: Option<String> = conn
            .query_row(
                "SELECT last_synced_at FROM sync_metadata WHERE entity_type = 'products'",
                [],
                |row| row.get(0),
            )
            .ok();

        Ok(SyncStatus {
            is_first_run: partners_count == 0,
            partners_synced_at,
            products_synced_at,
            is_syncing: false,
        })
    };

    // Auto-process pending invoices if we have internet (implied by sync)
    // We just try - if it fails it stays pending.
    let pending_invoices: Vec<String> = {
        match db.conn.lock() {
            Ok(conn) => {
                match conn.prepare("SELECT id FROM invoices WHERE status = 'pending'") {
                    Ok(mut stmt) => {
                         stmt.query_map([], |row| row.get(0))
                            .map(|rows| rows.filter_map(|r| r.ok()).collect())
                            .unwrap_or_default()
                    },
                    Err(_) => Vec::new(),
                }
            },
            Err(_) => Vec::new(),
        }
    };

    if !pending_invoices.is_empty() {
        info!("Found {} pending invoices. Attempting to auto-send...", pending_invoices.len());
        for id in pending_invoices {
            info!("Auto-sending invoice: {}", id);
            // We ignore errors here as send_invoice handles logging and status updates
            let _ = send_invoice(db.clone(), id).await;
        }
    }

    result
}

// Convert API partners to our internal model
fn convert_api_partners_to_model(
    api_partners: Vec<api_client::PartnerInfo>,
    marca_agent: Option<String>,
) -> Vec<PartnerWithLocations> {
    let normalized_marca = marca_agent
        .and_then(|value| {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });

    api_partners
        .into_iter()
        .filter(|api_partner| {
            let clasa = api_partner.clasa.as_deref().unwrap_or("").trim().to_uppercase();
            let simbol_clasa = api_partner.simbol_clasa.as_deref().unwrap_or("").trim().to_uppercase();
            clasa == "AGENTI" || simbol_clasa == "AGENTI"
        })
        .filter_map(|api_partner| {
            // Generate ID if empty - use COD or CIF or UUID as fallback
            let partner_id = if api_partner.id.is_empty() {
                api_partner.cod.clone()
                    .filter(|c| !c.is_empty())
                    .or_else(|| api_partner.cod_fiscal.clone().filter(|c| !c.is_empty()))
                    .unwrap_or_else(|| Uuid::new_v4().to_string())
            } else {
                api_partner.id.clone()
            };
            
            let now = Utc::now().to_rfc3339();

            let mut seen_location_keys: HashSet<String> = HashSet::new();

            // Convert locations with all fields
            let locations: Vec<Location> = api_partner.sedii
                .into_iter()
                .filter(|sediu| {
                    let inactiv = sediu.inactiv.as_deref().unwrap_or("").trim().to_uppercase();
                    inactiv != "DA"
                })
                .filter(|sediu| {
                    if let Some(expected_marca) = &normalized_marca {
                        let sediu_marca = sediu
                            .agent
                            .as_ref()
                            .and_then(|agent| agent.marca.as_ref())
                            .map(|marca| marca.trim());

                        matches!(sediu_marca, Some(value) if value == expected_marca)
                    } else {
                        true
                    }
                })
                .filter_map(|sediu| {
                    let dedupe_key = if !sediu.id_sediu.trim().is_empty() {
                        format!("id:{}", sediu.id_sediu.trim())
                    } else if let Some(cod) = &sediu.cod_sediu {
                        let cod_trimmed = cod.trim();
                        if !cod_trimmed.is_empty() {
                            format!("cod:{}", cod_trimmed)
                        } else {
                            format!("den:{}:{}", sediu.denumire.trim(), sediu.localitate.as_deref().unwrap_or("").trim())
                        }
                    } else {
                        format!("den:{}:{}", sediu.denumire.trim(), sediu.localitate.as_deref().unwrap_or("").trim())
                    };

                    if !seen_location_keys.insert(dedupe_key) {
                        return None;
                    }

                    let base_location_id = if sediu.id_sediu.is_empty() {
                        sediu.cod_sediu
                            .clone()
                            .filter(|c| !c.is_empty())
                            .unwrap_or_else(|| Uuid::new_v4().to_string())
                    } else {
                        sediu.id_sediu.clone()
                    };

                    // Keep location IDs unique across all partners to avoid cross-partner overwrite
                    let location_id = format!("{}:{}", partner_id, base_location_id);
                    
                    // Build address smartly: if we have street info, include it; otherwise just city and county
                    let address = if sediu.strada.is_some() || sediu.numar.is_some() {
                        // Has street info - build full address
                        let mut parts = Vec::new();
                        
                        // Combine street and number
                        let mut street_part = String::new();
                        if let Some(strada) = &sediu.strada {
                            if !strada.trim().is_empty() {
                                street_part.push_str(strada.trim());
                            }
                        }
                        if let Some(numar) = &sediu.numar {
                            if !numar.trim().is_empty() {
                                if !street_part.is_empty() {
                                    street_part.push_str(" ");
                                }
                                street_part.push_str(numar.trim());
                            }
                        }
                        if !street_part.is_empty() {
                            parts.push(street_part);
                        }
                        
                        // Add city
                        if let Some(localitate) = &sediu.localitate {
                            if !localitate.trim().is_empty() {
                                parts.push(localitate.trim().to_string());
                            }
                        }
                        
                        // Add county
                        if let Some(judet) = &sediu.judet {
                            if !judet.trim().is_empty() {
                                parts.push(judet.trim().to_string());
                            }
                        }
                        
                        parts.join(", ")
                    } else {
                        // No street info - just show city and county
                        let mut parts = Vec::new();
                        
                        if let Some(localitate) = &sediu.localitate {
                            if !localitate.trim().is_empty() {
                                parts.push(localitate.trim().to_string());
                            }
                        }
                        
                        if let Some(judet) = &sediu.judet {
                            if !judet.trim().is_empty() {
                                parts.push(judet.trim().to_string());
                            }
                        }
                        
                        parts.join(", ")
                    };

                    Some(Location {
                        id: location_id,
                        partner_id: partner_id.clone(),
                        name: sediu.denumire,
                        address: Some(address),
                        cod_sediu: sediu.cod_sediu,
                        localitate: sediu.localitate,
                        strada: sediu.strada,
                        numar: sediu.numar,
                        judet: sediu.judet,
                        tara: sediu.tara,
                        cod_postal: sediu.cod_postal,
                        telefon: sediu.telefon,
                        email: sediu.email,
                        inactiv: sediu.inactiv,
                    })
                })
                .collect();

            if normalized_marca.is_some() && locations.is_empty() {
                return None;
            }

            Some(PartnerWithLocations {
                id: partner_id,
                name: api_partner.denumire,
                cif: api_partner.cod_fiscal,
                reg_com: api_partner.registru_comert,
                cod: api_partner.cod,
                blocat: api_partner.blocat,
                tva_la_incasare: api_partner.tva_la_incasare,
                persoana_fizica: api_partner.persoana_fizica,
                cod_extern: api_partner.cod_extern,
                cod_intern: api_partner.cod_intern,
                observatii: api_partner.observatii,
                data_adaugarii: api_partner.data_adaugarii.clone(),
                created_at: api_partner.data_adaugarii.unwrap_or(now.clone()),
                updated_at: now,
                clasa: api_partner.clasa,
                simbol_clasa: api_partner.simbol_clasa,
                cod_clasa: api_partner.cod_clasa,
                inactiv: api_partner.inactiv,
                categorie_pret_implicita: api_partner.categorie_pret_implicita,
                simbol_categorie_pret: api_partner.simbol_categorie_pret,
                scadenta_la_vanzare: Some("30".to_string()),
                scadenta_la_cumparare: api_partner.scadenta_la_cumparare,
                credit_client: api_partner.credit_client,
                discount_fix: api_partner.discount_fix,
                tip_partener: api_partner.tip_partener,
                mod_aplicare_discount: api_partner.mod_aplicare_discount,
                moneda: api_partner.moneda,
                data_nastere: api_partner.data_nastere,
                caracterizare_contabila_denumire: api_partner.caracterizare_contabila_denumire,
                caracterizare_contabila_simbol: api_partner.caracterizare_contabila_simbol,
                locations,
            })
        })
        .collect()
}

// Convert API articles to our internal model
fn convert_api_articles_to_model(api_articles: Vec<api_client::ArticleInfo>) -> Vec<Product> {
    api_articles
        .into_iter()
        .map(|api_article| {
            // Generate ID if empty - use CodObiect or UUID as fallback
            let product_id = if api_article.id.is_empty() {
                api_article.cod_obiect.clone()
                    .filter(|c| !c.is_empty())
                    .unwrap_or_else(|| Uuid::new_v4().to_string())
            } else {
                api_article.id.clone()
            };
            
            // Parse price from string
            let price = parse_price(&api_article.pret_vanzare).unwrap_or(0.0);
            
            // Parse TVA percentage from string
            let tva_percent = match &api_article.procent_tva {
                Some(tva_str) => tva_str.parse::<f64>().ok(),
                None => None,
            };

            Product {
                id: product_id,
                name: api_article.denumire,
                unit_of_measure: api_article.um,
                price,
                class: api_article.clasa,
                tva_percent,
            }
        })
        .collect()
}

#[tauri::command]
pub fn check_online_status() -> Result<bool, String> {
    // For now, always return true - the frontend handles online/offline via navigator.onLine
    Ok(true)
}

// ==================== API TEST COMMANDS ====================

#[tauri::command]
pub async fn test_api_partners() -> Result<String, String> {
    info!("Testing API GET partners...");
    
    match api_client::ApiClient::from_default() {
        Ok(api) => {
            match api.get_all_partners(None).await {
                Ok(partners) => {
                    let count = partners.len();
                    info!("Successfully fetched {} partners from API", count);
                    
                    // Return summary with first 3 partners
                    let summary = if count > 0 {
                        let sample: Vec<String> = partners.iter()
                            .take(3)
                            .map(|p| format!("{} (CIF: {})", p.denumire, p.cod_fiscal.as_deref().unwrap_or("N/A")))
                            .collect();
                        format!("✅ Success! Found {} partners.\n\nSample:\n{}", count, sample.join("\n"))
                    } else {
                        "✅ API works but returned 0 partners".to_string()
                    };
                    
                    Ok(summary)
                }
                Err(e) => {
                    let error_msg = format!("❌ API call failed: {}", e);
                    info!("{}", error_msg);
                    Err(error_msg)
                }
            }
        }
        Err(e) => {
            let error_msg = format!("❌ Cannot connect to API: {}", e);
            info!("{}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn test_api_articles() -> Result<String, String> {
    info!("Testing API GET articles...");
    
    match api_client::ApiClient::from_default() {
        Ok(api) => {
            match api.get_all_articles().await {
                Ok(articles) => {
                    let count = articles.len();
                    info!("Successfully fetched {} articles from API", count);
                    
                    // Return summary with first 3 articles
                    let summary = if count > 0 {
                        let sample: Vec<String> = articles.iter()
                            .take(3)
                            .map(|a| format!("{} - {} {} (Preț: {})", 
                                a.denumire, 
                                a.um,
                                a.clasa.as_deref().unwrap_or(""),
                                a.pret_vanzare.as_deref().unwrap_or("N/A")
                            ))
                            .collect();
                        format!("✅ Success! Found {} articles.\n\nSample:\n{}", count, sample.join("\n"))
                    } else {
                        "✅ API works but returned 0 articles".to_string()
                    };
                    
                    Ok(summary)
                }
                Err(e) => {
                    let error_msg = format!("❌ API call failed: {}", e);
                    info!("{}", error_msg);
                    Err(error_msg)
                }
            }
        }
        Err(e) => {
            let error_msg = format!("❌ Cannot connect to API: {}", e);
            info!("{}", error_msg);
            Err(error_msg)
        }
    }
}

// ==================== PARTNER COMMANDS ====================

#[tauri::command]
pub fn get_partners(db: State<'_, Database>) -> Result<Vec<PartnerWithLocations>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name, cif, reg_com, cod, blocat, tva_la_incasare, persoana_fizica, cod_extern, cod_intern, observatii, data_adaugarii, created_at, updated_at, clasa, simbol_clasa, cod_clasa, inactiv, categorie_pret_implicita, simbol_categorie_pret, scadenta_la_vanzare, scadenta_la_cumparare, credit_client, discount_fix, tip_partener, mod_aplicare_discount, moneda, data_nastere, caracterizare_contabila_denumire, caracterizare_contabila_simbol FROM partners WHERE simbol_clasa = 'AGENTI' OR clasa = 'AGENTI' ORDER BY name")
        .map_err(|e| e.to_string())?;

    let partners: Vec<(
        String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, String, String,
        Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>
    )> = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?, row.get(9)?, row.get(10)?, row.get(11)?, row.get(12)?, row.get(13)?,
                row.get(14)?, row.get(15)?, row.get(16)?, row.get(17)?, row.get(18)?, row.get(19)?, row.get(20)?, row.get(21)?, row.get(22)?, row.get(23)?, row.get(24)?, row.get(25)?, row.get(26)?, row.get(27)?, row.get(28)?, row.get(29)?
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut result = Vec::new();

    for (id, name, cif, reg_com, cod, blocat, tva_la_incasare, persoana_fizica, cod_extern, cod_intern, observatii, data_adaugarii, created_at, updated_at, clasa, simbol_clasa, cod_clasa, inactiv, categorie_pret_implicita, simbol_categorie_pret, scadenta_la_vanzare, scadenta_la_cumparare, credit_client, discount_fix, tip_partener, mod_aplicare_discount, moneda, data_nastere, caracterizare_contabila_denumire, caracterizare_contabila_simbol) in partners {
        let mut loc_stmt = conn
            .prepare("SELECT id, partner_id, name, address, cod_sediu, localitate, strada, numar, judet, tara, cod_postal, telefon, email, inactiv FROM locations WHERE partner_id = ?1")
            .map_err(|e| e.to_string())?;

        let locations: Vec<Location> = loc_stmt
            .query_map([&id], |row| {
                Ok(Location {
                    id: row.get(0)?,
                    partner_id: row.get(1)?,
                    name: row.get(2)?,
                    address: row.get(3)?,
                    cod_sediu: row.get(4)?,
                    localitate: row.get(5)?,
                    strada: row.get(6)?,
                    numar: row.get(7)?,
                    judet: row.get(8)?,
                    tara: row.get(9)?,
                    cod_postal: row.get(10)?,
                    telefon: row.get(11)?,
                    email: row.get(12)?,
                    inactiv: row.get(13)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        result.push(PartnerWithLocations {
            id,
            name,
            cif,
            reg_com,
            cod,
            blocat,
            tva_la_incasare,
            persoana_fizica,
            cod_extern,
            cod_intern,
            observatii,
            data_adaugarii,
            created_at,
            updated_at,
            clasa,
            simbol_clasa,
            cod_clasa,
            inactiv,
            categorie_pret_implicita,
            simbol_categorie_pret,
            scadenta_la_vanzare,
            scadenta_la_cumparare,
            credit_client,
            discount_fix,
            tip_partener,
            mod_aplicare_discount,
            moneda,
            data_nastere,
            caracterizare_contabila_denumire,
            caracterizare_contabila_simbol,
            locations,
        });
    }

    Ok(result)
}

#[tauri::command]
pub fn search_partners(
    db: State<'_, Database>,
    query: String,
) -> Result<Vec<PartnerWithLocations>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let search_query = format!("%{}%", query);

    let mut stmt = conn
        .prepare("SELECT id, name, cif, reg_com, cod, blocat, tva_la_incasare, persoana_fizica, cod_extern, cod_intern, observatii, data_adaugarii, created_at, updated_at, clasa, simbol_clasa, cod_clasa, inactiv, categorie_pret_implicita, simbol_categorie_pret, scadenta_la_vanzare, scadenta_la_cumparare, credit_client, discount_fix, tip_partener, mod_aplicare_discount, moneda, data_nastere, caracterizare_contabila_denumire, caracterizare_contabila_simbol FROM partners WHERE (simbol_clasa = 'AGENTI' OR clasa = 'AGENTI') AND name LIKE ?1 ORDER BY name")
        .map_err(|e| e.to_string())?;

    let partners: Vec<(
        String, String, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, String, String,
        Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>
    )> = stmt
        .query_map([&search_query], |row| {
            Ok((
                row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?, row.get(9)?, row.get(10)?, row.get(11)?, row.get(12)?, row.get(13)?,
                row.get(14)?, row.get(15)?, row.get(16)?, row.get(17)?, row.get(18)?, row.get(19)?, row.get(20)?, row.get(21)?, row.get(22)?, row.get(23)?, row.get(24)?, row.get(25)?, row.get(26)?, row.get(27)?, row.get(28)?, row.get(29)?
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut result = Vec::new();

    for (id, name, cif, reg_com, cod, blocat, tva_la_incasare, persoana_fizica, cod_extern, cod_intern, observatii, data_adaugarii, created_at, updated_at, clasa, simbol_clasa, cod_clasa, inactiv, categorie_pret_implicita, simbol_categorie_pret, scadenta_la_vanzare, scadenta_la_cumparare, credit_client, discount_fix, tip_partener, mod_aplicare_discount, moneda, data_nastere, caracterizare_contabila_denumire, caracterizare_contabila_simbol) in partners {
        let mut loc_stmt = conn
            .prepare("SELECT id, partner_id, name, address, cod_sediu, localitate, strada, numar, judet, tara, cod_postal, telefon, email, inactiv FROM locations WHERE partner_id = ?1")
            .map_err(|e| e.to_string())?;

        let locations: Vec<Location> = loc_stmt
            .query_map([&id], |row| {
                Ok(Location {
                    id: row.get(0)?,
                    partner_id: row.get(1)?,
                    name: row.get(2)?,
                    address: row.get(3)?,
                    cod_sediu: row.get(4)?,
                    localitate: row.get(5)?,
                    strada: row.get(6)?,
                    numar: row.get(7)?,
                    judet: row.get(8)?,
                    tara: row.get(9)?,
                    cod_postal: row.get(10)?,
                    telefon: row.get(11)?,
                    email: row.get(12)?,
                    inactiv: row.get(13)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        result.push(PartnerWithLocations {
            id,
            name,
            cif,
            reg_com,
            cod,
            blocat,
            tva_la_incasare,
            persoana_fizica,
            cod_extern,
            cod_intern,
            observatii,
            data_adaugarii,
            created_at,
            updated_at,
            clasa,
            simbol_clasa,
            cod_clasa,
            inactiv,
            categorie_pret_implicita,
            simbol_categorie_pret,
            scadenta_la_vanzare,
            scadenta_la_cumparare,
            credit_client,
            discount_fix,
            tip_partener,
            mod_aplicare_discount,
            moneda,
            data_nastere,
            caracterizare_contabila_denumire,
            caracterizare_contabila_simbol,
            locations,
        });
    }

    Ok(result)
}

// ==================== PRODUCT COMMANDS ====================

#[tauri::command]
pub fn get_products(db: State<'_, Database>, partner_id: Option<String>) -> Result<Vec<Product>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = if partner_id.is_some() {
        conn.prepare(
            "SELECT p.id, p.name, p.unit_of_measure, COALESCE(oi.pret, p.price) AS price, p.class, p.procent_tva \
             FROM products p \
             LEFT JOIN offer_items oi ON oi.product_id = p.id AND oi.id_client = ?1 \
             ORDER BY p.name",
        )
        .map_err(|e| e.to_string())?
    } else {
        conn.prepare("SELECT id, name, unit_of_measure, CASE WHEN price = 0 THEN COALESCE(pret_cu_tva, pret_valuta, pret_referinta, 0) ELSE price END AS price, class, procent_tva FROM products ORDER BY name")
            .map_err(|e| e.to_string())?
    };

    let products: Vec<Product> = if let Some(pid) = &partner_id {
        stmt.query_map([pid], map_product_row)
    } else {
        stmt.query_map([], map_product_row)
    }
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(products)
}

#[tauri::command]
pub fn search_products(db: State<'_, Database>, query: String, partner_id: Option<String>) -> Result<Vec<Product>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let search_query = format!("%{}%", query);

    let mut stmt = if partner_id.is_some() {
        conn.prepare(
            "SELECT p.id, p.name, p.unit_of_measure, COALESCE(oi.pret, p.price) AS price, p.class, p.procent_tva \
             FROM products p \
             LEFT JOIN offer_items oi ON oi.product_id = p.id AND oi.id_client = ?2 \
             WHERE p.name LIKE ?1 OR p.class LIKE ?1 \
             ORDER BY p.name",
        )
        .map_err(|e| e.to_string())?
    } else {
        conn.prepare("SELECT id, name, unit_of_measure, CASE WHEN price = 0 THEN COALESCE(pret_cu_tva, pret_valuta, pret_referinta, 0) ELSE price END AS price, class, procent_tva FROM products WHERE name LIKE ?1 OR class LIKE ?1 ORDER BY name")
            .map_err(|e| e.to_string())?
    };

    let products: Vec<Product> = if let Some(pid) = &partner_id {
        stmt.query_map([&search_query, pid], map_product_row)
    } else {
        stmt.query_map([&search_query], map_product_row)
    }
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(products)
}

// ==================== INVOICE COMMANDS ====================

#[tauri::command]
pub fn create_invoice(
    db: State<'_, Database>,
    request: CreateInvoiceRequest,
) -> Result<Invoice, String> {
    info!("Creating invoice - Partner ID received: {}", request.partner_id);
    
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    let invoice_id = Uuid::new_v4().to_string();

    // Get partner name and cod
    let (partner_name, partner_cod): (String, Option<String>) = conn
        .query_row(
            "SELECT name, cod FROM partners WHERE id = ?1",
            [&request.partner_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("Partner not found: {}", e))?;

    info!("Partner found in DB - Name: {}, COD: {:?}", partner_name, partner_cod);

    // Get location name and address
    let (location_name, location_address): (String, Option<String>) = conn
        .query_row(
            "SELECT name, address FROM locations WHERE id = ?1",
            [&request.location_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("Location not found: {}", e))?;

    // Calculate total and prepare items
    let mut total_amount = 0.0;
    let mut items_to_insert: Vec<(String, String, String, f64, f64, String, f64)> = Vec::new();

    for item in &request.items {
        // First try to get price from offer_items
        let offer_price: Option<f64> = conn
            .query_row(
                "SELECT pret FROM offer_items WHERE product_id = ?1 AND id_client = ?2",
                [&item.product_id, &request.partner_id],
                |row| row.get(0),
            )
            .ok();

        let (product_name, product_price, um): (String, f64, String) = conn
            .query_row(
                "SELECT name, price, unit_of_measure FROM products WHERE id = ?1",
                [&item.product_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| format!("Product not found: {}", e))?;

        // Use offer price if available, otherwise use product price
        let price = offer_price.unwrap_or(product_price);
        
        if offer_price.is_some() {
            info!("Using offer price {} for product {} (partner {})", price, product_name, request.partner_id);
        } else {
            info!("No offer price found for product {} (partner {}), using standard price {}", product_name, request.partner_id, price);
        }

        let item_total = price * item.quantity;
        total_amount += item_total;

        items_to_insert.push((
            Uuid::new_v4().to_string(),
            item.product_id.clone(),
            product_name,
            item.quantity,
            price,
            um,
            item_total,
        ));
    }

    // Get invoice number from agent settings (settings-based numbering)
    let (invoice_number, invoice_end): (i64, i64) = conn
        .query_row(
            "SELECT COALESCE(invoice_number_current, 1), COALESCE(invoice_number_end, 99999) FROM agent_settings WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap_or((1, 99999));

    // Validate we haven't exceeded the end number
    if invoice_number > invoice_end {
        return Err(format!(
            "Invoice number {} exceeds maximum configured number {}. Please update the number range in settings.",
            invoice_number, invoice_end
        ));
    }

    info!("Using invoice number {} from settings (max: {})", invoice_number, invoice_end);

    // Insert invoice with number from settings
    conn.execute(
        "INSERT INTO invoices (id, invoice_number, partner_id, location_id, status, total_amount, notes, created_at) VALUES (?1, ?2, ?3, ?4, 'pending', ?5, ?6, ?7)",
        (&invoice_id, invoice_number, &request.partner_id, &request.location_id, total_amount, &request.notes, &now),
    )
    .map_err(|e| e.to_string())?;

    // Increment the current invoice number in settings for next invoice
    // Using UPSERT to handle case when agent_settings has no rows
    conn.execute(
        "INSERT INTO agent_settings (id, invoice_number_current) VALUES (1, ?1) 
         ON CONFLICT(id) DO UPDATE SET invoice_number_current = invoice_number_current + 1",
        [invoice_number + 1],
    )
    .map_err(|e| e.to_string())?;

    info!("Invoice created successfully. Next invoice number will be: {}", invoice_number + 1);

    // Insert invoice items
    for (item_id, product_id, _, quantity, unit_price, _, total_price) in &items_to_insert {
        conn.execute(
            "INSERT INTO invoice_items (id, invoice_id, product_id, quantity, unit_price, total_price) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (item_id, &invoice_id, product_id, quantity, unit_price, total_price),
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(Invoice {
        id: invoice_id,
        partner_id: request.partner_id,
        partner_name,
        partner_cif: None,
        partner_reg_com: None,
        location_id: request.location_id,
        location_name,
        location_address,
        status: InvoiceStatus::Pending,
        total_amount,
        item_count: items_to_insert.len() as i32,
        notes: request.notes,
        created_at: now,
        sent_at: None,
        error_message: None,
        partner_payment_term: None,
    })
}

#[tauri::command]
pub fn get_invoices(
    db: State<'_, Database>,
    status_filter: Option<String>,
) -> Result<Vec<Invoice>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let sql = match &status_filter {
        Some(status) => format!(
            r#"
            SELECT
                i.id, i.partner_id, p.name, p.cif, p.reg_com, i.location_id, l.name, l.address,
                i.status, i.total_amount, i.notes, i.created_at, i.sent_at, i.error_message,
                (SELECT COUNT(*) FROM invoice_items WHERE invoice_id = i.id),
                p.scadenta_la_vanzare
            FROM invoices i
            JOIN partners p ON i.partner_id = p.id
            JOIN locations l ON i.location_id = l.id
            WHERE i.status = '{}'
            ORDER BY i.created_at DESC
            "#,
            status
        ),
        None => r#"
            SELECT
                i.id, i.partner_id, p.name, p.cif, p.reg_com, i.location_id, l.name, l.address,
                i.status, i.total_amount, i.notes, i.created_at, i.sent_at, i.error_message,
                (SELECT COUNT(*) FROM invoice_items WHERE invoice_id = i.id),
                p.scadenta_la_vanzare
            FROM invoices i
            JOIN partners p ON i.partner_id = p.id
            JOIN locations l ON i.location_id = l.id
            ORDER BY i.created_at DESC
            "#
        .to_string(),
    };

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;

    let invoices: Vec<Invoice> = stmt
        .query_map([], |row| {
            Ok(Invoice {
                id: row.get(0)?,
                partner_id: row.get(1)?,
                partner_name: row.get(2)?,
                partner_cif: row.get(3)?,
                partner_reg_com: row.get(4)?,
                location_id: row.get(5)?,
                location_name: row.get(6)?,
                location_address: row.get(7)?,
                status: InvoiceStatus::from(row.get::<_, String>(8)?),
                total_amount: row.get(9)?,
                notes: row.get(10)?,
                created_at: row.get(11)?,
                sent_at: row.get(12)?,
                error_message: row.get(13)?,
                item_count: row.get(14)?,
                partner_payment_term: row.get(15)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(invoices)
}

#[tauri::command]
pub fn get_invoice_detail(
    db: State<'_, Database>,
    invoice_id: String,
) -> Result<InvoiceDetail, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Get invoice
    let invoice: Invoice = conn
        .query_row(
            r#"
            SELECT
                i.id, i.partner_id, p.name, p.cif, p.reg_com, i.location_id, l.name, l.address,
                i.status, i.total_amount, i.notes, i.created_at, i.sent_at, i.error_message,
                (SELECT COUNT(*) FROM invoice_items WHERE invoice_id = i.id),
                p.scadenta_la_vanzare
            FROM invoices i
            JOIN partners p ON i.partner_id = p.id
            JOIN locations l ON i.location_id = l.id
            WHERE i.id = ?1
            "#,
            [&invoice_id],
            |row| {
                Ok(Invoice {
                    id: row.get(0)?,
                    partner_id: row.get(1)?,
                    partner_name: row.get(2)?,
                    partner_cif: row.get(3)?,
                    partner_reg_com: row.get(4)?,
                    location_id: row.get(5)?,
                    location_name: row.get(6)?,
                    location_address: row.get(7)?,
                    status: InvoiceStatus::from(row.get::<_, String>(8)?),
                    total_amount: row.get(9)?,
                    notes: row.get(10)?,
                    created_at: row.get(11)?,
                    sent_at: row.get(12)?,
                    error_message: row.get(13)?,
                    item_count: row.get(14)?,
                    partner_payment_term: row.get(15)?,
                })
            },
        )
        .map_err(|e| format!("Invoice not found: {}", e))?;

    // Get invoice items
    let mut stmt = conn
        .prepare(
            r#"
            SELECT
                ii.id, ii.invoice_id, ii.product_id, pr.name, ii.quantity, ii.unit_price, pr.unit_of_measure, ii.total_price, pr.procent_tva
            FROM invoice_items ii
            JOIN products pr ON ii.product_id = pr.id
            WHERE ii.invoice_id = ?1
            "#,
        )
        .map_err(|e| e.to_string())?;

    let items: Vec<InvoiceItem> = stmt
        .query_map([&invoice_id], |row| {
            // Parse TVA percentage from TEXT to f64
            let tva_percent: Option<f64> = match row.get::<_, Option<String>>(8)? {
                Some(s) => s.parse::<f64>().ok(),
                None => None,
            };
            
            Ok(InvoiceItem {
                id: row.get(0)?,
                invoice_id: row.get(1)?,
                product_id: row.get(2)?,
                product_name: row.get(3)?,
                quantity: row.get(4)?,
                unit_price: row.get(5)?,
                unit_of_measure: row.get(6)?,
                total_price: row.get(7)?,
                tva_percent,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(InvoiceDetail { invoice, items })
}

#[tauri::command]
pub async fn send_invoice(db: State<'_, Database>, invoice_id: String) -> Result<Invoice, String> {
    // Get invoice details and items
    let (invoice, items, partner_cod, location_id_sediu, invoice_number, partner_moneda, partner_payment_term): (Invoice, Vec<(String, f64, f64, String)>, Option<String>, Option<String>, i64, Option<String>, Option<String>) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        
        // Get invoice with partner cod
        let invoice: Invoice = conn
            .query_row(
                r#"
                SELECT
                    i.id, i.partner_id, p.name, p.cif, p.reg_com, i.location_id, l.name, l.address,
                    i.status, i.total_amount, i.notes, i.created_at, i.sent_at, i.error_message,
                    (SELECT COUNT(*) FROM invoice_items WHERE invoice_id = i.id),
                    p.cod
                FROM invoices i
                JOIN partners p ON i.partner_id = p.id
                JOIN locations l ON i.location_id = l.id
                WHERE i.id = ?1
                "#,
                [&invoice_id],
                |row| {
                    Ok(Invoice {
                        id: row.get(0)?,
                        partner_id: row.get(1)?,
                        partner_name: row.get(2)?,
                        partner_cif: row.get(3)?,
                        partner_reg_com: row.get(4)?,
                        location_id: row.get(5)?,
                        location_name: row.get(6)?,
                        location_address: row.get(7)?,
                        status: InvoiceStatus::from(row.get::<_, String>(8)?),
                        total_amount: row.get(9)?,
                        notes: row.get(10)?,
                        created_at: row.get(11)?,
                        sent_at: row.get(12)?,
                        error_message: row.get(13)?,
                        item_count: row.get(14)?,
                        partner_payment_term: None,
                    })
                },
            )
            .map_err(|e| format!("Invoice not found: {}", e))?;

        let (partner_cod, partner_moneda, partner_payment_term): (Option<String>, Option<String>, Option<String>) = conn
            .query_row(
                "SELECT cod_intern, moneda, scadenta_la_vanzare FROM partners WHERE id = ?1",
                [&invoice.partner_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| format!("Failed to get partner settings: {}", e))?;

        info!("Partner info - Name: {}, ID: {}, CodIntern: {:?}", invoice.partner_name, invoice.partner_id, partner_cod);

        let location_id_sediu: Option<String> = conn
            .query_row("SELECT id_sediu FROM locations WHERE id = ?1", [&invoice.location_id], |row| row.get(0))
            .ok()
            .flatten();

        // Get invoice number from the invoice record
        let invoice_number: i64 = conn
            .query_row("SELECT invoice_number FROM invoices WHERE id = ?1", [&invoice_id], |row| row.get(0))
            .map_err(|e| format!("Failed to get invoice number: {}", e))?;

        // Get invoice items with UM from products
        let mut stmt = conn
            .prepare(
                "SELECT ii.product_id, ii.quantity, ii.unit_price, p.unit_of_measure \
                 FROM invoice_items ii \
                 JOIN products p ON ii.product_id = p.id \
                 WHERE ii.invoice_id = ?1",
            )
            .map_err(|e| e.to_string())?;

        let items: Vec<(String, f64, f64, String)> = stmt
            .query_map([&invoice_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        (invoice, items, partner_cod, location_id_sediu, invoice_number, partner_moneda, partner_payment_term)
    };

    // Get agent settings
    let agent_settings = get_agent_settings(db.clone())?;

    // Validate required settings
    if agent_settings.agent_name.is_none() || agent_settings.agent_name.as_ref().unwrap().is_empty() {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let err_msg = "Agent name is not configured. Please set it in Settings.".to_string();
        conn.execute(
            "UPDATE invoices SET status = 'pending', error_message = ?1 WHERE id = ?2",
            [&err_msg, &invoice_id],
        ).ok();
        return Err(err_msg);
    }
    if agent_settings.carnet_series.is_none() || agent_settings.carnet_series.as_ref().unwrap().is_empty() {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let err_msg = "Carnet series is not configured. Please set it in Settings.".to_string();
        conn.execute(
            "UPDATE invoices SET status = 'pending', error_message = ?1 WHERE id = ?2",
            [&err_msg, &invoice_id],
        ).ok();
        return Err(err_msg);
    }
    if agent_settings.simbol_carnet_livr.is_none() || agent_settings.simbol_carnet_livr.as_ref().unwrap().is_empty() {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let err_msg = "Simbol Carnet Livrări is not configured. Please set it in Settings.".to_string();
        conn.execute(
            "UPDATE invoices SET status = 'pending', error_message = ?1 WHERE id = ?2",
            [&err_msg, &invoice_id],
        ).ok();
        return Err(err_msg);
    }
    if agent_settings.cod_carnet.is_none() {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let err_msg = "Cod Carnet is not configured. Please set it in Settings.".to_string();
        conn.execute(
            "UPDATE invoices SET status = 'pending', error_message = ?1 WHERE id = ?2",
            [&err_msg, &invoice_id],
        ).ok();
        return Err(err_msg);
    }
    if agent_settings.cod_carnet_livr.is_none() {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let err_msg = "Cod Carnet Livrări is not configured. Please set it in Settings.".to_string();
        conn.execute(
            "UPDATE invoices SET status = 'pending', error_message = ?1 WHERE id = ?2",
            [&err_msg, &invoice_id],
        ).ok();
        return Err(err_msg);
    }
    if agent_settings.simbol_gestiune_livrare.is_none() || agent_settings.simbol_gestiune_livrare.as_ref().unwrap().is_empty() {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let err_msg = "Simbol Gestiune Livrare is not configured. Please set it in Settings.".to_string();
        conn.execute(
            "UPDATE invoices SET status = 'pending', error_message = ?1 WHERE id = ?2",
            [&err_msg, &invoice_id],
        ).ok();
        return Err(err_msg);
    }
    if partner_cod.is_none() || partner_cod.as_ref().unwrap().is_empty() {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let err_msg = format!("Partner {} does not have a COD set in WME", invoice.partner_name);
        conn.execute(
            "UPDATE invoices SET status = 'pending', error_message = ?1 WHERE id = ?2",
            [&err_msg, &invoice_id],
        ).ok();
        return Err(err_msg);
    }

    let marca_agent = agent_settings
        .marca_agent
        .clone()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    if marca_agent.is_none() {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let err_msg = "Marca Agent is not configured. Please set it in Settings.".to_string();
        conn.execute(
            "UPDATE invoices SET status = 'pending', error_message = ?1 WHERE id = ?2",
            [&err_msg, &invoice_id],
        ).ok();
        return Err(err_msg);
    }

    let marca_agent = marca_agent.unwrap();

    if !marca_agent.chars().all(|c| c.is_ascii_digit()) {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let err_msg = "Marca Agent must be numeric for WME sending.".to_string();
        conn.execute(
            "UPDATE invoices SET status = 'pending', error_message = ?1 WHERE id = ?2",
            [&err_msg, &invoice_id],
        ).ok();
        return Err(err_msg);
    }

    // After validations, mark as sending
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE invoices SET status = 'sending' WHERE id = ?1",
            [&invoice_id],
        )
        .map_err(|e| e.to_string())?;
    }
    // Parse invoice date
    let invoice_date = chrono::DateTime::parse_from_rfc3339(&invoice.created_at)
        .map_err(|e| format!("Failed to parse invoice date: {}", e))?;
    
    let an_lucru = invoice_date.year();
    let luna_lucru = invoice_date.month() as i32;
    let data_formatted = invoice_date.format("%d.%m.%Y").to_string();
    let scadenta = compute_due_date(&invoice.created_at, partner_payment_term.as_deref())?;
    let moneda = partner_moneda
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "RON".to_string());
    let locatie = if invoice.location_name.trim().is_empty() {
        "SEDIU".to_string()
    } else {
        invoice.location_name.clone()
    };
    let cod_delegat = agent_settings
        .cod_delegat
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_default();

    // Build WME items
    let gestiune = agent_settings.simbol_gestiune_livrare.clone().unwrap();
    let tip_contabil = agent_settings
        .tip_contabil
        .clone()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "valoare".to_string());
    let wme_items: Vec<api_client::WmeInvoiceItem> = items
        .into_iter()
        .map(|(product_id, quantity, price, um)| api_client::WmeInvoiceItem {
            id_articol: product_id,
            cant: quantity,
            pret: price,
            um: Some(um),
            gestiune: Some(gestiune.clone()),
            tip_contabil: Some(tip_contabil.clone()),
            pret_inreg: 0.0,
            pret_achiz: 0.0,
            observatii: None,
            tva: None,
        })
        .collect();

    // Build WME request
    let wme_request = api_client::WmeInvoiceRequest {
        tip_document: Some("FACTURA IESIRE".to_string()),
        an_lucru: Some(an_lucru.to_string()),
        luna_lucru: Some(luna_lucru.to_string()),
        cod_subunitate: None,
        documente: vec![api_client::WmeDocument {
            tip_document: Some("FACTURA IESIRE".to_string()),
            numar_document: Some(invoice_number.to_string()), // Folosim numărul din aplicație
            simbol_carnet: Some(agent_settings.carnet_series.clone().unwrap()),
            nr_livr: Some(invoice_number.to_string()),
            simbol_carnet_livr: Some(agent_settings.simbol_carnet_livr.clone().unwrap()),
            simbol_gestiune_livrare: Some(agent_settings.simbol_gestiune_livrare.clone().unwrap()),
            numerotare_automata: None, // Nu mai folosim numerotare automată - folosim NrDoc
            data: Some(data_formatted.clone()),
            data_livr: Some(data_formatted),
            operatie: Some("A".to_string()),
            anulat: Some("N".to_string()),
            listat: Some("D".to_string()),
            cod_client: Some(partner_cod.unwrap()),
            id_sediu: location_id_sediu,
            locatie: Some(locatie),
            agent: Some(marca_agent),
            tip_tva: Some("1".to_string()),
            tip_tranzactie: Some("1".to_string()),
            factura_simplificata: Some("N".to_string()),
            moneda: Some(moneda),
            curs: Some("1".to_string()),
            operat: Some("D".to_string()),
            cod_delegat: Some(cod_delegat),
            emisa_de: Some("1".to_string()),
            scadenta: Some(scadenta),
            observatii: invoice.notes.clone(),
            items: Some(wme_items),
        }],
    };

    // Log the JSON payload for debugging
    info!("=== WME API REQUEST PAYLOAD ===");
    match serde_json::to_string_pretty(&wme_request) {
        Ok(json) => info!("{}", json),
        Err(e) => info!("Failed to serialize request: {}", e),
    }
    info!("===============================");

    // Send to WME API
    let result = match api_client::ApiClient::from_default() {
        Ok(client) => client.send_invoice_to_wme(wme_request).await,
        Err(e) => Err(format!("Failed to create API client: {}", e)),
    };

    let now = Utc::now().to_rfc3339();

    // Update based on result and return the invoice
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    match result {
        Ok(response) => {
            // Verify that document was actually created
            if let Some(doc) = response.documente_importate.first() {
                // Check if document has a number (was actually created)
                if doc.numar.is_some() && !doc.numar.as_ref().unwrap().is_empty() {
                    let doc_info = format!("WME: {} {}", 
                        doc.serie.clone().unwrap_or_default(), 
                        doc.numar.clone().unwrap_or_default()
                    );
                    
                    info!("Invoice successfully created in WME: {}", doc_info);
                    
                    conn.execute(
                        "UPDATE invoices SET status = 'sent', sent_at = ?1, error_message = ?2 WHERE id = ?3",
                        [&now, &doc_info, &invoice_id],
                    )
                    .map_err(|e| e.to_string())?;
                } else {
                    // API returned success but no document number - treat as error
                    let error_msg = format!("API responded OK but document was not created. Result: {:?}", response.result);
                    warn!("Invoice send failed - no document created: {}", error_msg);
                    
                    conn.execute(
                        "UPDATE invoices SET status = 'pending', error_message = ?1 WHERE id = ?2",
                        [&error_msg, &invoice_id],
                    )
                    .map_err(|e| e.to_string())?;
                }
            } else {
                // No documents in response - treat as error
                let error_msg = "API responded OK but returned no documents".to_string();
                warn!("Invoice send failed - empty response: {}", error_msg);
                
                conn.execute(
                    "UPDATE invoices SET status = 'pending', error_message = ?1 WHERE id = ?2",
                    [&error_msg, &invoice_id],
                )
                .map_err(|e| e.to_string())?;
            }
        }
        Err(error) => {
            // Offline/Error Handling:
            // User requested to not show error but treat as "pending" (in asteptare).
            // We save the error message for context but set status to pending.
            let error_msg = format!("Salvat local (Offline/Eroare): {}", error);
            warn!("Failed to send invoice (handling as offline): {}", error);

            conn.execute(
                "UPDATE invoices SET status = 'pending', error_message = ?1 WHERE id = ?2",
                [&error_msg, &invoice_id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // Fetch the updated invoice
    let invoice: Invoice = conn
        .query_row(
            r#"
            SELECT
                i.id, i.partner_id, p.name, p.cif, p.reg_com, i.location_id, l.name, l.address,
                i.status, i.total_amount, i.notes, i.created_at, i.sent_at, i.error_message,
                (SELECT COUNT(*) FROM invoice_items WHERE invoice_id = i.id),
                p.cod
            FROM invoices i
            JOIN partners p ON i.partner_id = p.id
            JOIN locations l ON i.location_id = l.id
            WHERE i.id = ?1
            "#,
            [&invoice_id],
            |row| {
                Ok(Invoice {
                    id: row.get(0)?,
                    partner_id: row.get(1)?,
                    partner_name: row.get(2)?,
                    partner_cif: row.get(3)?,
                    partner_reg_com: row.get(4)?,
                    location_id: row.get(5)?,
                    location_name: row.get(6)?,
                    location_address: row.get(7)?,
                    status: InvoiceStatus::from(row.get::<_, String>(8)?),
                    total_amount: row.get(9)?,
                    notes: row.get(10)?,
                    created_at: row.get(11)?,
                    sent_at: row.get(12)?,
                    error_message: row.get(13)?,
                    item_count: row.get(14)?,
                    partner_payment_term: None,
                })
            },
        )
        .map_err(|e| format!("Invoice not found: {}", e))?;

    // Drop the lock before async operation
    drop(conn);

    Ok(invoice)
}

#[tauri::command]
pub async fn preview_invoice_json(db: State<'_, Database>, invoice_id: String) -> Result<String, String> {
    info!("Previewing JSON for invoice: {}", invoice_id);

    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Fetch invoice basic info
    let (partner_name, location_name, notes, created_at, invoice_number): (String, String, Option<String>, String, i64) = conn
        .query_row(
            "SELECT p.name, l.name, i.notes, i.created_at, i.invoice_number FROM invoices i JOIN partners p ON i.partner_id = p.id JOIN locations l ON i.location_id = l.id WHERE i.id = ?1",
            [&invoice_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .map_err(|e| format!("Invoice not found: {}", e))?;

    // Get agent settings
    let agent_settings = get_agent_settings(db.clone()).map_err(|e| e.to_string())?;

    // Get partner CodIntern and location ID
    let (partner_cod, location_id_sediu, partner_moneda, partner_payment_term): (Option<String>, Option<String>, Option<String>, Option<String>) = conn
        .query_row(
            "SELECT p.cod_intern, l.id_sediu, p.moneda, p.scadenta_la_vanzare FROM invoices i JOIN partners p ON i.partner_id = p.id JOIN locations l ON i.location_id = l.id WHERE i.id = ?1",
            [&invoice_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|e| format!("Failed to get partner info: {}", e))?;

    // Get invoice items
    let mut stmt = conn
        .prepare(
            "SELECT ii.product_id, ii.quantity, ii.unit_price, p.unit_of_measure \
             FROM invoice_items ii \
             JOIN products p ON ii.product_id = p.id \
             WHERE ii.invoice_id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let items: Vec<(String, f64, f64, String)> = stmt
        .query_map([&invoice_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    drop(stmt);
    drop(conn);

    // Validate required settings
    if agent_settings.agent_name.is_none() || agent_settings.agent_name.as_ref().unwrap().is_empty() {
        return Err("Agent name is not configured. Please set it in Settings.".to_string());
    }
    if agent_settings.carnet_series.is_none() || agent_settings.carnet_series.as_ref().unwrap().is_empty() {
        return Err("Carnet series is not configured. Please set it in Settings.".to_string());
    }
    if agent_settings.simbol_carnet_livr.is_none() || agent_settings.simbol_carnet_livr.as_ref().unwrap().is_empty() {
        return Err("Simbol Carnet Livrări is not configured. Please set it in Settings.".to_string());
    }
    if agent_settings.simbol_gestiune_livrare.is_none() || agent_settings.simbol_gestiune_livrare.as_ref().unwrap().is_empty() {
        return Err("Simbol Gestiune Livrare is not configured. Please set it in Settings.".to_string());
    }
    if agent_settings.cod_carnet.is_none() {
        return Err("Cod Carnet is not configured. Please set it in Settings.".to_string());
    }
    if agent_settings.cod_carnet_livr.is_none() {
        return Err("Cod Carnet Livrări is not configured. Please set it in Settings.".to_string());
    }
    if partner_cod.is_none() || partner_cod.as_ref().unwrap().is_empty() {
        return Err(format!("Partner {} does not have a COD set in WME", partner_name));
    }

    let marca_agent = agent_settings
        .marca_agent
        .clone()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or("Marca Agent is not configured. Please set it in Settings.")?;

    if !marca_agent.chars().all(|c| c.is_ascii_digit()) {
        return Err("Marca Agent must be numeric for WME sending.".to_string());
    }

    // Parse invoice date
    let invoice_date = chrono::DateTime::parse_from_rfc3339(&created_at)
        .map_err(|e| format!("Failed to parse invoice date: {}", e))?;
    
    let an_lucru = invoice_date.year();
    let luna_lucru = invoice_date.month() as i32;
    let data_formatted = invoice_date.format("%d.%m.%Y").to_string();
    let scadenta = compute_due_date(&created_at, partner_payment_term.as_deref())?;
    let moneda = partner_moneda
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "RON".to_string());
    let locatie = if location_name.trim().is_empty() {
        "SEDIU".to_string()
    } else {
        location_name
    };
    let cod_delegat = agent_settings
        .cod_delegat
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_default();

    // Build WME items
    let gestiune = agent_settings.simbol_gestiune_livrare.clone().unwrap();
    let tip_contabil = agent_settings
        .tip_contabil
        .clone()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "valoare".to_string());
    let wme_items: Vec<api_client::WmeInvoiceItem> = items
        .into_iter()
        .map(|(product_id, quantity, price, um)| api_client::WmeInvoiceItem {
            id_articol: product_id,
            cant: quantity,
            pret: price,
            um: Some(um),
            gestiune: Some(gestiune.clone()),
            tip_contabil: Some(tip_contabil.clone()),
            pret_inreg: 0.0,
            pret_achiz: 0.0,
            observatii: None,
            tva: None,
        })
        .collect();

    // Build WME request
    let wme_request = api_client::WmeInvoiceRequest {
        tip_document: Some("FACTURA IESIRE".to_string()),
        an_lucru: Some(an_lucru.to_string()),
        luna_lucru: Some(luna_lucru.to_string()),
        cod_subunitate: None,
        documente: vec![api_client::WmeDocument {
            tip_document: Some("FACTURA IESIRE".to_string()),
            numar_document: Some(invoice_number.to_string()), // Folosim numărul din aplicație
            simbol_carnet: Some(agent_settings.carnet_series.clone().unwrap()),
            nr_livr: Some(invoice_number.to_string()),
            simbol_carnet_livr: Some(agent_settings.simbol_carnet_livr.clone().unwrap()),
            simbol_gestiune_livrare: Some(agent_settings.simbol_gestiune_livrare.clone().unwrap()),
            numerotare_automata: None, // Nu mai folosim numerotare automată - folosim NrDoc
            data: Some(data_formatted.clone()),
            data_livr: Some(data_formatted),
            operatie: Some("A".to_string()),
            anulat: Some("N".to_string()),
            listat: Some("D".to_string()),
            cod_client: Some(partner_cod.unwrap()),
            id_sediu: location_id_sediu,
            locatie: Some(locatie),
            agent: Some(marca_agent),
            tip_tva: Some("1".to_string()),
            tip_tranzactie: Some("1".to_string()),
            factura_simplificata: Some("N".to_string()),
            moneda: Some(moneda),
            curs: Some("1".to_string()),
            operat: Some("D".to_string()),
            cod_delegat: Some(cod_delegat),
            emisa_de: Some("1".to_string()),
            scadenta: Some(scadenta),
            observatii: notes.clone(),
            items: Some(wme_items),
        }],
    };

    // Return pretty JSON
    serde_json::to_string_pretty(&wme_request)
        .map_err(|e| format!("Failed to serialize request: {}", e))
}

#[tauri::command]
pub async fn send_all_pending_invoices(db: State<'_, Database>) -> Result<Vec<String>, String> {
    // Get all pending invoices
    let pending_ids: Vec<String> = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id FROM invoices WHERE status = 'pending' OR status = 'failed' ORDER BY created_at ASC")
            .map_err(|e| e.to_string())?;

        let ids: Vec<String> = stmt.query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        
        ids
    };

    if pending_ids.is_empty() {
        return Ok(vec![]);
    }

    info!("Found {} pending/failed invoices to send", pending_ids.len());
    let mut sent_ids = Vec::new();

    // Try to send each pending invoice
    for invoice_id in pending_ids {
        match send_invoice(db.clone(), invoice_id.clone()).await {
            Ok(invoice) => {
                if invoice.status == InvoiceStatus::Sent {
                    info!("Successfully sent invoice {}", invoice_id);
                    sent_ids.push(invoice_id);
                } else {
                    info!("Invoice {} failed to send: {:?}", invoice_id, invoice.error_message);
                }
            }
            Err(e) => {
                info!("Error sending invoice {}: {}", invoice_id, e);
            }
        }
    }

    Ok(sent_ids)
}

#[tauri::command]
pub fn cancel_invoice_sending(db: State<'_, Database>, invoice_id: String) -> Result<Invoice, String> {
    info!("Canceling invoice send: {}", invoice_id);

    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Check current status first
    let current_status: String = conn
        .query_row(
            "SELECT status FROM invoices WHERE id = ?1",
            [&invoice_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Invoice not found: {}", e))?;

    // Only allow canceling if status is "sending"
    if current_status != "sending" {
        return Err(format!("Cannot cancel invoice with status '{}'. Only 'sending' invoices can be cancelled.", current_status));
    }

    // Update invoice status to pending
    conn.execute(
        "UPDATE invoices SET status = 'pending', error_message = 'Trimitere anulată de utilizator' WHERE id = ?1",
        [&invoice_id],
    )
    .map_err(|e| e.to_string())?;

    // Fetch the updated invoice
    let invoice: Invoice = conn
        .query_row(
            r#"
            SELECT
                i.id, i.partner_id, p.name, p.cif, p.reg_com, i.location_id, l.name, l.address,
                i.status, i.total_amount, i.notes, i.created_at, i.sent_at, i.error_message,
                (SELECT COUNT(*) FROM invoice_items WHERE invoice_id = i.id)
            FROM invoices i
            JOIN partners p ON i.partner_id = p.id
            JOIN locations l ON i.location_id = l.id
            WHERE i.id = ?1
            "#,
            [&invoice_id],
            |row| {
                Ok(Invoice {
                    id: row.get(0)?,
                    partner_id: row.get(1)?,
                    partner_name: row.get(2)?,
                    partner_cif: row.get(3)?,
                    partner_reg_com: row.get(4)?,
                    location_id: row.get(5)?,
                    location_name: row.get(6)?,
                    location_address: row.get(7)?,
                    status: InvoiceStatus::from(row.get::<_, String>(8)?),
                    total_amount: row.get(9)?,
                    notes: row.get(10)?,
                    created_at: row.get(11)?,
                    sent_at: row.get(12)?,
                    error_message: row.get(13)?,
                    item_count: row.get(14)?,
                    partner_payment_term: None,
                })
            },
        )
        .map_err(|e| format!("Invoice not found: {}", e))?;

    Ok(invoice)
}

#[tauri::command]
pub fn delete_invoice(db: State<'_, Database>, invoice_id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Delete invoice items first
    conn.execute(
        "DELETE FROM invoice_items WHERE invoice_id = ?1",
        [&invoice_id],
    )
    .map_err(|e| e.to_string())?;

    // Delete invoice
    conn.execute("DELETE FROM invoices WHERE id = ?1", [&invoice_id])
        .map_err(|e| e.to_string())?;

    info!("Deleted invoice {}", invoice_id);
    Ok(())
}

// ==================== PRINT COMMANDS ====================

#[tauri::command]
pub fn get_available_printers() -> Result<Vec<String>, String> {
    #[cfg(target_os = "windows")]
    {
        // 1. Try WMIC first (Much faster than PowerShell)
        // wmic printer get name
        let wmic_output = std::process::Command::new("wmic")
            .args(&["printer", "get", "name"])
            .output();

        if let Ok(output) = wmic_output {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                let printers: Vec<String> = text
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty() && l.to_lowercase() != "name") // Filter header and empty lines
                    .collect();

                if !printers.is_empty() {
                    return Ok(printers);
                }
            }
        }

        // 2. Fallback to PowerShell if WMIC fails or returns no printers
        // Use Get-CimInstance which is generally preferred over Get-WmiObject
        let output = std::process::Command::new("powershell")
            .args(&[
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Get-CimInstance Win32_Printer | Select-Object -ExpandProperty Name",
            ])
            .output()
            .map_err(|e| format!("Failed to get printers: {}", e))?;

        if !output.status.success() {
            return Ok(vec!["Default".to_string()]);
        }

        let printers: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(if printers.is_empty() {
            vec!["Default".to_string()]
        } else {
            printers
        })
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(vec!["Default".to_string()])
    }
}

#[tauri::command]
pub async fn print_invoice_to_html(
    db: State<'_, Database>,
    invoice_id: String,
    printer_name: Option<String>,
) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    // Get invoice number first
    let invoice_number: i64 = conn
        .query_row(
            "SELECT invoice_number FROM invoices WHERE id = ?1",
            [&invoice_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Invoice not found: {}", e))?;

    // Fetch invoice details and payment term
    let (invoice, payment_term_days) = get_invoice_for_print(&conn, &invoice_id)?;
    
    info!("📅 Payment term retrieved from database for partner '{}': {:?}", invoice.partner_name, payment_term_days);
    
    // Get agent settings for delegate info
    let agent_settings_result = conn.query_row(
        "SELECT delegate_name, delegate_act FROM agent_settings WHERE id = 1",
        [],
        |row| Ok((row.get::<_, Option<String>>(0)?, row.get::<_, Option<String>>(1)?)),
    );
    
    let (delegate_name, delegate_act) = agent_settings_result.unwrap_or((None, None));

    // Fetch invoice items
    let mut stmt = conn
        .prepare(
            r#"
            SELECT ii.id, ii.product_id, p.name, ii.quantity, ii.unit_price, p.unit_of_measure, ii.total_price, p.procent_tva
            FROM invoice_items ii
            JOIN products p ON ii.product_id = p.id
            WHERE ii.invoice_id = ?1
            "#,
        )
        .map_err(|e| e.to_string())?;

    let items: Vec<InvoiceItem> = stmt
        .query_map([&invoice_id], |row| {
            // Parse TVA percentage from TEXT to f64
            let tva_percent: Option<f64> = match row.get::<_, Option<String>>(7)? {
                Some(s) => s.parse::<f64>().ok(),
                None => None,
            };
            
            Ok(InvoiceItem {
                id: row.get(0)?,
                invoice_id: invoice_id.clone(),
                product_id: row.get(1)?,
                product_name: row.get(2)?,
                quantity: row.get(3)?,
                unit_price: row.get(4)?,
                unit_of_measure: row.get(5)?,
                total_price: row.get(6)?,
                tva_percent,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    // Read logo and convert to base64
    let logo_base64 = read_logo_to_base64();
    
    // Use partner's payment term or default to 30 days
    let payment_days = payment_term_days.unwrap_or(30);
    
    info!("📅 Using payment term: {} days (partner: '{}', retrieved: {:?}, final: {})", 
        payment_days, invoice.partner_name, payment_term_days, payment_days);

    // Get carnet series from agent settings
    let carnet_series = conn.query_row(
        "SELECT carnet_series FROM agent_settings WHERE id = 1",
        [],
        |row| row.get::<_, Option<String>>(0)
    ).ok().flatten().unwrap_or_else(|| "FACTURA".to_string());

    // Get car number from agent settings
    let car_number = conn.query_row(
        "SELECT car_number FROM agent_settings WHERE id = 1",
        [],
        |row| row.get::<_, Option<String>>(0)
    ).ok().flatten();

    // Generate HTML
    let html = print_invoice::generate_invoice_html(
        &invoice, 
        &items, 
        invoice_number, 
        logo_base64.as_deref(),
        payment_days,
        delegate_name.as_deref(),
        delegate_act.as_deref(),
        car_number.as_deref(),
        &carnet_series
    );

    // Save to invoices folder in AppData
    let app_data_dir = dirs::config_dir()
        .ok_or("Could not find app data directory")?
        .join("facturi.softconsulting.com")
        .join("invoices");
    
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create invoices directory: {}", e))?;

    let html_file_path = app_data_dir.join(format!("factura_{}.html", invoice_id));
    let pdf_file_path = app_data_dir.join(format!("factura_{}.pdf", invoice_id));
    
    std::fs::write(&html_file_path, &html)
        .map_err(|e| format!("Failed to write HTML file: {}", e))?;

    let html_path_str = html_file_path.to_string_lossy().to_string();
    let pdf_path_str = pdf_file_path.to_string_lossy().to_string();
    
    info!("Generated invoice HTML at: {}", html_path_str);
    
    // Convert HTML to PDF using Edge (headless)
    #[cfg(target_os = "windows")]
    {
        // Try to generate PDF using available tools
        let mut pdf_generated = false;
        let mut print_file = html_path_str.clone();
        
        // Try Edge first (Windows 10+)
        let edge_paths = vec![
            "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
            "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe",
        ];
        
        for edge_path in edge_paths {
            if std::path::Path::new(edge_path).exists() {
                let file_url = format!("file:///{}", html_path_str.replace('\\', "/"));
                
                let output = std::process::Command::new(edge_path)
                    .args(&[
                        "--headless",
                        "--disable-gpu",
                        "--no-sandbox",
                        "--disable-dev-shm-usage",
                        &format!("--print-to-pdf={}", pdf_path_str),
                        &file_url,
                    ])
                    .output();
                
                match output {
                    Ok(result) => {
                        info!("Edge command executed. Status: {}", result.status);
                        if !result.stderr.is_empty() {
                            let stderr = String::from_utf8_lossy(&result.stderr);
                            info!("Edge stderr: {}", stderr);
                        }
                        
                        // Give Edge time to write the file (poll until fully written)
                        let mut waited = 0;
                        while waited < 5000 {
                            if wait_for_file_ready(&pdf_path_str, 1000, 300) {
                                pdf_generated = true;
                                print_file = pdf_path_str.clone();
                                info!("PDF generated successfully at: {}", pdf_path_str);
                                break;
                            }
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            waited += 100;
                        }
                        if pdf_generated {
                            break;
                        }
                    }
                    Err(e) => {
                        info!("Failed to use Edge: {}", e);
                    }
                }
            }
        }
        
        // If PDF generation failed, use HTML directly for printing
        if !pdf_generated {
            info!("PDF generation failed, will print HTML directly");
            print_file = html_path_str.clone();
        }
        
        // Print PDF using SumatraPDF
        let printer = printer_name.unwrap_or_else(|| String::from(""));
        
        // Check standard installation paths first
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
        
        // Also check bundled resources path
        let bundled_path = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.join("resources").join("SumatraPDF.exe")));
        
        let mut sumatra_paths = vec![
            format!(r"{}\AppData\Local\SumatraPDF\SumatraPDF.exe", user_profile),
            r"C:\Program Files\SumatraPDF\SumatraPDF.exe".to_string(),
            r"C:\Program Files (x86)\SumatraPDF\SumatraPDF.exe".to_string(),
        ];
        
        // Add bundled path if exists
        if let Some(bundled) = bundled_path {
            sumatra_paths.insert(0, bundled.to_string_lossy().to_string());
        }
        
        let mut sumatra_exe = None;
        for path in &sumatra_paths {
            if std::path::Path::new(path).exists() {
                sumatra_exe = Some(path.to_string());
                info!("Found SumatraPDF at: {}", path);
                break;
            }
        }
        
        // If not in standard paths, check portable version in app data
        if sumatra_exe.is_none() {
            let app_data_dir = dirs::data_dir()
                .ok_or("Could not get app data directory")?
                .join("facturi.softconsulting.com");
            let sumatra_portable = app_data_dir.join("tools").join("SumatraPDF.exe");
            
            if sumatra_portable.exists() {
                sumatra_exe = Some(sumatra_portable.to_string_lossy().to_string());
                info!("Found portable SumatraPDF");
            }
        }
        
        // Use SumatraPDF for printing
        if let Some(sumatra_path) = sumatra_exe {
            info!("Printing to '{}' using SumatraPDF", printer);

            let mut invoice_args: Vec<String> = Vec::new();
            if printer.trim().is_empty() {
                invoice_args.push("-print-to-default".to_string());
            } else {
                invoice_args.push("-print-to".to_string());
                invoice_args.push(printer.clone());
            }
            invoice_args.extend([
                "-print-settings".to_string(),
                "noscale".to_string(),
                print_file.clone(),
                "-silent".to_string(),
                "-exit-when-done".to_string(),
                "-exit-on-print".to_string(),
            ]);

            match std::process::Command::new(&sumatra_path).args(&invoice_args).spawn() {
                Ok(_) => info!("Invoice print job sent successfully to printer '{}': {}", printer, invoice_id),
                Err(e) => warn!("Invoice SumatraPDF print failed: {}", e),
            }

            match save_invoice_certificate_file(&invoice_id) {
                Ok((_cert_html_path, _cert_pdf_path, cert_print_file)) => {
                    std::thread::sleep(std::time::Duration::from_millis(400));

                    let mut cert_args: Vec<String> = Vec::new();
                    if printer.trim().is_empty() {
                        cert_args.push("-print-to-default".to_string());
                    } else {
                        cert_args.push("-print-to".to_string());
                        cert_args.push(printer.clone());
                    }
                    cert_args.extend([
                        "-print-settings".to_string(),
                        "noscale".to_string(),
                        cert_print_file.clone(),
                        "-silent".to_string(),
                        "-exit-when-done".to_string(),
                        "-exit-on-print".to_string(),
                    ]);

                    match std::process::Command::new(&sumatra_path).args(&cert_args).spawn() {
                        Ok(_) => info!("Certificate print job sent successfully to printer '{}': {}", printer, invoice_id),
                        Err(e) => warn!("Certificate SumatraPDF print failed: {}", e),
                    }
                }
                Err(e) => warn!("Certificate generation/print skipped: {}", e),
            }
        } else {
            info!("SumatraPDF not found. PDF saved at: {}", print_file);
        }
        
        let file_type = if pdf_generated { "PDF" } else { "HTML" };
        info!("Print dispatched ({}) to printer '{}': {}", file_type, printer, invoice_id);
    }
    
    #[cfg(target_os = "macos")]
    {
        // Use macOS print command on PDF or HTML
        std::process::Command::new("lp")
            .arg(&html_path_str)
            .spawn()
            .ok();
    }
    
    #[cfg(target_os = "linux")]
    {
        // Use Linux print command on PDF or HTML
        Command::new("lp")
            .arg(&html_path_str)
            .spawn()
            .map_err(|e| format!("Failed to print: {}", e))?;
    }
    
    Ok(pdf_path_str)
}

#[tauri::command]
pub async fn print_invoice_certificate(
    invoice_id: String,
    printer_name: Option<String>,
) -> Result<String, String> {
    let (_html_path, _pdf_path, target) = save_invoice_certificate_file(&invoice_id)?;
    let print_file = target.clone();
    let _is_pdf = print_file.ends_with(".pdf");

    info!("[CERT][PRINT] Printing certificate for invoice {} (file: {})", invoice_id, print_file);

    #[cfg(target_os = "windows")]
    {
        let printer = printer_name.unwrap_or_default();
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
        let bundled_path = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.join("resources").join("SumatraPDF.exe")));

        let mut sumatra_paths = vec![
            format!(r"{}\AppData\Local\SumatraPDF\SumatraPDF.exe", user_profile),
            r"C:\Program Files\SumatraPDF\SumatraPDF.exe".to_string(),
            r"C:\Program Files (x86)\SumatraPDF\SumatraPDF.exe".to_string(),
        ];

        if let Some(bundled) = bundled_path {
            sumatra_paths.insert(0, bundled.to_string_lossy().to_string());
        }

        let mut sumatra_exe = None;
        for path in &sumatra_paths {
            if std::path::Path::new(path).exists() {
                sumatra_exe = Some(path.to_string());
                break;
            }
        }

        if sumatra_exe.is_none() {
            let app_data_dir = dirs::data_dir()
                .ok_or("Could not get app data directory")?
                .join("facturi.softconsulting.com");
            let sumatra_portable = app_data_dir.join("tools").join("SumatraPDF.exe");
            if sumatra_portable.exists() {
                sumatra_exe = Some(sumatra_portable.to_string_lossy().to_string());
            }
        }

        if let Some(sumatra_path) = sumatra_exe {
            let mut command = std::process::Command::new(&sumatra_path);

            if printer.trim().is_empty() {
                command.arg("-print-to-default");
            } else {
                command.arg("-print-to").arg(&printer);
            }

            command
                .arg("-print-settings")
                .arg("noscale")
                .arg(&print_file)
                .arg("-silent")
                .arg("-exit-when-done")
                .arg("-exit-on-print")
                .spawn()
                .map_err(|e| format!("Failed to start print with SumatraPDF: {}", e))?;
                
            info!("[CERT][PRINT] Sent to SumatraPDF");
        } else {
            return Err("SumatraPDF not found. Instalează SumatraPDF sau configurează calea aplicației de printare.".to_string());
        }
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("lp")
            .arg(&print_file)
            .spawn()
            .ok();
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("lp")
            .arg(&print_file)
            .spawn()
            .map_err(|e| format!("Failed to print certificate: {}", e))?;
    }

    Ok(print_file)
}

#[tauri::command]
pub async fn preview_invoice_certificate(
        invoice_id: String,
) -> Result<String, String> {
        let (_html_path, _pdf_path, target) = save_invoice_certificate_file(&invoice_id)?;

        open::that(&target).map_err(|e| format!("Failed to open certificate preview: {}", e))?;
        Ok(target)
}

#[tauri::command]
pub async fn print_collection_to_html(
    db: State<'_, Database>,
    collection_id: String,
    printer_name: Option<String>,
) -> Result<String, String> {
    info!("[CHITANTE][PRINT] Start print_collection_to_html for collection_id={} printer={:?}", collection_id, printer_name);
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Check if this collection is part of a group
    let receipt_group_id: Option<String> = conn
        .query_row(
            "SELECT receipt_group_id FROM collections WHERE id = ?1 OR receipt_group_id = ?1 LIMIT 1",
            [&collection_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Collection not found: {}", e))?;

    let query = if receipt_group_id.is_some() {
        "SELECT
            id,
            receipt_group_id,
            receipt_series,
            receipt_number,
            id_partener,
            partner_name,
            numar_factura,
            serie_factura,
            cod_document,
            valoare,
            data_incasare,
            status,
            synced_at,
            error_message,
            created_at
         FROM collections
         WHERE receipt_group_id = ?1
         ORDER BY created_at DESC"
    } else {
        "SELECT
            id,
            receipt_group_id,
            receipt_series,
            receipt_number,
            id_partener,
            partner_name,
            numar_factura,
            serie_factura,
            cod_document,
            valoare,
            data_incasare,
            status,
            synced_at,
            error_message,
            created_at
         FROM collections
         WHERE id = ?1
         ORDER BY created_at DESC"
    };

    let param = if let Some(gid) = &receipt_group_id { gid } else { &collection_id };

    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([param], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, f64>(9)?,
                row.get::<_, String>(10)?,
                row.get::<_, String>(11)?,
                row.get::<_, Option<String>>(12)?,
                row.get::<_, Option<String>>(13)?,
                row.get::<_, String>(14)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut loaded = Vec::new();
    for row in rows {
        loaded.push(row.map_err(|e| e.to_string())?);
    }

    if loaded.is_empty() {
        return Err("Collection not found".to_string());
    }

    let first = &loaded[0];
    let group_total: f64 = loaded.iter().map(|r| r.9).sum();

    // Aggregate invoice references
    let invoice_refs: Vec<String> = loaded.iter()
        .map(|r| {
             let serie = r.7.as_deref().unwrap_or("").trim();
             let numar = r.6.as_deref().unwrap_or("").trim();
             if serie.is_empty() && numar.is_empty() {
                 return String::new();
             }
             if serie.is_empty() {
                 return numar.to_string();
             }
             if numar.is_empty() {
                 return serie.to_string();
             }
             format!("{}/{}", serie, numar)
        })
        .filter(|s| !s.is_empty())
        .collect();
    
    let invoice_ref_str = if invoice_refs.is_empty() {
        "Avans".to_string()
    } else {
        invoice_refs.join(", ")
    };

    let collection = Collection {
        id: first.0.clone(),
        // Use first row data for common fields
        id_partener: first.4.clone(),
        partner_name: first.5.clone(),
        // Store combined invoices in numar_factura so generate_receipt_html sees them
        numar_factura: Some(invoice_ref_str),
        serie_factura: Some(String::new()), // Clear series since it's merged
        cod_document: first.3.clone().or_else(|| first.8.clone()), // Use receipt number if available
        valoare: group_total,
        data_incasare: first.10.clone(),
        status: CollectionStatus::from(first.11.clone()),
        synced_at: first.12.clone(),
        error_message: first.13.clone(),
        created_at: first.14.clone(),
    };

    info!(
        "[CHITANTE][PRINT] Loaded GROUP collection id={} count={} val={} status={}",
        collection.id,
        loaded.len(),
        collection.valoare,
        collection.status.to_string()
    );

    let (agent_name, nume_casa, carnet_series) = conn
        .query_row(
            "SELECT agent_name, nume_casa, carnet_series FROM agent_settings WHERE id = 1",
            [],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            },
        )
        .unwrap_or((None, None, None));

    let issue_date = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&collection.data_incasare) {
        dt.format("%d.%m.%Y").to_string()
    } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&collection.data_incasare, "%Y-%m-%dT%H:%M:%S%.fZ") {
        dt.format("%d.%m.%Y").to_string()
    } else {
        chrono::Utc::now().format("%d.%m.%Y").to_string()
    };

    // Prioritize receipt_series from DB, then carnet_series, then fallback
    let doc_series = first.2.clone()
        .or(carnet_series)
        .unwrap_or_else(|| "CH".to_string());
    
    // Prioritize receipt_number from DB
    let doc_number = first.3.clone()
        .or_else(|| Some(collection.id.chars().take(8).collect::<String>()))
        .unwrap_or_else(|| "N/A".to_string());

    let agent_display = agent_name
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "N/A".to_string());

    let (partner_cui, partner_reg_com, partner_address, partner_localitate, partner_judet) =
        get_partner_receipt_info(&conn, &collection.id_partener);

    let (html_path_str, pdf_path_str) = save_receipt_html_file(
        &collection,
        &doc_series,
        &doc_number,
        &issue_date,
        Some(agent_display.as_str()),
        nume_casa.as_deref().unwrap_or("CASA LEI"),
        partner_address.as_deref(),
        partner_localitate.as_deref(),
        partner_judet.as_deref(),
        partner_cui.as_deref(),
        partner_reg_com.as_deref(),
        &collection_id,
    )?;

    #[cfg(target_os = "windows")]
    {
        let mut pdf_generated = false;
        let mut print_file = html_path_str.clone();

        let edge_paths = vec![
            "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
            "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe",
        ];

        for edge_path in edge_paths {
            if std::path::Path::new(edge_path).exists() {
                let file_url = format!("file:///{}", html_path_str.replace('\\', "/"));

                let output = std::process::Command::new(edge_path)
                    .args(&[
                        "--headless",
                        "--disable-gpu",
                        "--no-sandbox",
                        "--disable-dev-shm-usage",
                        &format!("--print-to-pdf={}", pdf_path_str),
                        &file_url,
                    ])
                    .output();

                if let Ok(result) = output {
                    info!("Receipt Edge command executed. Status: {}", result.status);
                    let mut waited = 0;
                    while waited < 5000 {
                        if wait_for_file_ready(&pdf_path_str, 1000, 300) {
                            pdf_generated = true;
                            print_file = pdf_path_str.clone();
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        waited += 100;
                    }
                    if pdf_generated {
                        break;
                    }
                }
            }
        }

        if !pdf_generated {
            print_file = html_path_str.clone();
        }

        let printer = printer_name.unwrap_or_default();
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
        let bundled_path = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.join("resources").join("SumatraPDF.exe")));

        let mut sumatra_paths = vec![
            format!(r"{}\AppData\Local\SumatraPDF\SumatraPDF.exe", user_profile),
            r"C:\Program Files\SumatraPDF\SumatraPDF.exe".to_string(),
            r"C:\Program Files (x86)\SumatraPDF\SumatraPDF.exe".to_string(),
        ];

        if let Some(bundled) = bundled_path {
            sumatra_paths.insert(0, bundled.to_string_lossy().to_string());
        }

        let mut sumatra_exe = None;
        for path in &sumatra_paths {
            if std::path::Path::new(path).exists() {
                sumatra_exe = Some(path.to_string());
                break;
            }
        }

        if sumatra_exe.is_none() {
            let app_data_dir = dirs::data_dir()
                .ok_or("Could not get app data directory")?
                .join("facturi.softconsulting.com");
            let sumatra_portable = app_data_dir.join("tools").join("SumatraPDF.exe");
            if sumatra_portable.exists() {
                sumatra_exe = Some(sumatra_portable.to_string_lossy().to_string());
            }
        }

        if let Some(sumatra_path) = sumatra_exe {
            let mut command = std::process::Command::new(&sumatra_path);

            if printer.trim().is_empty() {
                command.arg("-print-to-default");
            } else {
                command.arg("-print-to").arg(&printer);
            }

            command
                .arg("-print-settings")
                .arg("noscale")
                .arg(&print_file)
                .arg("-silent")
                .arg("-exit-when-done")
                .arg("-exit-on-print")
                .spawn()
                .map_err(|e| format!("Failed to start print with SumatraPDF: {}", e))?;
        } else {
            return Err("SumatraPDF not found. Instalează SumatraPDF sau configurează calea aplicației de printare.".to_string());
        }
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("lp")
            .arg(&html_path_str)
            .spawn()
            .ok();
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("lp")
            .arg(&html_path_str)
            .spawn()
            .map_err(|e| format!("Failed to print receipt: {}", e))?;
    }

    Ok(pdf_path_str)
}

fn get_invoice_for_print(
    conn: &rusqlite::Connection,
    invoice_id: &str,
) -> Result<(Invoice, Option<i64>), String> {
    conn.query_row(
        r#"
        SELECT
            i.id, i.partner_id, p.name, p.cif, p.reg_com, i.location_id, l.name, l.address,
            i.status, i.total_amount, i.notes, i.created_at, i.sent_at, i.error_message,
            (SELECT COUNT(*) FROM invoice_items WHERE invoice_id = i.id),
            p.scadenta_la_vanzare
        FROM invoices i
        JOIN partners p ON i.partner_id = p.id
        JOIN locations l ON i.location_id = l.id
        WHERE i.id = ?1
        "#,
        [invoice_id],
        |row| {
            let invoice = Invoice {
                id: row.get(0)?,
                partner_id: row.get(1)?,
                partner_name: row.get(2)?,
                partner_cif: row.get(3)?,
                partner_reg_com: row.get(4)?,
                location_id: row.get(5)?,
                location_name: row.get(6)?,
                location_address: row.get(7)?,
                status: InvoiceStatus::from(row.get::<_, String>(8)?),
                total_amount: row.get(9)?,
                notes: row.get(10)?,
                created_at: row.get(11)?,
                sent_at: row.get(12)?,
                error_message: row.get(13)?,
                item_count: row.get(14)?,
                partner_payment_term: None,
            };
            
            // Parse scadenta_la_vanzare to i64 (days)
            let scadenta_str: Option<String> = row.get(15)?;
            info!("🔍 Raw scadenta_la_vanzare from DB for partner '{}': {:?}", invoice.partner_name, scadenta_str);
            
            let scadenta: Option<i64> = scadenta_str
                .and_then(|s| {
                    let parsed = s.trim().parse::<i64>().ok();
                    info!("🔍 Parsed scadenta_la_vanzare: '{}' -> {:?}", s.trim(), parsed);
                    parsed
                });
            
            Ok((invoice, scadenta))
        },
    )
    .map_err(|e| format!("Invoice not found: {}", e))
}

// ==================== AGENT SETTINGS COMMANDS ====================

#[tauri::command]
pub fn get_agent_settings(db: State<'_, Database>) -> Result<AgentSettings, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let result = conn.query_row(
        "SELECT agent_name, carnet_series, simbol_carnet_livr, simbol_gestiune_livrare, tip_contabil, cod_carnet, cod_carnet_livr, cod_delegat, delegate_name, delegate_act, car_number, invoice_number_start, invoice_number_end, invoice_number_current, marca_agent, nume_casa, auto_sync_collections_enabled, auto_sync_collections_time, receipt_series, receipt_number_start, receipt_number_end, receipt_number_current FROM agent_settings WHERE id = 1",
        [],
        |row| {
            let auto_sync_enabled: Option<i32> = row.get(16)?;
            Ok(AgentSettings {
                agent_name: row.get(0)?,
                carnet_series: row.get(1)?,
                simbol_carnet_livr: row.get(2)?,
                simbol_gestiune_livrare: row.get(3)?,
                tip_contabil: row.get(4)?,
                cod_carnet: row.get(5)?,
                cod_carnet_livr: row.get(6)?,
                cod_delegat: row.get(7)?,
                delegate_name: row.get(8)?,
                delegate_act: row.get(9)?,
                car_number: row.get(10)?,
                invoice_number_start: row.get(11)?,
                invoice_number_end: row.get(12)?,
                invoice_number_current: row.get(13)?,
                marca_agent: row.get(14)?,
                nume_casa: row.get(15)?,
                auto_sync_collections_enabled: auto_sync_enabled.map(|v| v != 0),
                auto_sync_collections_time: row.get(17)?,
                receipt_series: row.get(18)?,
                receipt_number_start: row.get(19)?,
                receipt_number_end: row.get(20)?,
                receipt_number_current: row.get(21)?,
            })
        },
    );

    match result {
        Ok(settings) => Ok(settings),
        Err(_) => Ok(AgentSettings {
            agent_name: None,
            carnet_series: None,
            simbol_carnet_livr: None,
            simbol_gestiune_livrare: None,
            tip_contabil: Some("valoare".to_string()),
            cod_carnet: None,
            cod_carnet_livr: None,
            cod_delegat: None,
            delegate_name: None,
            delegate_act: None,
            car_number: None,
            invoice_number_start: Some(1),
            invoice_number_end: Some(99999),
            invoice_number_current: Some(1),
            marca_agent: None,
            nume_casa: None,
            auto_sync_collections_enabled: Some(false),
            auto_sync_collections_time: Some("23:00".to_string()),
            receipt_series: None,
            receipt_number_start: Some(1),
            receipt_number_end: Some(99999),
            receipt_number_current: Some(1),
        }),
    }
}

#[tauri::command]
pub fn save_agent_settings(
    db: State<'_, Database>,
    agent_name: Option<String>,
    carnet_series: Option<String>,
    simbol_carnet_livr: Option<String>,
    simbol_gestiune_livrare: Option<String>,
    tip_contabil: Option<String>,
    cod_carnet: Option<String>,
    cod_carnet_livr: Option<String>,
    cod_delegat: Option<String>,
    delegate_name: Option<String>,
    delegate_act: Option<String>,
    car_number: Option<String>,
    invoice_number_start: Option<i64>,
    invoice_number_end: Option<i64>,
    invoice_number_current: Option<i64>,
    marca_agent: Option<String>,
    nume_casa: Option<String>,
    auto_sync_collections_enabled: Option<bool>,
    auto_sync_collections_time: Option<String>,
    receipt_series: Option<String>,
    receipt_number_start: Option<i64>,
    receipt_number_end: Option<i64>,
    receipt_number_current: Option<i64>,
) -> Result<AgentSettings, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();

    // Smart logic for invoice numbering:
    // If invoice_number_start is provided and current is less than start, set current = start
    let final_invoice_current = match (invoice_number_start, invoice_number_current) {
        (Some(start), Some(current)) if current < start => {
            info!("Auto-adjusting invoice_number_current from {} to {} (matching start)", current, start);
            Some(start)
        },
        (Some(start), None) => {
            info!("Initializing invoice_number_current to {} (start value)", start);
            Some(start)
        },
        _ => invoice_number_current,
    };

    // Same logic for receipt numbering
    let final_receipt_current = match (receipt_number_start, receipt_number_current) {
        (Some(start), Some(current)) if current < start => {
            info!("Auto-adjusting receipt_number_current from {} to {} (matching start)", current, start);
            Some(start)
        },
        (Some(start), None) => {
            info!("Initializing receipt_number_current to {} (start value)", start);
            Some(start)
        },
        _ => receipt_number_current,
    };

    // Convert bool to i32 for SQLite
    let auto_sync_enabled_int = auto_sync_collections_enabled.map(|v| if v { 1 } else { 0 });

    let normalized_tip_contabil = tip_contabil
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or(Some("valoare".to_string()));

    conn.execute(
        "INSERT INTO agent_settings (id, agent_name, carnet_series, simbol_carnet_livr, simbol_gestiune_livrare, tip_contabil, cod_carnet, cod_carnet_livr, cod_delegat, delegate_name, delegate_act, car_number, invoice_number_start, invoice_number_end, invoice_number_current, marca_agent, nume_casa, auto_sync_collections_enabled, auto_sync_collections_time, receipt_series, receipt_number_start, receipt_number_end, receipt_number_current, updated_at) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23) \
         ON CONFLICT(id) DO UPDATE SET agent_name = excluded.agent_name, carnet_series = excluded.carnet_series, simbol_carnet_livr = excluded.simbol_carnet_livr, simbol_gestiune_livrare = excluded.simbol_gestiune_livrare, tip_contabil = excluded.tip_contabil, cod_carnet = excluded.cod_carnet, cod_carnet_livr = excluded.cod_carnet_livr, cod_delegat = excluded.cod_delegat, delegate_name = excluded.delegate_name, delegate_act = excluded.delegate_act, car_number = excluded.car_number, invoice_number_start = excluded.invoice_number_start, invoice_number_end = excluded.invoice_number_end, invoice_number_current = excluded.invoice_number_current, marca_agent = excluded.marca_agent, nume_casa = excluded.nume_casa, auto_sync_collections_enabled = excluded.auto_sync_collections_enabled, auto_sync_collections_time = excluded.auto_sync_collections_time, receipt_series = excluded.receipt_series, receipt_number_start = excluded.receipt_number_start, receipt_number_end = excluded.receipt_number_end, receipt_number_current = excluded.receipt_number_current, updated_at = excluded.updated_at",
        params![
            agent_name, carnet_series, simbol_carnet_livr, simbol_gestiune_livrare,
            normalized_tip_contabil, cod_carnet, cod_carnet_livr, cod_delegat, delegate_name,
            delegate_act, car_number, invoice_number_start, invoice_number_end, final_invoice_current,
            marca_agent, nume_casa, auto_sync_enabled_int, auto_sync_collections_time,
            receipt_series, receipt_number_start, receipt_number_end, final_receipt_current,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(AgentSettings {
        agent_name,
        carnet_series,
        simbol_carnet_livr,
        simbol_gestiune_livrare,
        tip_contabil: normalized_tip_contabil,
        cod_carnet,
        cod_carnet_livr,
        cod_delegat,
        delegate_name,
        delegate_act,
        car_number,
        invoice_number_start: invoice_number_start.map(|v| v as i32),
        invoice_number_end: invoice_number_end.map(|v| v as i32),
        invoice_number_current: final_invoice_current.map(|v| v as i32),
        marca_agent,
        nume_casa,
        auto_sync_collections_enabled,
        auto_sync_collections_time,
        receipt_series,
        receipt_number_start: receipt_number_start.map(|v| v as i32),
        receipt_number_end: receipt_number_end.map(|v| v as i32),
        receipt_number_current: final_receipt_current.map(|v| v as i32),
    })
}

// ==================== DEBUG COMMANDS ====================

#[tauri::command]
pub fn debug_db_counts(db: State<'_, Database>) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    let partners_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM partners", [], |row| row.get(0))
        .unwrap_or(0);
    
    let products_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM products", [], |row| row.get(0))
        .unwrap_or(0);
    
    let partners_list: Vec<(String, String, Option<String>)> = conn
        .prepare("SELECT id, name, inactiv FROM partners ORDER BY name")
        .map_err(|e| e.to_string())?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    
    let products_list: Vec<(String, String)> = conn
        .prepare("SELECT id, name FROM products ORDER BY name")
        .map_err(|e| e.to_string())?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    
    let mut result = format!("Database counts:\nPartners: {}\nProducts: {}\n\n", partners_count, products_count);
    result.push_str("Partners:\n");
    for (id, name, inactiv) in partners_list {
        result.push_str(&format!("  - {} | {} | inactiv: {:?}\n", id, name, inactiv));
    }
    result.push_str("\nProducts:\n");
    for (id, name) in products_list {
        result.push_str(&format!("  - {} | {}\n", id, name));
    }
    
    Ok(result)
}

#[tauri::command]
pub fn debug_partner_payment_terms(db: State<'_, Database>, partner_id: String) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    let result: Result<(String, Option<String>, Option<String>, Option<String>), _> = conn.query_row(
        "SELECT name, cif, reg_com, scadenta_la_vanzare FROM partners WHERE id = ?1",
        [&partner_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    );
    
    match result {
        Ok((name, cif, reg_com, scadenta)) => {
            let mut output = format!("🔍 Partner Debug Info\n\n");
            output.push_str(&format!("ID: {}\n", partner_id));
            output.push_str(&format!("Name: {}\n", name));
            output.push_str(&format!("CIF: {}\n", cif.unwrap_or("N/A".to_string())));
            output.push_str(&format!("Reg.Com: {}\n", reg_com.unwrap_or("N/A".to_string())));
            output.push_str(&format!("\n📅 Payment Terms (scadenta_la_vanzare):\n"));
            output.push_str(&format!("  Raw value: {:?}\n", scadenta));
            
            if let Some(s) = &scadenta {
                match s.trim().parse::<i64>() {
                    Ok(days) => output.push_str(&format!("  Parsed: {} days ✅\n", days)),
                    Err(e) => output.push_str(&format!("  Parse ERROR: {} ❌\n  String: '{}'\n", e, s)),
                }
            } else {
                output.push_str("  Value: NULL (will use default 10 days) ⚠️\n");
            }
            
            Ok(output)
        }
        Err(e) => Err(format!("Partner not found: {}", e))
    }
}

#[tauri::command]
pub async fn update_all_partners_payment_terms(
    db: State<'_, Database>,
    new_days: String,
) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    info!("🔄 Updating all partners payment terms to {} days", new_days);
    
    let updated = conn.execute(
        "UPDATE partners SET scadenta_la_vanzare = ?1",
        [&new_days],
    ).map_err(|e| e.to_string())?;
    
    info!("✅ Updated {} partners with new payment term: {} days", updated, new_days);
    
    Ok(format!("Successfully updated {} partners to {} days payment term", updated, new_days))
}

#[tauri::command]
pub async fn open_external_link(url: String) -> Result<(), String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("Invalid URL protocol".to_string());
    }
    open::that(url).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_report_html(report_name: String, html_content: String) -> Result<String, String> {
    let safe_name = report_name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();

    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();

    let reports_dir = dirs::config_dir()
        .ok_or("Could not find app data directory")?
        .join("facturi.softconsulting.com")
        .join("reports");

    std::fs::create_dir_all(&reports_dir)
        .map_err(|e| format!("Failed to create reports directory: {}", e))?;

    let file_path = reports_dir.join(format!("{}_{}.html", safe_name, timestamp));

    std::fs::write(&file_path, html_content)
        .map_err(|e| format!("Failed to save report HTML: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn print_report_html(
    report_name: String,
    html_content: String,
    printer_name: Option<String>,
) -> Result<String, String> {
    let saved_path = save_report_html(report_name, html_content)?;
    let html_path_str = saved_path.clone();

    #[cfg(target_os = "windows")]
    {
        let reports_dir = std::path::Path::new(&saved_path)
            .parent()
            .ok_or("Invalid report file path")?;

        let stem = std::path::Path::new(&saved_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("Invalid report file name")?;

        let pdf_path = reports_dir.join(format!("{}.pdf", stem));
        let pdf_path_str = pdf_path.to_string_lossy().to_string();

        let mut print_file = html_path_str.clone();

        let edge_paths = vec![
            "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
            "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe",
        ];

        for edge_path in edge_paths {
            if std::path::Path::new(edge_path).exists() {
                let file_url = format!("file:///{}", html_path_str.replace('\\', "/"));
                if let Ok(result) = std::process::Command::new(edge_path)
                    .args(&[
                        "--headless",
                        "--disable-gpu",
                        "--no-sandbox",
                        "--disable-dev-shm-usage",
                        &format!("--print-to-pdf={}", pdf_path_str),
                        &file_url,
                    ])
                    .output()
                {
                    info!("Report Edge command executed. Status: {}", result.status);
                    if wait_for_file_ready(&pdf_path_str, 5000, 300) {
                        print_file = pdf_path_str.clone();
                        break;
                    }
                }
            }
        }

        let printer = printer_name.unwrap_or_default();
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
        let bundled_path = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.join("resources").join("SumatraPDF.exe")));

        let mut sumatra_paths = vec![
            format!(r"{}\AppData\Local\SumatraPDF\SumatraPDF.exe", user_profile),
            r"C:\Program Files\SumatraPDF\SumatraPDF.exe".to_string(),
            r"C:\Program Files (x86)\SumatraPDF\SumatraPDF.exe".to_string(),
        ];

        if let Some(bundled) = bundled_path {
            sumatra_paths.insert(0, bundled.to_string_lossy().to_string());
        }

        let mut sumatra_exe = None;
        for path in &sumatra_paths {
            if std::path::Path::new(path).exists() {
                sumatra_exe = Some(path.to_string());
                break;
            }
        }

        if sumatra_exe.is_none() {
            let app_data_dir = dirs::data_dir()
                .ok_or("Could not get app data directory")?
                .join("facturi.softconsulting.com");
            let sumatra_portable = app_data_dir.join("tools").join("SumatraPDF.exe");
            if sumatra_portable.exists() {
                sumatra_exe = Some(sumatra_portable.to_string_lossy().to_string());
            }
        }

        if let Some(sumatra_path) = sumatra_exe {
            let mut command = std::process::Command::new(&sumatra_path);
            if printer.trim().is_empty() {
                command.arg("-print-to-default");
            } else {
                command.arg("-print-to").arg(&printer);
            }

            command
                .arg("-print-settings")
                .arg("noscale")
                .arg(&print_file)
                .arg("-silent")
                .arg("-exit-when-done")
                .arg("-exit-on-print")
                .spawn()
                .map_err(|e| format!("Failed to print report: {}", e))?;
        } else {
            return Err("SumatraPDF not found. Instalează SumatraPDF sau configurează calea aplicației de printare.".to_string());
        }
    }

    #[cfg(target_os = "macos")]
    {
        let mut cmd = std::process::Command::new("lp");
        if let Some(printer) = printer_name {
            if !printer.trim().is_empty() {
                cmd.arg("-d").arg(printer);
            }
        }
        cmd.arg(&html_path_str)
            .spawn()
            .map_err(|e| format!("Failed to print report: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        let mut cmd = std::process::Command::new("lp");
        if let Some(printer) = printer_name {
            if !printer.trim().is_empty() {
                cmd.arg("-d").arg(printer);
            }
        }
        cmd.arg(&html_path_str)
            .spawn()
            .map_err(|e| format!("Failed to print report: {}", e))?;
    }

    Ok(saved_path)
}


// ==================== COLLECTION COMMANDS ====================

#[tauri::command]
pub async fn sync_client_balances(
    _app: tauri::AppHandle,
    db: State<'_, Database>,
) -> Result<String, String> {
    let settings = get_agent_settings(db.clone())?;
    let marca_agent = settings
        .marca_agent
        .and_then(|value| {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });

    if let Some(marca) = &marca_agent {
        info!("Sync client balances with MarcaAgent filter: {}", marca);
    } else {
        info!("MarcaAgent not set; syncing client balances without filter");
    }

    let partner_ids = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT id FROM partners WHERE simbol_clasa = 'AGENTI' OR clasa = 'AGENTI'")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;

        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(|e| e.to_string())?);
        }
        ids
    };

    if partner_ids.is_empty() {
        return Err("Nu există parteneri locali. Fă mai întâi sincronizarea partenerilor.".to_string());
    }

    let partner_set: HashSet<String> = partner_ids
        .into_iter()
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty())
        .collect();

    // Use ApiClient to fetch balances
    let api = api_client::ApiClient::from_default()?;

    // Fetch all balances from WME for the configured agent, then keep only local partners
    let solduri = api
        .get_all_solduri(marca_agent)
        .await
        .map_err(|e: String| e)?;

    let total_solduri_fetched = solduri.len();

    let solduri: Vec<api_client::SoldInfo> = solduri
        .into_iter()
        .filter(|s| {
            if let Some(id_partener) = &s.id_partener {
                partner_set.contains(id_partener.trim())
            } else {
                false
            }
        })
        .filter(|s| api_client::parse_f64(&s.rest) > 0.0)
        .collect();

    info!(
        "[SOLDURI] fetched={}, local_partners={}, kept_after_filters={}",
        total_solduri_fetched,
        partner_set.len(),
        solduri.len()
    );

    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Transaction to replace balances
    conn.execute("DELETE FROM client_balances", []).map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "INSERT INTO client_balances (
            id_partener, cod_fiscal, denumire, tip_document, cod_document, 
            serie, numar, data, valoare, rest, termen, moneda, 
            sediu, id_sediu, curs, observatii, cod_obligatie, marca_agent, synced_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19
        )"
    ).map_err(|e| e.to_string())?;

    let now = Utc::now().to_rfc3339();

    for s in solduri {
        // Parse fields using helper or manually
        // We use api_client::parse_f64 if available public, else local parse
        // Assuming api_client::parse_f64 is public
        let valoare = api_client::parse_f64(&s.valoare);
        let rest = api_client::parse_f64(&s.rest);
        let curs = api_client::parse_f64(&s.curs);
        
        // Parse date for sorting/display usage if needed, but we store as string
        // s.data is Option<String>
        
        let id_partener_normalized = s.id_partener.map(|v| v.trim().to_string());

        stmt.execute(params![
            id_partener_normalized,
            s.cod_fiscal,
            s.denumire,
            s.tip_document,
            s.cod_document,
            s.serie,
            s.numar,
            s.data,
            valoare,
            rest,
            s.termen,
            s.moneda,
            s.sediu,
            s.id_sediu,
            curs,
            s.observatii,
            s.cod_obligatie,
            s.marca_agent,
            now
        ]).map_err(|e| e.to_string())?;
    }

    Ok(format!("Synced client balances"))
}

#[tauri::command]
pub fn get_client_balances(
    db: State<'_, Database>,
    partner_id: Option<String>,
) -> Result<Vec<ClientBalance>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    // Combine synced balances from WME with local invoices from DB.
    // Local collections still in-flight (pending/sending) are subtracted from remaining amount.
    // An invoice disappears only when the local collected total reaches full amount.
    let mut query = "SELECT
        q.id, q.id_partener, q.cod_fiscal, q.denumire, q.tip_document, q.cod_document,
        q.serie, q.numar, q.data, q.valoare, q.rest, q.termen, q.moneda,
        q.sediu, q.id_sediu, q.curs, q.observatii, q.cod_obligatie, q.marca_agent, q.synced_at
        FROM (
            SELECT 
                cb.id, cb.id_partener, cb.cod_fiscal, cb.denumire, cb.tip_document, cb.cod_document,
                cb.serie, cb.numar, cb.data, cb.valoare,
                CASE
                    WHEN COALESCE(cb.rest, 0) - COALESCE(c.total_collected, 0) > 0
                        THEN COALESCE(cb.rest, 0) - COALESCE(c.total_collected, 0)
                    ELSE 0
                END AS rest,
                cb.termen, cb.moneda,
                cb.sediu, cb.id_sediu, cb.curs, cb.observatii, cb.cod_obligatie, cb.marca_agent, cb.synced_at
            FROM client_balances cb
            LEFT JOIN (
                SELECT
                    id_partener,
                    COALESCE(serie_factura, '') AS serie_factura,
                    COALESCE(numar_factura, '') AS numar_factura,
                    COALESCE(cod_document, '') AS cod_document,
                    SUM(valoare) AS total_collected
                FROM collections
                WHERE status IN ('pending', 'sending', 'synced')
                GROUP BY id_partener, COALESCE(serie_factura, ''), COALESCE(numar_factura, ''), COALESCE(cod_document, '')
            ) c ON (
                cb.id_partener = c.id_partener AND
                COALESCE(cb.serie, '') = c.serie_factura AND
                COALESCE(cb.numar, '') = c.numar_factura AND
                COALESCE(cb.cod_document, '') = c.cod_document
            )

            UNION ALL

            SELECT
                NULL AS id,
                i.partner_id AS id_partener,
                p.cif AS cod_fiscal,
                p.name AS denumire,
                'FACTURA' AS tip_document,
                CAST(i.invoice_number AS TEXT) AS cod_document,
                COALESCE((SELECT carnet_series FROM agent_settings WHERE id = 1), 'FACTURA') AS serie,
                CAST(i.invoice_number AS TEXT) AS numar,
                i.created_at AS data,
                -- Calculate Gross Total (Total + VAT) for internal invoices
                (
                    SELECT COALESCE(SUM(ii.total_price * (1.0 + COALESCE(CAST(p.procent_tva AS REAL), 0) / 100.0)), 0)
                    FROM invoice_items ii
                    JOIN products p ON p.id = ii.product_id
                    WHERE ii.invoice_id = i.id
                ) AS valoare,
                CASE
                    WHEN (
                        SELECT COALESCE(SUM(ii.total_price * (1.0 + COALESCE(CAST(p.procent_tva AS REAL), 0) / 100.0)), 0)
                        FROM invoice_items ii
                        JOIN products p ON p.id = ii.product_id
                        WHERE ii.invoice_id = i.id
                    ) - COALESCE(c2.total_collected, 0) > 0.01
                        THEN (
                            SELECT COALESCE(SUM(ii.total_price * (1.0 + COALESCE(CAST(p.procent_tva AS REAL), 0) / 100.0)), 0)
                            FROM invoice_items ii
                            JOIN products p ON p.id = ii.product_id
                            WHERE ii.invoice_id = i.id
                        ) - COALESCE(c2.total_collected, 0)
                    ELSE 0
                END AS rest,
                replace(
                    datetime(
                        replace(substr(i.created_at, 1, 19), 'T', ' '),
                        '+' || COALESCE(NULLIF(trim(p.scadenta_la_vanzare), ''), '30') || ' days'
                    ),
                    ' ',
                    'T'
                ) AS termen,
                'RON' AS moneda,
                l.name AS sediu,
                l.id_sediu AS id_sediu,
                1.0 AS curs,
                i.notes AS observatii,
                NULL AS cod_obligatie,
                (SELECT marca_agent FROM agent_settings WHERE id = 1) AS marca_agent,
                i.created_at AS synced_at
            FROM invoices i
            JOIN partners p ON p.id = i.partner_id
            JOIN locations l ON l.id = i.location_id
            LEFT JOIN (
                SELECT
                    id_partener,
                    COALESCE(numar_factura, '') AS numar_factura,
                    COALESCE(cod_document, '') AS cod_document,
                    SUM(valoare) AS total_collected
                FROM collections
                WHERE status IN ('pending', 'sending', 'synced')
                GROUP BY id_partener, COALESCE(numar_factura, ''), COALESCE(cod_document, '')
            ) c2 ON (
                c2.id_partener = i.partner_id AND
                c2.numar_factura = CAST(i.invoice_number AS TEXT) AND
                c2.cod_document = CAST(i.invoice_number AS TEXT)
            )
            WHERE i.status IN ('pending', 'sending', 'sent', 'failed')
              AND NOT EXISTS (
                SELECT 1
                FROM client_balances cb2
                WHERE cb2.id_partener = i.partner_id
                  AND COALESCE(cb2.numar, '') = CAST(i.invoice_number AS TEXT)
              )
        ) q
        WHERE COALESCE(q.rest, 0) > 0".to_string();
    
    let mut params: Vec<String> = Vec::new();
    
    if let Some(pid) = partner_id {
        query.push_str(" AND TRIM(q.id_partener) = TRIM(?1)");
        params.push(pid);
    }

    query.push_str(" ORDER BY CASE WHEN date(q.termen) < date('now', 'start of day') THEN 0 ELSE 1 END, date(q.termen) ASC");
    
    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    
    let balances = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
        Ok(ClientBalance {
            id: row.get(0)?,
            id_partener: row.get(1)?,
            cod_fiscal: row.get(2)?,
            denumire: row.get(3)?,
            tip_document: row.get(4)?,
            cod_document: row.get(5)?,
            serie: row.get(6)?,
            numar: row.get(7)?,
            data: row.get(8)?,
            valoare: row.get(9)?,
            rest: row.get(10)?,
            termen: row.get(11)?,
            moneda: row.get(12)?,
            sediu: row.get(13)?,
            id_sediu: row.get(14)?,
            curs: row.get(15)?,
            observatii: row.get(16)?,
            cod_obligatie: row.get(17)?,
            marca_agent: row.get(18)?,
            synced_at: row.get(19)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut result = Vec::new();
    for b in balances {
        result.push(b.map_err(|e| e.to_string())?);
    }
    
    Ok(result)
}

#[tauri::command]
pub fn record_collection(
    db: State<'_, Database>,
    collection: Collection,
) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let (receipt_series_opt, carnet_series_opt): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT receipt_series, carnet_series FROM agent_settings WHERE id = 1",
            [],
            |row| Ok((row.get(0).ok(), row.get(1).ok()))
        )
        .unwrap_or_default();

    let receipt_series = receipt_series_opt
        .clone()
        .filter(|s| !s.trim().is_empty())
        .or(carnet_series_opt)
        .unwrap_or_else(|| "CH".to_string());

    let receipt_number = generate_receipt_number(&conn)?;
    
    // Check if there's already a pending or sending collection for this invoice
    let existing_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM collections WHERE 
         id_partener = ?1 AND serie_factura = ?2 AND numar_factura = ?3 AND cod_document = ?4 AND
         (status = 'pending' OR status = 'sending')",
        params![&collection.id_partener, &collection.serie_factura, &collection.numar_factura, &collection.cod_document],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;
    
    if existing_count > 0 {
        return Err("Există deja o încasare în curs de procesare pentru această factură".to_string());
    }
    
    // Ensure ID is generated if not provided (though frontend should provide UUID)
    let id = if collection.id.is_empty() {
        Uuid::new_v4().to_string()
    } else {
        collection.id
    };
    
    conn.execute(
        "INSERT INTO collections (
            id, receipt_group_id, receipt_series, receipt_number,
            id_partener, partner_name, numar_factura, serie_factura,
            cod_document, valoare, data_incasare, status, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            id.clone(),
            id,
            receipt_series,
            receipt_number,
            collection.id_partener,
            collection.partner_name,
            collection.numar_factura,
            collection.serie_factura,
            collection.cod_document,
            collection.valoare,
            collection.data_incasare,
            "pending",
            Utc::now().to_rfc3339()
        ]
    ).map_err(|e| e.to_string())?;
    
    Ok(id)
}

#[tauri::command]
pub fn record_collection_group(
    db: State<'_, Database>,
    request: CreateCollectionGroupRequest,
) -> Result<String, String> {
    let partner_id = request.id_partener.trim().to_string();
    if partner_id.is_empty() {
        return Err("Partener invalid pentru încasare".to_string());
    }

    if request.allocations.is_empty() {
        return Err("Selectează cel puțin o factură".to_string());
    }

    let current_balances = get_client_balances(db.clone(), Some(partner_id.clone()))?;
    let mut remaining_map: HashMap<String, f64> = HashMap::new();
    for balance in current_balances {
        let key = build_invoice_key(
            &balance.id_partener,
            &balance.serie,
            &balance.numar,
            &balance.cod_document,
        );
        remaining_map.insert(key, balance.rest.unwrap_or(0.0));
    }

    for allocation in &request.allocations {
        if allocation.valoare <= 0.0 {
            return Err("Valoarea pe fiecare factură trebuie să fie mai mare decât 0".to_string());
        }

        let key = build_invoice_key(
            &partner_id,
            &allocation.serie_factura,
            &allocation.numar_factura,
            &allocation.cod_document,
        );

        let remaining = remaining_map
            .get(&key)
            .copied()
            .ok_or_else(|| {
                format!(
                    "Factura {} {} nu mai are sold disponibil",
                    allocation.serie_factura.clone().unwrap_or_default(),
                    allocation.numar_factura.clone().unwrap_or_default()
                )
            })?;

        if allocation.valoare - remaining > 0.0001 {
            return Err(format!(
                "Valoarea introdusă depășește soldul disponibil ({:.2}) pentru factura {} {}",
                remaining,
                allocation.serie_factura.clone().unwrap_or_default(),
                allocation.numar_factura.clone().unwrap_or_default()
            ));
        }
    }

    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let (receipt_series_opt, carnet_series_opt): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT receipt_series, carnet_series FROM agent_settings WHERE id = 1",
            [],
            |row| Ok((row.get(0).ok(), row.get(1).ok()))
        )
        .unwrap_or_default();

    let receipt_series = receipt_series_opt
        .clone()
        .filter(|s| !s.trim().is_empty())
        .or(carnet_series_opt)
        .unwrap_or_else(|| "CH".to_string());

    let receipt_number = generate_receipt_number(&conn)?;
    let receipt_group_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    conn.execute("BEGIN IMMEDIATE TRANSACTION", [])
        .map_err(|e| e.to_string())?;

    for allocation in &request.allocations {
        let existing_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM collections WHERE
                 id_partener = ?1 AND serie_factura = ?2 AND numar_factura = ?3 AND cod_document = ?4 AND
                 (status = 'pending' OR status = 'sending')",
                params![
                    &partner_id,
                    &allocation.serie_factura,
                    &allocation.numar_factura,
                    &allocation.cod_document
                ],
                |row| row.get(0),
            )
            .map_err(|e| {
                let _ = conn.execute("ROLLBACK", []);
                e.to_string()
            })?;

        if existing_count > 0 {
            let _ = conn.execute("ROLLBACK", []);
            return Err(format!(
                "Există deja o încasare în curs pentru factura {} {}",
                allocation.serie_factura.clone().unwrap_or_default(),
                allocation.numar_factura.clone().unwrap_or_default()
            ));
        }

        let row_id = Uuid::new_v4().to_string();
        if let Err(e) = conn.execute(
            "INSERT INTO collections (
                id, receipt_group_id, receipt_series, receipt_number,
                id_partener, partner_name, numar_factura, serie_factura,
                cod_document, valoare, data_incasare, status, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                row_id,
                &receipt_group_id,
                &receipt_series,
                &receipt_number,
                &partner_id,
                &request.partner_name,
                &allocation.numar_factura,
                &allocation.serie_factura,
                &allocation.cod_document,
                allocation.valoare,
                &now,
                "pending",
                &now
            ],
        ) {
            let _ = conn.execute("ROLLBACK", []);
            return Err(e.to_string());
        }
    }

    conn.execute("COMMIT", []).map_err(|e| e.to_string())?;

    Ok(receipt_group_id)
}

#[tauri::command]
pub fn record_collection_from_invoice(
    db: State<'_, Database>,
    invoice_id: String,
    paid_amount: f64,
) -> Result<String, String> {
    if paid_amount <= 0.0 {
        return Err("Suma încasată trebuie să fie mai mare decât 0".to_string());
    }

    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let (partner_id, partner_name, invoice_number, total_amount, carnet_series): (String, String, i64, f64, Option<String>) = conn
        .query_row(
            r#"
            SELECT i.partner_id, p.name, i.invoice_number, i.total_amount,
                   (SELECT carnet_series FROM agent_settings WHERE id = 1)
            FROM invoices i
            JOIN partners p ON p.id = i.partner_id
            WHERE i.id = ?1
            "#,
            [&invoice_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .map_err(|e| format!("Factura nu a fost găsită: {}", e))?;

    // Calculate Gross Total (Total with VAT)
    let mut stmt_items = conn
        .prepare(
            "SELECT ii.total_price, p.procent_tva \
             FROM invoice_items ii \
             JOIN products p ON ii.product_id = p.id \
             WHERE ii.invoice_id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let gross_total: f64 = stmt_items
        .query_map([&invoice_id], |row| {
            let price: f64 = row.get(0)?;
            let tva_str: Option<String> = row.get(1)?;
            let tva_percent = tva_str
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
            
            let vat_amount = price * (tva_percent / 100.0);
            Ok(price + vat_amount)
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .sum();

    // Drop the statement to release the borrow on conn
    drop(stmt_items);

    // Allow a small epsilon for floating point comparison
    const EPSILON: f64 = 0.01;

    // Use gross_total for validation instead of total_amount (which is net)
    if paid_amount > (gross_total + EPSILON) {
        return Err(format!(
            "Suma încasată ({:.2}) nu poate depăși totalul facturii cu TVA ({:.2})",
            paid_amount,
            gross_total
        ));
    }

    let invoice_number_str = invoice_number.to_string();
    let receipt_series = get_receipt_series(&conn)?;
    let series = carnet_series.unwrap_or_else(|| "FACTURA".to_string());

    let receipt_number = generate_receipt_number(&conn)?;

    let collected_total: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(valoare), 0) FROM collections WHERE
             id_partener = ?1 AND numar_factura = ?2 AND cod_document = ?3 AND
             status IN ('pending', 'sending', 'synced')",
            params![&partner_id, &invoice_number_str, &invoice_number_str],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    // Calculate remaining based on Gross Total
    let remaining = (gross_total - collected_total).max(0.0);

    if remaining <= EPSILON {
        return Err("Factura este deja încasată integral".to_string());
    }

    if paid_amount > (remaining + EPSILON) {
        return Err(format!(
            "Suma încasată ({:.2}) depășește restul disponibil ({:.2})",
            paid_amount,
            remaining
        ));
    }

    let collection_id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO collections (
            id, receipt_group_id, receipt_series, receipt_number,
            id_partener, partner_name, numar_factura, serie_factura,
            cod_document, valoare, data_incasare, status, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            &collection_id,
            &collection_id,
            receipt_series,
            receipt_number,
            &partner_id,
            &partner_name,
            &invoice_number_str,
            &series,
            &invoice_number_str,
            paid_amount,
            Utc::now().to_rfc3339(),
            "pending",
            Utc::now().to_rfc3339(),
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(collection_id)
}

#[tauri::command]
pub fn get_invoice_remaining_for_collection(
    db: State<'_, Database>,
    invoice_id: String,
) -> Result<f64, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let (partner_id, invoice_number, total_amount): (String, i64, f64) = conn
        .query_row(
            r#"
            SELECT i.partner_id, i.invoice_number, i.total_amount
            FROM invoices i
            WHERE i.id = ?1
            "#,
            [&invoice_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| format!("Factura nu a fost găsită: {}", e))?;

    let invoice_number_str = invoice_number.to_string();

    let collected_total: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(valoare), 0) FROM collections WHERE
             id_partener = ?1 AND numar_factura = ?2 AND cod_document = ?3 AND
             status IN ('pending', 'sending', 'synced')",
            params![&partner_id, &invoice_number_str, &invoice_number_str],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok((total_amount - collected_total).max(0.0))
}

#[tauri::command]
pub fn get_collections(
    db: State<'_, Database>,
    status_filter: Option<String>,
) -> Result<Vec<Collection>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let query = r#"
        SELECT
            COALESCE(receipt_group_id, id) AS group_id,
            id_partener,
            MAX(partner_name) AS partner_name,
            MAX(numar_factura) AS first_numar_factura,
            MAX(serie_factura) AS first_serie_factura,
            MAX(cod_document) AS first_cod_document,
            SUM(valoare) AS total_valoare,
            MAX(data_incasare) AS data_incasare,
            SUM(CASE WHEN status = 'sending' THEN 1 ELSE 0 END) AS cnt_sending,
            SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) AS cnt_failed,
            SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END) AS cnt_pending,
            MAX(synced_at) AS synced_at,
            MAX(error_message) AS error_message,
            MAX(created_at) AS created_at,
            MAX(receipt_series) AS receipt_series,
            MAX(receipt_number) AS receipt_number,
            COUNT(*) AS invoice_count
        FROM collections
        GROUP BY COALESCE(receipt_group_id, id), id_partener
        ORDER BY MAX(created_at) DESC
    "#;

    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            let cnt_sending: i64 = row.get(8)?;
            let cnt_failed: i64 = row.get(9)?;
            let cnt_pending: i64 = row.get(10)?;

            let status = if cnt_sending > 0 {
                CollectionStatus::Sending
            } else if cnt_failed > 0 {
                CollectionStatus::Failed
            } else if cnt_pending > 0 {
                CollectionStatus::Pending
            } else {
                CollectionStatus::Synced
            };

            let invoice_count: i64 = row.get(16)?;
            let first_numar_factura: Option<String> = row.get(3)?;
            let first_serie_factura: Option<String> = row.get(4)?;
            let first_cod_document: Option<String> = row.get(5)?;
            let receipt_series: Option<String> = row.get(14)?;
            let receipt_number: Option<String> = row.get(15)?;

            let numar_factura = if invoice_count > 1 {
                Some(format!("{} facturi", invoice_count))
            } else {
                first_numar_factura
            };

            let serie_factura = if invoice_count > 1 {
                receipt_series
            } else {
                first_serie_factura
            };

            Ok(Collection {
                id: row.get(0)?,
                id_partener: row.get(1)?,
                partner_name: row.get(2)?,
                numar_factura,
                serie_factura,
                cod_document: receipt_number.or(first_cod_document),
                valoare: row.get(6)?,
                data_incasare: row.get(7)?,
                status,
                synced_at: row.get(11)?,
                error_message: row.get(12)?,
                created_at: row.get(13)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for row in rows {
        let collection = row.map_err(|e| e.to_string())?;
        if let Some(filter) = &status_filter {
            if collection.status.to_string() != *filter {
                continue;
            }
        }
        result.push(collection);
    }

    Ok(result)
}

#[tauri::command]
pub async fn sync_collections(
    db: State<'_, Database>,
) -> Result<SyncStatus, String> {
    let pending_collections = get_collections(db.clone(), Some("pending".to_string()))?;

    for collection in pending_collections {
        let _ = send_collection(db.clone(), collection.id).await;
    }

    get_sync_status(db)
}

#[tauri::command]
pub async fn send_collection(
    db: State<'_, Database>,
    collection_id: String,
) -> Result<Collection, String> {
    use chrono::Datelike;
    info!("[CHITANTE][SEND] Start send_collection for id/group={}", collection_id);

    let settings: AgentSettings = get_agent_settings(db.clone())?;

    let rows = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT
                    id,
                    COALESCE(receipt_group_id, id) as receipt_group_id,
                    receipt_series,
                    receipt_number,
                    id_partener,
                    partner_name,
                    numar_factura,
                    serie_factura,
                    cod_document,
                    valoare,
                    data_incasare,
                    status,
                    synced_at,
                    error_message,
                    created_at
                 FROM collections
                 WHERE COALESCE(receipt_group_id, id) = ?1",
            )
            .map_err(|e| e.to_string())?;

        let mapped = stmt
            .query_map([&collection_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, f64>(9)?,
                    row.get::<_, String>(10)?,
                    row.get::<_, String>(11)?,
                    row.get::<_, Option<String>>(12)?,
                    row.get::<_, Option<String>>(13)?,
                    row.get::<_, String>(14)?,
                ))
            })
            .map_err(|e| e.to_string())?;

        let mut result = Vec::new();
        for row in mapped {
            result.push(row.map_err(|e| e.to_string())?);
        }
        result
    };

    if rows.is_empty() {
        return Err("Chitanța nu a fost găsită".to_string());
    }

    let receipt_group_id = rows[0].1.clone();
    let receipt_series = rows[0]
        .2
        .clone()
        .unwrap_or_else(|| settings.carnet_series.clone().unwrap_or_else(|| "CH".to_string()));
    let receipt_number = rows[0]
        .3
        .clone()
        .or_else(|| rows[0].8.clone())
        .unwrap_or_else(|| chrono::Local::now().format("%Y%m%d%H%M%S").to_string());
    let partner_id = rows[0].4.clone();
    let partner_name = rows[0].5.clone();
    let doc_date_source = rows[0].10.clone();

    let total_value: f64 = rows.iter().map(|r| r.9).sum();
    let invoice_count = rows.len();

    let collection_for_print = Collection {
        id: receipt_group_id.clone(),
        id_partener: partner_id.clone(),
        partner_name: partner_name.clone(),
        numar_factura: if invoice_count > 1 {
            Some(format!("{} facturi", invoice_count))
        } else {
            rows[0].6.clone()
        },
        serie_factura: if invoice_count > 1 {
            rows[0].2.clone()
        } else {
            rows[0].7.clone()
        },
        cod_document: Some(receipt_number.clone()),
        valoare: total_value,
        data_incasare: doc_date_source.clone(),
        status: CollectionStatus::Pending,
        synced_at: None,
        error_message: None,
        created_at: rows[0].14.clone(),
    };

    let now = Utc::now();
    let issue_date_for_print = if let Ok(d) = chrono::NaiveDateTime::parse_from_str(&doc_date_source, "%Y-%m-%dT%H:%M:%S%.fZ") {
        d.format("%d.%m.%Y").to_string()
    } else {
        now.format("%d.%m.%Y").to_string()
    };

    let (partner_cui, partner_reg_com, partner_address, partner_localitate, partner_judet) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        get_partner_receipt_info(&conn, &partner_id)
    };

    let (saved_html_path, _) = save_receipt_html_file(
        &collection_for_print,
        &receipt_series,
        &receipt_number,
        &issue_date_for_print,
        settings.agent_name.as_deref(),
        settings.nume_casa.as_deref().unwrap_or("CASA LEI"),
        partner_address.as_deref(),
        partner_localitate.as_deref(),
        partner_judet.as_deref(),
        partner_cui.as_deref(),
        partner_reg_com.as_deref(),
        &receipt_group_id,
    )?;

    info!(
        "[CHITANTE][SEND] Receipt snapshot saved before API call: {}",
        saved_html_path
    );

    let distribuire_valoare: Vec<api_client::DistribuireValoare> = rows
        .iter()
        .map(|r| api_client::DistribuireValoare {
            reprezinta: "Factura".to_string(),
            numar_factura: r.6.clone().unwrap_or_default(),
            serie_factura: r.7.clone().unwrap_or_default(),
            termen_factura: "".to_string(),
            valoare: r.9,
        })
        .collect();

    info!(
        "[CHITANTE][SEND] Loaded group {} partner={:?} allocations={} total={} marca_agent={:?} nume_casa={:?}",
        receipt_group_id,
        partner_name,
        rows.len(),
        total_value,
        settings.marca_agent,
        settings.nume_casa
    );

    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        
        // Check if we can transition to sending state
        // Only allow if not already sending or synced
        // This prevents race conditions when multiple sync triggers happen
        let affected = conn.execute(
            "UPDATE collections SET status = 'sending' WHERE COALESCE(receipt_group_id, id) = ?1 AND status != 'sending' AND status != 'synced'",
            [&receipt_group_id],
        )
        .map_err(|e| e.to_string())?;

        if affected == 0 {
            info!("[CHITANTE][SEND] Collection group {} is already sending or synced. Skipping duplicate send.", receipt_group_id);
            return Ok(collection_for_print);
        }
    }

    let api = api_client::ApiClient::from_default()?;
    let an_lucru = now.year();
    let luna_lucru = now.month() as i32;

    let doc_date = issue_date_for_print.clone();

    let doc = api_client::CasaBancaDocument {
        sursa: "CASA".to_string(),
        nume_casa: settings.nume_casa.clone().unwrap_or("CASA LEI".to_string()),
        numar_cont: "".to_string(),
        data: doc_date.clone(),
        agent: settings.marca_agent.clone().unwrap_or_default(),
        moneda: "RON".to_string(),
        document_cumulativ: "".to_string(),
        tranzactii: vec![api_client::CasaBancaTranzactie {
            tip_tranzactie: "Incasare".to_string(),
            diferenta_pe_avans: "".to_string(),
            tip_doc: "Chitanta".to_string(),
            serie_doc: receipt_series,
            nr_doc: receipt_number,
            obiect_tranzactie: "Client".to_string(),
            data: doc_date,
            curs: 1.0,
            id_partener: partner_id.clone(),
            valoare: total_value,
            obs: "".to_string(),
            anulat: "NU".to_string(),
            distribuire_valoare,
        }],
    };

    let request = api_client::CasaBancaRequest {
        an_lucru,
        luna_lucru,
        cod_subunitate: None,
        documente: vec![doc],
    };

    if let Ok(payload) = serde_json::to_string_pretty(&request) {
        info!("[CHITANTE][SEND] CasaBanca payload for group {}:\n{}", receipt_group_id, payload);
    }

    let now_str = Utc::now().to_rfc3339();

    // DUPLICATE PREVENTION: Check if invoice is already paid in WME before sending receipt
    // This handles the case where a previous attempt succeeded on server but failed to return OK to client
    let _start_check = std::time::Instant::now();
    // ApiClient already created above
    
    // We need to check the balance for the partner to see if the invoice is still unpaid
    // Filters for get_solduri_clienti
    let check_filter = api_client::SolduriFilterRequest {
        id_partener: Some(partner_id.clone()),
        marca_agent: None,
        paginare: None,
    };
    
    let mut already_paid = false;
    
    // Only perform this check if we are retrying (status was pending/failed) or just always to be safe?
    // Always checking is safer but adds latency. Given this is "Send" action initiated by user or sync, latency is acceptable for safety.
    info!("[CHITANTE][SEND] Checking WME balance for partner {} to prevent duplicates...", partner_id);
    
    match api.get_solduri_clienti(check_filter).await {
        Ok(response) => {
            // Check if our invoices exist in the balance list with remaining amount
            // If they are missing or have rest=0, they are paid.
            
            // Collect all invoice numbers from this receipt group
            let invoice_numbers: std::collections::HashSet<String> = rows.iter()
                .filter_map(|r| r.6.clone()) // numar_factura
                .collect();
            
            if !invoice_numbers.is_empty() {
                // Find these invoices in the response
                let mut found_unpaid_amount = 0.0;
                let mut found_invoices_count = 0;
                
                for sold in &response.info_solduri {
                    let sold_numar = sold.numar.as_deref().unwrap_or_default().trim();
                    if invoice_numbers.contains(sold_numar) {
                        let rest = api_client::parse_f64(&sold.rest);
                        if rest > 0.01 {
                            found_unpaid_amount += rest;
                            found_invoices_count += 1;
                        }
                    }
                }
                
                info!("[CHITANTE][SEND] Balance check: Found {} matching unpaid invoices with total rest {:.2}. Receipt total: {:.2}", 
                    found_invoices_count, found_unpaid_amount, total_value);
                
                // If we found NO matching unpaid invoices (count=0) OR the total unpaid amount is significantly less than receipt value
                // Then we assume it's already paid.
                // Note: It's possible partial payment exists. 
                // Strict check: if found_unpaid_amount < (total_value - 0.5) { ... }
                // Giving 0.5 RON buffer for rounding differences
                 if found_unpaid_amount < (total_value - 0.5) {
                    warn!("[CHITANTE][SEND] DETECTED DUPLICATE! WME Rest ({:.2}) < Receipt Value ({:.2}). Marking as synced.", found_unpaid_amount, total_value);
                    already_paid = true;
                }
            } else {
                 info!("[CHITANTE][SEND] No specific invoice numbers to check (advance payment?). Proceeding with send.");
            }
        },
        Err(e) => {
            warn!("[CHITANTE][SEND] Failed to check balance before sending: {}. Proceeding with send to be safe.", e);
            // If check fails (e.g. timeout), we proceed with send. 
            // Worst case: duplicate. 
            // Better than blocking payment if check API is down.
        }
    }
    
    if already_paid {
         // Skip sending to API, just mark as synced
         let conn = db.conn.lock().map_err(|e| e.to_string())?;
         conn.execute(
            "UPDATE collections SET status = 'synced', synced_at = ?1, error_message = NULL WHERE COALESCE(receipt_group_id, id) = ?2",
            params![now_str, receipt_group_id],
        )
        .map_err(|e| e.to_string())?;
        
        // Return updated collection
        drop(conn); // Drop lock
        let grouped = get_collections(db.clone(), None)?;
        let updated = grouped
            .into_iter()
            .find(|c| c.id == receipt_group_id)
            .ok_or_else(|| "Nu s-a putut încărca chitanța actualizată".to_string())?;
            
        return Ok(updated);
    }
    
    // Actually send the request (previously missing variable 'request' is now defined above)
    let api_result = api.send_collections_to_wme(request).await;

    match api_result {
        Ok(response) => {
            info!("[CHITANTE][SEND] CasaBanca response for group {} result={:?} errors={:?}", receipt_group_id, response.result, response.error_list);
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            let err_list = response.result.unwrap_or("".to_string());

            if err_list.to_lowercase() == "ok" || response.error_list.is_empty() {
                conn.execute(
                    "UPDATE collections SET status = 'synced', synced_at = ?1, error_message = NULL WHERE COALESCE(receipt_group_id, id) = ?2",
                    params![now_str, receipt_group_id],
                )
                .map_err(|e| e.to_string())?;
            } else {
                let err_msg = format!(
                    "API Error: {}; {:?}. Chitanță salvată: {}",
                    err_list,
                    response.error_list,
                    saved_html_path
                );
                conn.execute(
                    "UPDATE collections SET status = 'failed', error_message = ?1 WHERE COALESCE(receipt_group_id, id) = ?2",
                    params![err_msg, receipt_group_id],
                )
                .map_err(|e| e.to_string())?;
            }
        }
        Err(e) => {
            info!("[CHITANTE][SEND] CasaBanca call failed for group {} error={}", receipt_group_id, e);
            let conn = db.conn.lock().map_err(|err| err.to_string())?;
            let err_msg = format!("{}. Chitanță salvată: {}", e, saved_html_path);
            conn.execute(
                "UPDATE collections SET status = 'pending', error_message = ?1 WHERE COALESCE(receipt_group_id, id) = ?2",
                params![err_msg, receipt_group_id],
            )
            .map_err(|err| err.to_string())?;
        }
    }

    let grouped = get_collections(db.clone(), None)?;
    let updated = grouped
        .into_iter()
        .find(|c| c.id == receipt_group_id)
        .ok_or_else(|| "Nu s-a putut încărca chitanța actualizată".to_string())?;

    info!(
        "[CHITANTE][SEND] Finished send_collection group={} final_status={} error={:?}",
        updated.id,
        updated.status.to_string(),
        updated.error_message
    );

    Ok(updated)
}

#[tauri::command]
pub fn delete_collection(db: State<'_, Database>, collection_id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM collections WHERE COALESCE(receipt_group_id, id) = ?1 OR id = ?1",
        [&collection_id],
    )
        .map_err(|e| e.to_string())?;

    info!("Deleted collection {}", collection_id);
    Ok(())
}

#[tauri::command]
pub fn get_sales_report(
    db: State<'_, Database>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<SalesReportItem>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    let mut query = "SELECT 
        p.name as partner_name,
        COUNT(*) as count, 
        SUM(i.total_amount) as total,
        COALESCE(SUM(inv_qty.total_quantity), 0) as total_quantity
        FROM invoices i
        JOIN partners p ON p.id = i.partner_id
        LEFT JOIN (
            SELECT invoice_id, SUM(quantity) as total_quantity
            FROM invoice_items
            GROUP BY invoice_id
        ) inv_qty ON inv_qty.invoice_id = i.id
        WHERE 1 = 1".to_string();
        
    let mut params: Vec<String> = Vec::new();
    
    if let Some(start) = start_date {
        query.push_str(" AND i.created_at >= ?");
        params.push(format!("{}T00:00:00", start));
    }
    
    if let Some(end) = end_date {
        query.push_str(" AND i.created_at <= ?");
        params.push(format!("{}T23:59:59", end));
    }
    
    query.push_str(" GROUP BY p.name ORDER BY total DESC");
    
    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    
    // Note: This is simplified. We might need better VAT calculation.
    let items = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
        let total: f64 = row.get(2)?;
        let total_quantity: f64 = row.get(3)?;
        Ok(SalesReportItem {
            partner_name: row.get(0)?,
            invoice_count: row.get(1)?,
            total_amount: total,
            total_vat: total * 0.19, // Approximation
            total_quantity,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut result = Vec::new();
    for i in items {
        result.push(i.map_err(|e| e.to_string())?);
    }
    
    Ok(result)
}

#[tauri::command]
pub fn get_sales_print_report(
    db: State<'_, Database>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<SalesPrintItem>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut query = "WITH invoice_data AS (
        SELECT
            COALESCE(NULLIF(TRIM(p.cod_extern), ''), p.name, 'Partener') AS partner_name,
            i.created_at,
            COALESCE(inv_qty.total_quantity, 0) AS total_quantity,
            COALESCE(inv_totals.total_without_vat, i.total_amount) AS total_without_vat,
            COALESCE(inv_totals.total_with_vat, i.total_amount * 1.19) AS total_with_vat,
            COALESCE(col.total_collected, 0) AS collected_amount
        FROM invoices i
        JOIN partners p ON p.id = i.partner_id
        LEFT JOIN (
            SELECT invoice_id, SUM(quantity) AS total_quantity
            FROM invoice_items
            GROUP BY invoice_id
        ) inv_qty ON inv_qty.invoice_id = i.id
        LEFT JOIN (
            SELECT
                ii.invoice_id,
                SUM(ii.total_price) AS total_without_vat,
                SUM(
                    ii.total_price * (
                        1 + (COALESCE(CAST(pr.procent_tva AS REAL), 19) / 100.0)
                    )
                ) AS total_with_vat
            FROM invoice_items ii
            LEFT JOIN products pr ON pr.id = ii.product_id
            GROUP BY ii.invoice_id
        ) inv_totals ON inv_totals.invoice_id = i.id
        LEFT JOIN (
            SELECT
                id_partener,
                COALESCE(numar_factura, '') AS numar_factura,
                COALESCE(cod_document, '') AS cod_document,
                SUM(valoare) AS total_collected
            FROM collections
            WHERE status IN ('pending', 'sending', 'synced')
            GROUP BY id_partener, COALESCE(numar_factura, ''), COALESCE(cod_document, '')
        ) col ON (
            col.id_partener = i.partner_id
            AND (
                col.numar_factura = CAST(i.invoice_number AS TEXT)
                OR col.cod_document = CAST(i.invoice_number AS TEXT)
            )
        )
        WHERE 1 = 1"
        .to_string();

    let mut params: Vec<String> = Vec::new();

    if let Some(start) = start_date {
        query.push_str(" AND i.created_at >= ?");
        params.push(format!("{}T00:00:00", start));
    }

    if let Some(end) = end_date {
        query.push_str(" AND i.created_at <= ?");
        params.push(format!("{}T23:59:59", end));
    }

    query.push_str(
        "
    ),
    partner_data AS (
        SELECT
            partner_name,
            CASE
                WHEN collected_amount >= total_without_vat THEN 'paid'
                ELSE 'unpaid'
            END AS payment_section,
            COUNT(*) AS invoice_count,
            SUM(total_quantity) AS total_quantity,
            SUM(total_without_vat) AS total_without_vat,
            SUM(total_with_vat - total_without_vat) AS total_vat,
            SUM(total_with_vat) AS total_with_vat
        FROM invoice_data
        GROUP BY partner_name, payment_section
    )
    SELECT
        partner_name,
        invoice_count,
        total_quantity,
        ROUND(total_quantity / 30.0, 2) AS total_cofrage,
        total_without_vat,
        total_vat,
        total_with_vat,
        payment_section
    FROM partner_data
    WHERE 1 = 1"
    );

    query.push_str(" ORDER BY CASE payment_section WHEN 'unpaid' THEN 1 ELSE 2 END, total_with_vat DESC");

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    let items = stmt
        .query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok(SalesPrintItem {
                partner_name: row.get(0)?,
                invoice_count: row.get(1)?,
                total_quantity: row.get(2)?,
                total_cofrage: row.get(3)?,
                total_without_vat: row.get(4)?,
                total_vat: row.get(5)?,
                total_with_vat: row.get(6)?,
                payment_section: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for item in items {
        result.push(item.map_err(|e| e.to_string())?);
    }

    Ok(result)
}

#[tauri::command]
pub fn get_sales_products_report(
    db: State<'_, Database>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<SalesProductReportItem>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut query = "SELECT
        ii.product_id,
        pr.name AS product_name,
        pr.class AS product_class,
        p.name AS partner_name,
        COALESCE(i.invoice_number, 0) AS invoice_number,
        COALESCE((SELECT carnet_series FROM agent_settings WHERE id = 1), 'FACTURA') AS invoice_series,
        ii.quantity AS total_quantity,
        ROUND(ii.quantity / 30.0, 2) AS total_cofrage,
        ii.total_price AS total_without_vat,
        ii.total_price * (1 + (COALESCE(CAST(pr.procent_tva AS REAL), 19) / 100.0)) AS total_with_vat,
        i.created_at
    FROM invoice_items ii
    JOIN invoices i ON i.id = ii.invoice_id
    JOIN partners p ON p.id = i.partner_id
    LEFT JOIN products pr ON pr.id = ii.product_id
    WHERE 1 = 1"
        .to_string();

    let mut params: Vec<String> = Vec::new();

    if let Some(start) = start_date {
        query.push_str(" AND i.created_at >= ?");
        params.push(format!("{}T00:00:00", start));
    }

    if let Some(end) = end_date {
        query.push_str(" AND i.created_at <= ?");
        params.push(format!("{}T23:59:59", end));
    }

    query.push_str(" ORDER BY COALESCE(pr.class, ''), pr.name, i.created_at DESC");

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    let items = stmt
        .query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok(SalesProductReportItem {
                product_id: row.get(0)?,
                product_name: row.get(1)?,
                product_class: row.get(2)?,
                partner_name: row.get(3)?,
                invoice_number: row.get(4)?,
                invoice_series: row.get(5)?,
                total_quantity: row.get(6)?,
                total_cofrage: row.get(7)?,
                total_without_vat: row.get(8)?,
                total_with_vat: row.get(9)?,
                created_at: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for item in items {
        result.push(item.map_err(|e| e.to_string())?);
    }

    Ok(result)
}

#[tauri::command]
pub fn get_collections_report(
    db: State<'_, Database>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<CollectionsReportItem>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    let mut query = "SELECT 
        partner_name, 
        COUNT(*) as count, 
        SUM(valoare) as total,
        status
        FROM collections".to_string();
        
    let mut params: Vec<String> = Vec::new();

    let mut where_started = false;

    if let Some(start) = start_date {
        query.push_str(" WHERE data_incasare >= ?");
        params.push(format!("{}T00:00:00", start));
        where_started = true;
    }

    if let Some(end) = end_date {
        if where_started {
            query.push_str(" AND data_incasare <= ?");
        } else {
            query.push_str(" WHERE data_incasare <= ?");
        }
        params.push(format!("{}T23:59:59", end));
    }
    
    query.push_str(" GROUP BY partner_name, status ORDER BY total DESC");
    
    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    
    let items = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
        Ok(CollectionsReportItem {
            partner_name: row.get(0)?,
            collection_count: row.get(1)?,
            total_amount: row.get(2)?,
            status: row.get(3)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut result = Vec::new();
    for i in items {
        result.push(i.map_err(|e| e.to_string())?);
    }
    
    Ok(result)
}

#[tauri::command]
pub fn get_daily_collections_report(
    db: State<'_, Database>,
    date: Option<String>,
) -> Result<DailyCollectionsReport, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let target_date = date.unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());
    let previous_date = chrono::NaiveDate::parse_from_str(&target_date, "%Y-%m-%d")
        .map_err(|e| format!("Dată invalidă: {}", e))?
        .pred_opt()
        .ok_or_else(|| "Nu s-a putut calcula ziua precedentă".to_string())?
        .format("%Y-%m-%d")
        .to_string();

    let mut stmt = conn
        .prepare(
            r#"
            SELECT
                COALESCE(NULLIF(TRIM(p.cod_extern), ''), c.partner_name, p.name, 'Partener') AS partner_name,
                SUM(
                    CASE
                        WHEN i.id IS NOT NULL AND substr(i.created_at, 1, 10) = ?1 THEN c.valoare
                        ELSE 0
                    END
                ) AS amount_from_today_sales,
                SUM(
                    CASE
                        WHEN i.id IS NULL OR substr(i.created_at, 1, 10) <> ?1 THEN c.valoare
                        ELSE 0
                    END
                ) AS amount_from_previous_debt,
                SUM(c.valoare) AS total_amount
            FROM collections c
            LEFT JOIN partners p ON p.id = c.id_partener
            LEFT JOIN invoices i ON i.partner_id = c.id_partener
                AND (
                    CAST(i.invoice_number AS TEXT) = COALESCE(c.numar_factura, '')
                    OR CAST(i.invoice_number AS TEXT) = COALESCE(c.cod_document, '')
                )
            WHERE substr(c.data_incasare, 1, 10) = ?1
              AND c.status IN ('pending', 'sending', 'synced')
                        GROUP BY COALESCE(NULLIF(TRIM(p.cod_extern), ''), c.partner_name, p.name, 'Partener')
            ORDER BY total_amount DESC
            "#,
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([&target_date], |row| {
            Ok(DailyCollectionsPartnerItem {
                partner_name: row.get(0)?,
                amount_from_today_sales: row.get(1)?,
                amount_from_previous_debt: row.get(2)?,
                total_amount: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|e| e.to_string())?);
    }

    let current_day_collections_total: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(valoare), 0) FROM collections WHERE substr(data_incasare, 1, 10) = ?1 AND status IN ('pending', 'sending', 'synced')",
            [&target_date],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let previous_day_collections_total: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(valoare), 0) FROM collections WHERE substr(data_incasare, 1, 10) = ?1 AND status IN ('pending', 'sending', 'synced')",
            [&previous_date],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let current_day_receipts_count: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT COALESCE(receipt_group_id, id)) FROM collections WHERE substr(data_incasare, 1, 10) = ?1 AND status IN ('pending', 'sending', 'synced')",
            [&target_date],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let previous_day_receipts_count: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT COALESCE(receipt_group_id, id)) FROM collections WHERE substr(data_incasare, 1, 10) = ?1 AND status IN ('pending', 'sending', 'synced')",
            [&previous_date],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let (receipts_today_invoices_count, receipts_previous_debt_count): (i64, i64) = conn
        .query_row(
            r#"
            SELECT
                COUNT(DISTINCT CASE
                    WHEN i.id IS NOT NULL AND substr(i.created_at, 1, 10) = ?1
                    THEN COALESCE(c.receipt_group_id, c.id)
                END) AS receipts_today_invoices_count,
                COUNT(DISTINCT CASE
                    WHEN i.id IS NULL OR substr(i.created_at, 1, 10) <> ?1
                    THEN COALESCE(c.receipt_group_id, c.id)
                END) AS receipts_previous_debt_count
            FROM collections c
            LEFT JOIN invoices i ON i.partner_id = c.id_partener
                AND (
                    CAST(i.invoice_number AS TEXT) = COALESCE(c.numar_factura, '')
                    OR CAST(i.invoice_number AS TEXT) = COALESCE(c.cod_document, '')
                )
            WHERE substr(c.data_incasare, 1, 10) = ?1
              AND c.status IN ('pending', 'sending', 'synced')
            "#,
            [&target_date],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())?;

    Ok(DailyCollectionsReport {
        items,
        receipts_today_invoices_count,
        receipts_previous_debt_count,
        current_day_receipts_count,
        previous_day_receipts_count,
        current_day_collections_total,
        previous_day_collections_total,
        total_day_collections: current_day_collections_total,
    })
}

#[tauri::command]
pub fn print_daily_report(
    db: State<'_, Database>,
    date: Option<String>,
    printer_name: Option<String>,
) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    // Determine date to filter (YYYY-MM-DD)
    let date_str = date.unwrap_or_else(|| {
        Local::now().format("%Y-%m-%d").to_string()
    });

    info!("Generating daily sales report for date: {}", date_str);

    // Fetch invoices for this date
    // We assume created_at is ISO string, so we match YYYY-MM-DD%
    let param_date = format!("{}%", date_str);
    
    let mut stmt = conn.prepare(
        r#"SELECT
                i.id, i.partner_id, p.name, p.cif, p.reg_com, i.location_id, l.name, l.address,
                i.status, i.total_amount, i.notes, i.created_at, i.sent_at, i.error_message,
                (SELECT COUNT(*) FROM invoice_items WHERE invoice_id = i.id),
                p.scadenta_la_vanzare
            FROM invoices i
            JOIN partners p ON i.partner_id = p.id
            JOIN locations l ON i.location_id = l.id
            WHERE i.created_at LIKE ?1
            ORDER BY i.created_at ASC"#
    ).map_err(|e| e.to_string())?;

    let invoices_iter = stmt.query_map([&param_date], |row| {
        Ok(Invoice {
            id: row.get(0)?,
            partner_id: row.get(1)?,
            partner_name: row.get(2)?,
            partner_cif: row.get(3)?,
            partner_reg_com: row.get(4)?,
            location_id: row.get(5)?,
            location_name: row.get(6)?,
            location_address: row.get(7)?,
            status: InvoiceStatus::from(row.get::<_, String>(8)?),
            total_amount: row.get(9)?,
            notes: row.get(10)?,
            created_at: row.get(11)?,
            sent_at: row.get(12)?,
            error_message: row.get(13)?,
            item_count: row.get(14)?,
            partner_payment_term: row.get(15)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut invoices = Vec::new();
    let mut total_sales: f64 = 0.0;

    for invoice in invoices_iter {
        let inv = invoice.map_err(|e| e.to_string())?;
        total_sales += inv.total_amount;
        invoices.push(inv);
    }

    // Generate HTML
    let logo_base64 = read_logo_to_base64();
    let html = print_daily_report::generate_daily_report_html(
        &invoices,
        &date_str,
        total_sales,
        logo_base64.as_deref(),
    );

    // Save to reports folder
    let app_data_dir = dirs::config_dir()
        .ok_or("Could not find app data directory")?
        .join("facturi.softconsulting.com")
        .join("reports");
    
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create reports directory: {}", e))?;

    let file_name = format!("raport_{}", date_str);
    let html_file_path = app_data_dir.join(format!("{}.html", file_name));
    let pdf_file_path = app_data_dir.join(format!("{}.pdf", file_name));
    
    std::fs::write(&html_file_path, &html)
        .map_err(|e| format!("Failed to write HTML file: {}", e))?;

    let html_path_str = html_file_path.to_string_lossy().to_string();
    let pdf_path_str = pdf_file_path.to_string_lossy().to_string();
    
    info!("Generated report HTML at: {}", html_path_str);
    
    // Convert HTML to PDF using Edge (headless)
    #[cfg(target_os = "windows")]
    {
        let mut pdf_generated = false;
        let mut print_file = html_path_str.clone();
        
        // Try Edge first
        let edge_paths = vec![
            "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
            "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe",
        ];
        
        for edge_path in edge_paths {
            if std::path::Path::new(edge_path).exists() {
                let file_url = format!("file:///{}", html_path_str.replace('\\', "/"));
                
                let output = std::process::Command::new(edge_path)
                    .args(&[
                        "--headless",
                        "--disable-gpu",
                        "--no-sandbox",
                        "--disable-dev-shm-usage",
                        &format!("--print-to-pdf={}", pdf_path_str),
                        &file_url,
                    ])
                    .output();
                
                match output {
                    Ok(result) => {
                        info!("Edge command executed. Status: {}", result.status);
                        // Give Edge time to write
                        let mut waited = 0;
                        while waited < 5000 {
                            if wait_for_file_ready(&pdf_path_str, 1000, 300) {
                                pdf_generated = true;
                                print_file = pdf_path_str.clone();
                                info!("PDF generated successfully at: {}", pdf_path_str);
                                break;
                            }
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            waited += 100;
                        }
                        if pdf_generated {
                            break;
                        }
                    }
                    Err(e) => {
                        info!("Failed to use Edge: {}", e);
                    }
                }
            }
        }
        
        if !pdf_generated {
            info!("PDF generation failed, will print HTML directly");
        }
        
        // Print using SumatraPDF
        let printer = printer_name.unwrap_or_else(|| String::from(""));
        
        // Check if a default printer is available when no specific printer is given
        if printer.is_empty() {
            #[cfg(target_os = "windows")]
            {
                // Try to detect default printer using PowerShell
                if let Ok(output) = std::process::Command::new("powershell")
                    .args(&["-NoProfile", "-Command", "Get-CimInstance -Class Win32_Printer | Where-Object { $_.Default -eq $true } | Select-Object -ExpandProperty Name"])
                    .output()
                {
                    let default_printer = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !default_printer.is_empty() {
                        info!("Default printer detected: {}", default_printer);
                    } else {
                        warn!("⚠ No default printer configured in Windows. Printing may fail.");
                    }
                }
            }
        }
        
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
        let bundled_path = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.join("resources").join("SumatraPDF.exe")));
        
        let mut sumatra_paths = vec![
            format!(r"{}\AppData\Local\SumatraPDF\SumatraPDF.exe", user_profile),
            r"C:\Program Files\SumatraPDF\SumatraPDF.exe".to_string(),
            r"C:\Program Files (x86)\SumatraPDF\SumatraPDF.exe".to_string(),
        ];
        
        if let Some(p) = bundled_path {
            sumatra_paths.insert(0, p.to_string_lossy().to_string());
        }
        
        let mut printed = false;
        
        for sumatra_path in sumatra_paths {
            if std::path::Path::new(&sumatra_path).exists() {
                info!("Found SumatraPDF at: {}", sumatra_path);
                
                let mut args = vec![
                    "-print-to-default".to_string(),
                    "-silent".to_string(),
                ];
                
                if !printer.is_empty() {
                    args = vec![
                        "-print-to".to_string(),
                        printer.clone(),
                        "-silent".to_string(),
                    ];
                }
                
                args.push(print_file.clone());
                
                // Log the full command for debugging
                info!("Executing print command with args: {:?}", args);
                
                let output = std::process::Command::new(&sumatra_path)
                    .args(&args)
                    .output();
                    
                match output {
                    Ok(result) => {
                        info!("Print command executed. Status: {}", result.status);
                        let stdout = String::from_utf8_lossy(&result.stdout);
                        let stderr = String::from_utf8_lossy(&result.stderr);
                        
                        if !stdout.is_empty() {
                            info!("Print stdout: {}", stdout);
                        }
                        if !stderr.is_empty() {
                            info!("Print stderr: {}", stderr);
                        }
                        
                        if result.status.success() {
                            printed = true;
                            info!("✓ Document sent to printer successfully");
                            break;
                        } else {
                            warn!("✗ Print failed with exit code: {:?}", result.status.code());
                            
                            // Check for specific printer initialization errors
                            if stdout.contains("CreateDCW") && stdout.contains("failed") {
                                warn!("Printer driver error detected. The printer may be offline, disconnected, or have driver issues.");
                                if let Some(printer_name_match) = stdout.lines()
                                    .find(|line| line.contains("printer:"))
                                    .and_then(|line| line.split("printer: '").nth(1))
                                    .and_then(|s| s.split('\'').next())
                                {
                                    warn!("Printer: {} - Please check if it's powered on and connected.", printer_name_match);
                                }
                            } else if printer.is_empty() {
                                warn!("Hint: No printer specified. Ensure a default printer is set in Windows.");
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to execute print command: {}", e);
                    }
                }
            }
        }
        
        if printed {
            info!("✓ Report printed successfully");
            Ok(format!("Report printed successfully. File saved at: {}", print_file))
        } else {
            let msg = format!(
                "Could not print report. The printer may be offline or disconnected. PDF saved at: {}", 
                print_file
            );
            warn!("{}", msg);
            Ok(msg)
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Ok("Printing is only supported on Windows".to_string())
    }
}
