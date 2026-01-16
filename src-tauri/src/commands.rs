use crate::database::Database;
use crate::mock_api;
use crate::models::*;
use crate::print_invoice;
use crate::api_client;
use chrono::Utc;
use log::info;
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
    let (partners, products) = match api_client::ApiClient::from_default() {
        Ok(api) => {
            // Try to get partners from API
            match api.get_all_partners().await {
                Ok(api_partners) => {
                    
                    // Try to get articles from API
                    match api.get_all_articles().await {
                        Ok(api_articles) => {
                            
                            // Convert API data to our models
                            let partners = convert_api_partners_to_model(api_partners);
                            let products = convert_api_articles_to_model(api_articles);
                            
                            (partners, products)
                        }
                        Err(_) => {
                            (mock_api::fetch_partners().await, mock_api::fetch_products().await)
                        }
                    }
                }
                Err(_) => {
                    (mock_api::fetch_partners().await, mock_api::fetch_products().await)
                }
            }
        }
        Err(_) => {
            (mock_api::fetch_partners().await, mock_api::fetch_products().await)
        }
    };

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
                "INSERT OR IGNORE INTO partners (id, name, cif, reg_com, cod, blocat, tva_la_incasare, persoana_fizica, cod_extern, cod_intern, observatii, data_adaugarii, created_at, updated_at, clasa, simbol_clasa, cod_clasa, categorie_pret_implicita, simbol_categorie_pret, scadenta_la_vanzare, scadenta_la_cumparare, discount_fix, tip_partener, mod_aplicare_discount, moneda, data_nastere, caracterizare_contabila_denumire, caracterizare_contabila_simbol) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28)",
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
                    "INSERT OR IGNORE INTO locations (id, partner_id, name, address, cod_sediu, localitate, strada, numar, judet, tara, cod_postal, telefon, email, inactiv) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
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
            conn.execute(
                "INSERT OR IGNORE INTO products (id, name, unit_of_measure, price, class) VALUES (?1, ?2, ?3, ?4, ?5)",
                (&product.id, &product.name, &product.unit_of_measure, product.price, &product.class),
            )
            .map_err(|e| format!("Failed to save product: {}", e))?;
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
                    
                    let address = format!(
                        "{} {}, {}, {}",
                        sediu.strada.as_deref().unwrap_or(""),
                        sediu.numar.as_deref().unwrap_or(""),
                        sediu.localitate.as_deref().unwrap_or(""),
                        sediu.judet.as_deref().unwrap_or("")
                    ).trim().to_string();

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
                scadenta_la_vanzare: api_partner.scadenta_la_vanzare,
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
            let price = api_article.pret_vanzare
                .as_ref()
                .and_then(|p| p.parse::<f64>().ok())
                .unwrap_or(0.0);

            Product {
                id: product_id,
                name: api_article.denumire,
                unit_of_measure: api_article.um,
                price,
                class: api_article.clasa,
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
        .prepare("SELECT id, name, cif, reg_com, cod, blocat, tva_la_incasare, persoana_fizica, cod_extern, cod_intern, observatii, data_adaugarii, created_at, updated_at, clasa, simbol_clasa, cod_clasa, inactiv, categorie_pret_implicita, simbol_categorie_pret, scadenta_la_vanzare, scadenta_la_cumparare, credit_client, discount_fix, tip_partener, mod_aplicare_discount, moneda, data_nastere, caracterizare_contabila_denumire, caracterizare_contabila_simbol FROM partners ORDER BY name")
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
        .prepare("SELECT id, name, cif, reg_com, cod, blocat, tva_la_incasare, persoana_fizica, cod_extern, cod_intern, observatii, data_adaugarii, created_at, updated_at, clasa, simbol_clasa, cod_clasa, inactiv, categorie_pret_implicita, simbol_categorie_pret, scadenta_la_vanzare, scadenta_la_cumparare, credit_client, discount_fix, tip_partener, mod_aplicare_discount, moneda, data_nastere, caracterizare_contabila_denumire, caracterizare_contabila_simbol FROM partners WHERE name LIKE ?1 ORDER BY name")
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
pub fn get_products(db: State<'_, Database>) -> Result<Vec<Product>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name, unit_of_measure, price, class FROM products ORDER BY name")
        .map_err(|e| e.to_string())?;

    let products: Vec<Product> = stmt
        .query_map([], |row| {
            Ok(Product {
                id: row.get(0)?,
                name: row.get(1)?,
                unit_of_measure: row.get(2)?,
                price: row.get(3)?,
                class: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(products)
}

#[tauri::command]
pub fn search_products(db: State<'_, Database>, query: String) -> Result<Vec<Product>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let search_query = format!("%{}%", query);

    let mut stmt = conn
        .prepare("SELECT id, name, unit_of_measure, price, class FROM products WHERE name LIKE ?1 OR class LIKE ?1 ORDER BY name")
        .map_err(|e| e.to_string())?;

    let products: Vec<Product> = stmt
        .query_map([&search_query], |row| {
            Ok(Product {
                id: row.get(0)?,
                name: row.get(1)?,
                unit_of_measure: row.get(2)?,
                price: row.get(3)?,
                class: row.get(4)?,
            })
        })
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
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    let invoice_id = Uuid::new_v4().to_string();

    // Get partner name
    let partner_name: String = conn
        .query_row(
            "SELECT name FROM partners WHERE id = ?1",
            [&request.partner_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Partner not found: {}", e))?;

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
        let (product_name, price, um): (String, f64, String) = conn
            .query_row(
                "SELECT name, price, unit_of_measure FROM products WHERE id = ?1",
                [&item.product_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| format!("Product not found: {}", e))?;

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

    // Get next invoice number
    let invoice_number: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(invoice_number), 0) + 1 FROM invoices",
            [],
            |row| row.get(0),
        )
        .unwrap_or(1);

    // Insert invoice with auto-generated number
    conn.execute(
        "INSERT INTO invoices (id, invoice_number, partner_id, location_id, status, total_amount, notes, created_at) VALUES (?1, ?2, ?3, ?4, 'pending', ?5, ?6, ?7)",
        (&invoice_id, invoice_number, &request.partner_id, &request.location_id, total_amount, &request.notes, &now),
    )
    .map_err(|e| e.to_string())?;

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
                (SELECT COUNT(*) FROM invoice_items WHERE invoice_id = i.id)
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
                (SELECT COUNT(*) FROM invoice_items WHERE invoice_id = i.id)
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
                })
            },
        )
        .map_err(|e| format!("Invoice not found: {}", e))?;

    // Get invoice items
    let mut stmt = conn
        .prepare(
            r#"
            SELECT
                ii.id, ii.invoice_id, ii.product_id, pr.name, ii.quantity, ii.unit_price, pr.unit_of_measure, ii.total_price
            FROM invoice_items ii
            JOIN products pr ON ii.product_id = pr.id
            WHERE ii.invoice_id = ?1
            "#,
        )
        .map_err(|e| e.to_string())?;

    let items: Vec<InvoiceItem> = stmt
        .query_map([&invoice_id], |row| {
            Ok(InvoiceItem {
                id: row.get(0)?,
                invoice_id: row.get(1)?,
                product_id: row.get(2)?,
                product_name: row.get(3)?,
                quantity: row.get(4)?,
                unit_price: row.get(5)?,
                unit_of_measure: row.get(6)?,
                total_price: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(InvoiceDetail { invoice, items })
}

#[tauri::command]
pub async fn send_invoice(db: State<'_, Database>, invoice_id: String) -> Result<Invoice, String> {
    // Update status to 'sending'
    {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE invoices SET status = 'sending' WHERE id = ?1",
            [&invoice_id],
        )
        .map_err(|e| e.to_string())?;
    }

    // Attempt to send (50% failure rate)
    let result = mock_api::send_invoice_to_external().await;
    let now = Utc::now().to_rfc3339();

    // Update based on result and return the invoice
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    let invoice_sent = match result {
        Ok(()) => {
            conn.execute(
                "UPDATE invoices SET status = 'sent', sent_at = ?1, error_message = NULL WHERE id = ?2",
                [&now, &invoice_id],
            )
            .map_err(|e| e.to_string())?;
            true
        }
        Err(ref error) => {
            conn.execute(
                "UPDATE invoices SET status = 'failed', error_message = ?1 WHERE id = ?2",
                [error, &invoice_id],
            )
            .map_err(|e| e.to_string())?;
            false
        }
    };

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
                })
            },
        )
        .map_err(|e| format!("Invoice not found: {}", e))?;

    // Drop the lock before async operation
    drop(conn);

    // If invoice was sent successfully, trigger automatic printing
    // Note: Printing will be triggered by frontend calling print_invoice_to_html
    // We can't use await here with the db state due to Send trait requirements
    if invoice_sent {
        info!("Invoice {} ready for printing", invoice_id);
    }

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
        // Get list of Windows printers using WMI
        let output = std::process::Command::new("powershell")
            .args(&[
                "-NoProfile",
                "-Command",
                "Get-WmiObject Win32_Printer | Select-Object -ExpandProperty Name",
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

    // Fetch invoice details
    let invoice = get_invoice_for_print(&conn, &invoice_id)?;

    // Fetch invoice items
    let mut stmt = conn
        .prepare(
            r#"
            SELECT ii.id, ii.product_id, p.name, ii.quantity, ii.unit_price, p.unit_of_measure, ii.total_price
            FROM invoice_items ii
            JOIN products p ON ii.product_id = p.id
            WHERE ii.invoice_id = ?1
            "#,
        )
        .map_err(|e| e.to_string())?;

    let items: Vec<InvoiceItem> = stmt
        .query_map([&invoice_id], |row| {
            Ok(InvoiceItem {
                id: row.get(0)?,
                invoice_id: invoice_id.clone(),
                product_id: row.get(1)?,
                product_name: row.get(2)?,
                quantity: row.get(3)?,
                unit_price: row.get(4)?,
                unit_of_measure: row.get(5)?,
                total_price: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    // Read logo and convert to base64
    let logo_base64 = read_logo_to_base64();

    // Generate HTML
    let html = print_invoice::generate_invoice_html(&invoice, &items, invoice_number, logo_base64.as_deref());

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
                        
                        // Give Edge time to write the file (reduced for speed)
                        std::thread::sleep(std::time::Duration::from_millis(800));
                        
                        // Check if PDF was created
                        if std::path::Path::new(&pdf_path_str).exists() {
                            pdf_generated = true;
                            print_file = pdf_path_str.clone();
                            info!("PDF generated successfully at: {}", pdf_path_str);
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
) -> Result<Invoice, String> {
    conn.query_row(
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
        [invoice_id],
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
            })
        },
    )
    .map_err(|e| format!("Invoice not found: {}", e))
}

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
