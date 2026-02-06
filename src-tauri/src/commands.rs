use crate::database::Database;
use crate::mock_api;
use crate::models::*;
use crate::print_invoice;
use crate::api_client;
use chrono::{Utc, Datelike};
use log::{info, warn, error};
use tauri::State;
use uuid::Uuid;
use rusqlite::params;

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

// ==================== SYNC COMMANDS ====================

#[tauri::command]
pub fn clear_database(db: State<'_, Database>) -> Result<(), String> {
    db.clear_sync_data().map_err(|e| e.to_string())
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

    // Try to get partners from API
    let api_partners = api.get_all_partners().await.map_err(|e| format!("Failed to fetch partners: {}", e))?;
                    
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
    let partners = convert_api_partners_to_model(api_partners);
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
fn convert_api_partners_to_model(api_partners: Vec<api_client::PartnerInfo>) -> Vec<PartnerWithLocations> {
    api_partners
        .into_iter()
        .map(|api_partner| {
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

            // Convert locations with all fields
            let locations: Vec<Location> = api_partner.sedii
                .into_iter()
                .map(|sediu| {
                    // Generate location ID if empty
                    let location_id = if sediu.id_sediu.is_empty() {
                        sediu.cod_sediu.clone()
                            .filter(|c| !c.is_empty())
                            .unwrap_or_else(|| Uuid::new_v4().to_string())
                    } else {
                        sediu.id_sediu
                    };
                    
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

                    Location {
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
                    }
                })
                .collect();

            PartnerWithLocations {
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
            }
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
            match api.get_all_partners().await {
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
    let (invoice, items, partner_cod, location_id_sediu, invoice_number): (Invoice, Vec<(String, f64, f64, String)>, Option<String>, Option<String>, i64) = {
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

        let partner_cod: Option<String> = conn
            .query_row("SELECT cod_intern FROM partners WHERE id = ?1", [&invoice.partner_id], |row| row.get(0))
            .ok();

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

        (invoice, items, partner_cod, location_id_sediu, invoice_number)
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

    // Build WME items
    let gestiune = agent_settings.simbol_gestiune_livrare.clone().unwrap();
    let wme_items: Vec<api_client::WmeInvoiceItem> = items
        .into_iter()
        .map(|(product_id, quantity, price, um)| api_client::WmeInvoiceItem {
            id_articol: product_id,
            cant: quantity,
            pret: price,
            um: Some(um),
            gestiune: Some(gestiune.clone()),
            observatii: None,
            tva: None,
        })
        .collect();

    // Build WME request
    let wme_request = api_client::WmeInvoiceRequest {
        cod_partener: partner_cod.clone().unwrap(),
        cod_sediu: location_id_sediu.clone(),
        nume_delegate: agent_settings.delegate_name.clone().unwrap_or_default(),
        act_delegate: agent_settings.delegate_act.clone().unwrap_or_default(),
        tip_document: Some("FACTURA IESIRE".to_string()),
        an_lucru: Some(an_lucru.to_string()),
        luna_lucru: Some(luna_lucru.to_string()),
        cod_subunitate: None, // Empty for Central
        documente: vec![api_client::WmeDocument {
            tip_document: "FACTURA IESIRE".to_string(),
            simbol_gestiune: gestiune.clone(),
            nume_gestiune: gestiune.clone(),
            serie_document: agent_settings.carnet_series.clone(),
            numar_document: Some(invoice_number.to_string()), // Folosim numărul din aplicație
            simbol_carnet: Some(agent_settings.carnet_series.clone().unwrap()),
            simbol_carnet_livr: Some(agent_settings.simbol_carnet_livr.clone().unwrap()),
            simbol_gestiune_livrare: Some(agent_settings.simbol_gestiune_livrare.clone().unwrap()),
            numerotare_automata: None, // Nu mai folosim numerotare automată - folosim NrDoc
            data: Some(data_formatted.clone()),
            data_livr: Some(data_formatted),
            cod_client: Some(partner_cod.unwrap()),
            id_sediu: location_id_sediu,
            agent: Some(agent_settings.agent_name.unwrap()),
            observatii: invoice.notes.clone(),
            items: Some(wme_items),
        }],
        articole: vec![],
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
    let (partner_name, notes, created_at, invoice_number): (String, Option<String>, String, i64) = conn
        .query_row(
            "SELECT p.name, i.notes, i.created_at, i.invoice_number FROM invoices i JOIN partners p ON i.partner_id = p.id WHERE i.id = ?1",
            [&invoice_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|e| format!("Invoice not found: {}", e))?;

    // Get agent settings
    let agent_settings = get_agent_settings(db.clone()).map_err(|e| e.to_string())?;

    // Get partner CodIntern and location ID
    let (partner_cod, location_id_sediu): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT p.cod_intern, l.id_sediu FROM invoices i JOIN partners p ON i.partner_id = p.id JOIN locations l ON i.location_id = l.id WHERE i.id = ?1",
            [&invoice_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
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

    // Parse invoice date
    let invoice_date = chrono::DateTime::parse_from_rfc3339(&created_at)
        .map_err(|e| format!("Failed to parse invoice date: {}", e))?;
    
    let an_lucru = invoice_date.year();
    let luna_lucru = invoice_date.month() as i32;
    let data_formatted = invoice_date.format("%d.%m.%Y").to_string();

    // Build WME items
    let gestiune = agent_settings.simbol_gestiune_livrare.clone().unwrap();
    let wme_items: Vec<api_client::WmeInvoiceItem> = items
        .into_iter()
        .map(|(product_id, quantity, price, um)| api_client::WmeInvoiceItem {
            id_articol: product_id,
            cant: quantity,
            pret: price,
            um: Some(um),
            gestiune: Some(gestiune.clone()),
            observatii: None,
            tva: None,
        })
        .collect();

    // Build WME request
    let wme_request = api_client::WmeInvoiceRequest {
        cod_partener: partner_cod.clone().unwrap(),
        cod_sediu: location_id_sediu.clone(),
        nume_delegate: agent_settings.delegate_name.clone().unwrap_or_default(),
        act_delegate: agent_settings.delegate_act.clone().unwrap_or_default(),
        tip_document: Some("FACTURA IESIRE".to_string()),
        an_lucru: Some(an_lucru.to_string()),
        luna_lucru: Some(luna_lucru.to_string()),
        cod_subunitate: None,
        documente: vec![api_client::WmeDocument {
            tip_document: "FACTURA IESIRE".to_string(),
            simbol_gestiune: gestiune.clone(),
            nume_gestiune: gestiune.clone(),
            serie_document: agent_settings.carnet_series.clone(),
            numar_document: Some(invoice_number.to_string()), // Folosim numărul din aplicație
            simbol_carnet: Some(agent_settings.carnet_series.clone().unwrap()),
            simbol_carnet_livr: Some(agent_settings.simbol_carnet_livr.clone().unwrap()),
            simbol_gestiune_livrare: Some(agent_settings.simbol_gestiune_livrare.clone().unwrap()),
            numerotare_automata: None, // Nu mai folosim numerotare automată - folosim NrDoc
            data: Some(data_formatted.clone()),
            data_livr: Some(data_formatted),
            cod_client: Some(partner_cod.unwrap()),
            id_sediu: location_id_sediu,
            agent: Some(agent_settings.agent_name.unwrap()),
            observatii: notes.clone(),
            items: Some(wme_items),
        }],
        articole: vec![],
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
            
            // SumatraPDF with no scaling to keep normal size
            let result = std::process::Command::new(&sumatra_path)
                .args(&[
                    "-print-to",
                    &printer,
                    "-print-settings",
                    "noscale",
                    &print_file,
                    "-silent",
                    "-exit-when-done",
                    "-exit-on-print",
                ])
                .spawn();
            
            match result {
                Ok(_) => {
                    info!("Print job sent successfully to printer '{}': {}", printer, invoice_id);
                }
                Err(e) => {
                    info!("SumatraPDF print failed: {}", e);
                }
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
        "SELECT agent_name, carnet_series, simbol_carnet_livr, simbol_gestiune_livrare, cod_carnet, cod_carnet_livr, delegate_name, delegate_act, car_number, invoice_number_start, invoice_number_end, invoice_number_current FROM agent_settings WHERE id = 1",
        [],
        |row| Ok(AgentSettings {
            agent_name: row.get(0)?,
            carnet_series: row.get(1)?,
            simbol_carnet_livr: row.get(2)?,
            simbol_gestiune_livrare: row.get(3)?,
            cod_carnet: row.get(4)?,
            cod_carnet_livr: row.get(5)?,
            delegate_name: row.get(6)?,
            delegate_act: row.get(7)?,
            car_number: row.get(8)?,
            invoice_number_start: row.get(9)?,
            invoice_number_end: row.get(10)?,
            invoice_number_current: row.get(11)?,
        }),
    );

    match result {
        Ok(settings) => Ok(settings),
        Err(_) => Ok(AgentSettings {
            agent_name: None,
            carnet_series: None,
            simbol_carnet_livr: None,
            simbol_gestiune_livrare: None,
            cod_carnet: None,
            cod_carnet_livr: None,
            delegate_name: None,
            delegate_act: None,
            car_number: None,
            invoice_number_start: Some(1),
            invoice_number_end: Some(99999),
            invoice_number_current: Some(1),
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
    cod_carnet: Option<String>,
    cod_carnet_livr: Option<String>,
    delegate_name: Option<String>,
    delegate_act: Option<String>,
    car_number: Option<String>,
    invoice_number_start: Option<i64>,
    invoice_number_end: Option<i64>,
    invoice_number_current: Option<i64>,
) -> Result<AgentSettings, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();

    // Smart logic for invoice numbering:
    // If invoice_number_start is provided and current is less than start, set current = start
    let final_current = match (invoice_number_start, invoice_number_current) {
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

    conn.execute(
        "INSERT INTO agent_settings (id, agent_name, carnet_series, simbol_carnet_livr, simbol_gestiune_livrare, cod_carnet, cod_carnet_livr, delegate_name, delegate_act, car_number, invoice_number_start, invoice_number_end, invoice_number_current, updated_at) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13) \
         ON CONFLICT(id) DO UPDATE SET agent_name = excluded.agent_name, carnet_series = excluded.carnet_series, simbol_carnet_livr = excluded.simbol_carnet_livr, simbol_gestiune_livrare = excluded.simbol_gestiune_livrare, cod_carnet = excluded.cod_carnet, cod_carnet_livr = excluded.cod_carnet_livr, delegate_name = excluded.delegate_name, delegate_act = excluded.delegate_act, car_number = excluded.car_number, invoice_number_start = excluded.invoice_number_start, invoice_number_end = excluded.invoice_number_end, invoice_number_current = excluded.invoice_number_current, updated_at = excluded.updated_at",
        (&agent_name, &carnet_series, &simbol_carnet_livr, &simbol_gestiune_livrare, &cod_carnet, &cod_carnet_livr, &delegate_name, &delegate_act, &car_number, &invoice_number_start, &invoice_number_end, &final_current, &now),
    )
    .map_err(|e| e.to_string())?;

    Ok(AgentSettings {
        agent_name,
        carnet_series,
        simbol_carnet_livr,
        simbol_gestiune_livrare,
        cod_carnet,
        cod_carnet_livr,
        delegate_name,
        delegate_act,
        car_number,
        invoice_number_start: invoice_number_start.map(|v| v as i32),
        invoice_number_end: invoice_number_end.map(|v| v as i32),
        invoice_number_current: final_current.map(|v| v as i32),
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
