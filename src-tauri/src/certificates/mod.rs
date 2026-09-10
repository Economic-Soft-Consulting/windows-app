//! Quality certificates (certificat de calitate).
//!
//! Printed alongside every invoice. Building one makes live WME calls to find the matching
//! comanda, with three layers of cache fallback so a certificate can still be produced
//! offline. This was 600 lines buried in the helper prelude of commands.rs.

use chrono::{Datelike, Local, Utc};
use log::warn;
use tauri::State;

use crate::api_client;
use crate::commands::get_wme_api_client;
use rusqlite::params;
use crate::database::Database;
use crate::infra::paths;
use crate::infra::pdf::try_generate_pdf_from_html;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QualityCertificateProductLine {
    pub denumire: String,
    pub lot: String,
    pub data_productie: String,
    pub data_expirare: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CertificateCachePayload {
    pub subtitle: String,
    pub bon_analiza: String,
    pub product_lines: Vec<QualityCertificateProductLine>,
}

pub struct QualityCertificateContext {
    pub cert_date: String,
    pub subtitle: String,
    pub packed_date: String,
    pub beneficiary: String,
    pub invoice_display: String,
    pub invoice_date: String,
    pub car_number: String,
    pub bon_analiza: String,
    pub product_lines: Vec<QualityCertificateProductLine>,
}

pub fn parse_comanda_numar(value: Option<&String>) -> i64 {
    let raw = value.map(|v| v.trim()).unwrap_or_default();
    if raw.is_empty() {
        return i64::MIN;
    }

    if let Ok(parsed) = raw.parse::<i64>() {
        return parsed;
    }

    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.parse::<i64>().unwrap_or(i64::MIN)
}

pub fn parse_comanda_data_score(value: Option<&String>) -> i64 {
    let raw = value.map(|v| v.trim()).unwrap_or_default();
    if raw.is_empty() {
        return i64::MIN;
    }

    let date_part = raw.split_whitespace().next().unwrap_or(raw);

    // Attempt multiple formats as API might send DD.MM.YYYY, YYYY-MM-DD, or DD/MM/YYYY
    let parsed_date = chrono::NaiveDate::parse_from_str(date_part, "%d.%m.%Y")
        .or_else(|_| chrono::NaiveDate::parse_from_str(date_part, "%Y-%m-%d"))
        .or_else(|_| chrono::NaiveDate::parse_from_str(date_part, "%d/%m/%Y"));

    if let Ok(parsed) = parsed_date {
        return i64::from(parsed.year()) * 10_000 + i64::from(parsed.month()) * 100 + i64::from(parsed.day());
    }

    i64::MIN
}

pub fn normalize_or_placeholder(value: Option<String>, placeholder: &str) -> String {
    let normalized = value
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    normalized.unwrap_or_else(|| placeholder.to_string())
}

pub fn apply_cached_certificate_payload(ctx: &mut QualityCertificateContext, payload: CertificateCachePayload) {
    ctx.subtitle = payload.subtitle;
    ctx.bon_analiza = payload.bon_analiza;
    if !payload.product_lines.is_empty() {
        ctx.product_lines = payload.product_lines;
    }
}

pub fn load_cached_certificate_payload(conn: &rusqlite::Connection, invoice_id: &str) -> Option<CertificateCachePayload> {
    let payload_json: Option<String> = conn
        .query_row(
            "SELECT payload_json FROM invoice_certificate_cache WHERE invoice_id = ?1",
            [invoice_id],
            |row| row.get(0),
        )
        .ok();

    payload_json.and_then(|json| serde_json::from_str::<CertificateCachePayload>(&json).ok())
}

pub fn load_latest_cached_certificate_payload(conn: &rusqlite::Connection) -> Option<CertificateCachePayload> {
    let payload_json: Option<String> = conn
        .query_row(
            "SELECT payload_json FROM invoice_certificate_cache ORDER BY updated_at DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();

    payload_json.and_then(|json| serde_json::from_str::<CertificateCachePayload>(&json).ok())
}

pub fn save_cached_certificate_payload(
    conn: &rusqlite::Connection,
    invoice_id: &str,
    ctx: &QualityCertificateContext,
) -> Result<(), String> {
    let payload = CertificateCachePayload {
        subtitle: ctx.subtitle.clone(),
        bon_analiza: ctx.bon_analiza.clone(),
        product_lines: ctx.product_lines.clone(),
    };

    let payload_json = serde_json::to_string(&payload)
        .map_err(|e| format!("Failed to serialize certificate cache payload: {}", e))?;

    conn.execute(
        "INSERT INTO invoice_certificate_cache (invoice_id, payload_json, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(invoice_id) DO UPDATE SET payload_json = excluded.payload_json, updated_at = excluded.updated_at",
        params![invoice_id, payload_json, Utc::now().to_rfc3339()],
    )
    .map_err(|e| format!("Failed to save certificate cache payload: {}", e))?;

    Ok(())
}

pub async fn build_quality_certificate_context(
    db: &State<'_, Database>,
    invoice_id: &str,
    car_number: &str,
) -> Result<QualityCertificateContext, String> {
    let (invoice_number, invoice_series, created_at, partner_name, cert_serie, cert_id_client, api): (i64, Option<String>, String, String, String, String, api_client::ApiClient) = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;

        let row = conn.query_row(
            "SELECT i.invoice_number, i.invoice_series, i.created_at, p.name FROM invoices i JOIN partners p ON p.id = i.partner_id WHERE i.id = ?1",
            [invoice_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        ).map_err(|e| format!("Invoice not found for certificate: {}", e))?;

        let cert_filters: (Option<String>, Option<String>) = conn
            .query_row(
                "SELECT cert_comanda_serie, cert_comanda_id_client FROM agent_settings WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap_or((None, None));

        let cert_serie = cert_filters
            .0
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("CCAL")
            .to_string();

        let cert_id_client = cert_filters
            .1
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("1602")
            .to_string();

        let api = get_wme_api_client(&conn)?;
        (row.0, row.1, row.2, row.3, cert_serie, cert_id_client, api)
    };

    let created_at_dt = chrono::DateTime::parse_from_rfc3339(&created_at)
        .map_err(|e| format!("Failed to parse invoice date for certificate: {}", e))?
        .with_timezone(&Local);

    let invoice_day = created_at_dt.format("%d.%m.%Y").to_string();
    let data_referinta = format!("{} 00:00", invoice_day);
    let data_end = format!("{} 23:59", invoice_day);

    let mut ctx = QualityCertificateContext {
        cert_date: Local::now().format("%d.%m.%Y").to_string(),
        subtitle: "Nr.____ din data de __.__.____".to_string(),
        packed_date: invoice_day.clone(),
        beneficiary: partner_name,
        invoice_display: invoice_series
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .map(|s| format!("{} {}", s, invoice_number))
            .unwrap_or_else(|| invoice_number.to_string()),
        invoice_date: invoice_day.clone(),
        car_number: car_number.trim().to_string(),
        bon_analiza: "BA____/__.__.____".to_string(),
        product_lines: vec![QualityCertificateProductLine {
            denumire: "Produs".to_string(),
            lot: "____".to_string(),
            data_productie: "__.__.____".to_string(),
            data_expirare: "__.__.____".to_string(),
        }],
    };

    let response = api
        .get_info_comenzi_ext(api_client::ComenziExtFilterRequest {
            data_referinta: Some(data_referinta),
            data_end: Some(data_end),
            cod_comanda: None,
            id_partener: Some(cert_id_client.clone()),
            info_extensii: Some("D".to_string()),
        })
        .await;

    let response = match response {
        Ok(value) => value,
        Err(err) => {
            warn!("[CERT] GetInfoComenziExt failed: {}", err);

            if let Ok(conn) = db.conn.lock().map_err(|e| e.to_string()) {
                if let Some(payload) = load_cached_certificate_payload(&conn, invoice_id) {
                    apply_cached_certificate_payload(&mut ctx, payload);
                    warn!("[CERT] Using cached certificate data for invoice {} (offline/API error fallback).", invoice_id);
                    return Ok(ctx);
                }

                if let Some(payload) = load_cached_certificate_payload(&conn, "DAILY_TODAY").or_else(|| load_latest_cached_certificate_payload(&conn)) {
                    apply_cached_certificate_payload(&mut ctx, payload);
                    warn!("[CERT] Using daily/latest cached certificate data for invoice {} (offline/API error fallback).", invoice_id);
                    return Ok(ctx);
                }
            }

            warn!("[CERT] No cached certificate data found for invoice {}. Using placeholders.", invoice_id);
            return Ok(ctx);
        }
    };

    warn!(
        "[CERT] Primary query returned {} comenzi. Looking for Serie='{}' IDClient='{}'.",
        response.info_comenzi.len(),
        cert_serie,
        cert_id_client
    );
    for c in &response.info_comenzi {
        warn!(
            "[CERT]   Comanda: Numar={:?} Serie={:?} IDClient={:?} Data={:?} Items={}",
            c.numar, c.serie, c.id_client, c.data, c.items.len()
        );
    }

    let selected_command = response
        .info_comenzi
        .into_iter()
        .filter(|c| {
            let serie_ok = c.serie
                .as_deref()
                .map(|serie| serie.trim().eq_ignore_ascii_case(cert_serie.as_str()))
                .unwrap_or(false);
            let id_ok = c.id_client
                .as_deref()
                .map(|id| id.trim() == cert_id_client.as_str())
                .unwrap_or(false);

            let max_allowed_date_score = parse_comanda_data_score(Some(&invoice_day));
            let current_date_score = parse_comanda_data_score(c.data.as_ref());
            let date_ok = current_date_score <= max_allowed_date_score && current_date_score != i64::MIN;

            if !serie_ok || !id_ok || !date_ok {
                warn!(
                    "[CERT]   SKIP comanda Numar={:?}: serie_ok={} id_ok={} date_ok={}",
                    c.numar, serie_ok, id_ok, date_ok
                );
            }
            serie_ok && id_ok && date_ok
        })
        .max_by_key(|c| {
            (
                parse_comanda_data_score(c.data.as_ref()),
                parse_comanda_numar(c.numar.as_ref()),
            )
        });

    let selected_command = match selected_command {
        Some(command) => Some(command),
        None => {
            let fallback_start = (created_at_dt - chrono::Duration::days(30))
                .format("%d.%m.%Y")
                .to_string();

            let fallback_response = api
                .get_info_comenzi_ext(api_client::ComenziExtFilterRequest {
                    data_referinta: Some(format!("{} 00:00", fallback_start)),
                    data_end: Some(format!("{} 23:59", invoice_day)),
                    cod_comanda: None,
                    id_partener: Some(cert_id_client.clone()),
                    info_extensii: Some("D".to_string()),
                })
                .await;

            match fallback_response {
                Ok(value) => {
                    let fallback = value
                        .info_comenzi
                        .into_iter()
                        .filter(|c| {
                            let serie_ok = c.serie
                                .as_deref()
                                .map(|serie| serie.trim().eq_ignore_ascii_case(cert_serie.as_str()))
                                .unwrap_or(false);
                            let id_ok = c.id_client
                                .as_deref()
                                .map(|id| id.trim() == cert_id_client.as_str())
                                .unwrap_or(false);

                            let max_allowed_date_score = parse_comanda_data_score(Some(&invoice_day));
                            let current_date_score = parse_comanda_data_score(c.data.as_ref());
                            let date_ok = current_date_score <= max_allowed_date_score && current_date_score != i64::MIN;

                            serie_ok && id_ok && date_ok
                        })
                        .max_by_key(|c| {
                            (
                                parse_comanda_data_score(c.data.as_ref()),
                                parse_comanda_numar(c.numar.as_ref()),
                            )
                        });

                    if fallback.is_some() {
                        warn!(
                            "[CERT] No command found on invoice day {}. Using latest previous command from last 30 days.",
                            invoice_day
                        );
                    }

                    fallback
                }
                Err(err) => {
                    warn!(
                        "[CERT] Fallback GetInfoComenziExt failed for invoice day {}: {}",
                        invoice_day,
                        err
                    );
                    None
                }
            }
        }
    };

    let mut should_cache = false;

    if let Some(command) = selected_command {
        let cmd_numar = normalize_or_placeholder(command.numar, "____");
        let cmd_data = normalize_or_placeholder(command.data, "__.__.____");
        ctx.subtitle = format!("Nr.{} din data de {}", cmd_numar, cmd_data);
        warn!(
            "[CERT] Selected comanda Nr.{} din {} with {} items.",
            cmd_numar, cmd_data, command.items.len()
        );

        let mut lines_map: Vec<QualityCertificateProductLine> = Vec::new();
        let mut bon_list: Vec<String> = Vec::new();

        for item in command.items {
            if let Some(b) = item.bon_analiza.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
                let s = b.to_string();
                if !bon_list.contains(&s) {
                    bon_list.push(s);
                }
            }

            let den = normalize_or_placeholder(item.denumire, "Produs");
            let lot = normalize_or_placeholder(item.lot, "____");
            let d_prod = normalize_or_placeholder(item.data_productie, "__.__.____");
            let d_exp = normalize_or_placeholder(item.data_expirare, "__.__.____");

            let new_line = QualityCertificateProductLine {
                denumire: den,
                lot,
                data_productie: d_prod,
                data_expirare: d_exp,
            };
            if !lines_map.iter().any(|l| {
                l.denumire == new_line.denumire
                    && l.lot == new_line.lot
                    && l.data_productie == new_line.data_productie
                    && l.data_expirare == new_line.data_expirare
            }) {
                lines_map.push(new_line);
            }
        }

        let mut lines = lines_map;
        lines.sort_by(|a, b| a.denumire.cmp(&b.denumire));

        if !lines.is_empty() {
            ctx.product_lines = lines;
            should_cache = true;
        } else {
            warn!(
                "[CERT] Command found but no items matched invoice products for invoice {}.",
                invoice_id
            );
        }

        if !bon_list.is_empty() {
            ctx.bon_analiza = bon_list.join("; ");
        }
    } else {
        warn!(
            "[CERT] No command found for invoice day {} with filters Serie={} and IDClient={}. Using placeholders.",
            ctx.invoice_date,
            cert_serie,
            cert_id_client
        );
    }

    if should_cache {
        if let Ok(conn) = db.conn.lock().map_err(|e| e.to_string()) {
            if let Err(e) = save_cached_certificate_payload(&conn, invoice_id, &ctx) {
                warn!("[CERT] Could not persist certificate cache for invoice {}: {}", invoice_id, e);
            }
        }
    } else if let Ok(conn) = db.conn.lock().map_err(|e| e.to_string()) {
        if let Some(payload) = load_cached_certificate_payload(&conn, invoice_id) {
            apply_cached_certificate_payload(&mut ctx, payload);
            warn!("[CERT] Using cached certificate data for invoice {} because fresh command data was incomplete.", invoice_id);
        } else if let Some(payload) = load_cached_certificate_payload(&conn, "DAILY_TODAY").or_else(|| load_latest_cached_certificate_payload(&conn)) {
            apply_cached_certificate_payload(&mut ctx, payload);
            warn!("[CERT] Using daily/latest cached certificate data for invoice {} because fresh command data was incomplete.", invoice_id);
        }
    }

    Ok(ctx)
}

pub fn generate_quality_certificate_html(ctx: &QualityCertificateContext) -> String {
    use base64::{engine::general_purpose, Engine as _};

    let epc_img = general_purpose::STANDARD.encode(include_bytes!("../../../public/EPC 16 EC1.png"));
    let iso_img = general_purpose::STANDARD.encode(include_bytes!("../../../public/KARIN-ISO1.png"));
    let stamp_img = general_purpose::STANDARD.encode(include_bytes!("../../../public/STAMPILA1.png"));
    let product_lines_html = ctx
        .product_lines
        .iter()
        .map(|line| {
            format!(
                r#"<div class="cat-line">{} {} ddm {} Lot {}</div>"#,
                line.denumire,
                line.data_productie,
                line.data_expirare,
                line.lot
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

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
            <div class="header-line">Jud. MM</div>
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
        <div class="cert-subtitle">{}</div>

        <div class="cert-intro">
            În conformitate cu prevederile legale privind răspunderea,
            se atestă calitatea produselor livrate: ouă consum categoria A,
            cu data ouatului:
        </div>

        <div class="cat-grid">
            <div class="cat-group">
                {}
            </div>
        </div>

        <div class="cert-body">
            <p>Ambalate la data de {}. Livrate beneficiarului: {}. Conform facturii/avizului nr. {} din {}.</p>
            <p>Transport auto: {} indeplinesc parametri de calitate specificati conform {} (salmonella negativ).</p>
            <p>Caracteristici tehnice de livrare: SALUBRE; Rasa LOHMANN BROWN, LOHMANN SANDY; Aspectul cojii intactă, curată de formă normală, uscată;</p>
            <p>Camera de aer: imobilă, cu înălțimea maximă 5 mm. Albușul: clar, translucid, consistență gelatinoasă si lipsit de corpuri străine de orice natura.</p>
            <p>Gălbenuș vizibil, în fascicol de lumina sub formă de umbră. Mirosul și gust caracteristic oului proaspăt, fără miros și gust străin.</p>
            <p>Data durabilității minime este de 28 zile iar data recomandata pentru vanzare este de 28 de zile de la momentul ouatului.</p>
            <p>Temperatura de păstrare: 5-18 grade Celsius,În magazine, ferite de razele soarelui si sursa de caldura.</p>
            <p>In magazinele de desfacere, ouale se pastreaza in locuri racoroase, curate, ferite de alte produse ale caror miros le pot imprumuta.</p>
            <p>Produs fragil! A se manipula cu atenție la transport și depozitare.</p>
            <p>Prezentul certificat întocmit conform Reg.(CE) nr.1234/22.10.2007 de instituire a unei organizari comune a pietelor agricole si privind</p>
            <p>dispozitii specifice referitoare la anumite produse agricole ("Regulamentul unic OCP"). Regulamentul (CE)NR.589/2008 al Comisiei din 23.06.2008</p>
            <p>de stabilire a normelor de aplicare a Reg.(CE)nr.1234/2007 al Consiliului privind standardele de comercializare a oualelor, modificat de Regulamentul </p>
            <p>CE 598/2008. Mentionam ca ouale produse de noi cu cod pro.3RO MM 013 provin de la gaini crescute in custi imbunatatie si cu cod producator</p>
            <p>2RO MM 040 provin de la gaini cresute in sistem volieră. conform standardelor U.E. in vigoare. </p>
        </div>

        <div class="cert-footer">
            <div class="footer-col">
            </div>
            <div class="footer-col footer-right">
                Țara de origine:România<br>
                Cod stație sortare RO MM 023<br>
                Cod producător 3RO MM 013<br>
                Cod producător 2RO MM 040
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
        ctx.cert_date,
        ctx.subtitle,
        product_lines_html,
        ctx.packed_date,
        ctx.beneficiary,
        ctx.invoice_display,
        ctx.invoice_date,
        ctx.car_number,
        ctx.bon_analiza,
        stamp_img,
    )
}

pub async fn save_invoice_certificate_file(
    db: &State<'_, Database>,
    invoice_id: &str,
    car_number: &str,
) -> Result<(String, String, String), String> {
    let context = build_quality_certificate_context(db, invoice_id, car_number).await?;
    let html = generate_quality_certificate_html(&context);

    let app_data_dir = paths::config_app_dir()?
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

