use crate::database::Database;
use crate::mock_api;
use crate::models::*;
use chrono::Utc;
use log::info;
use tauri::State;
use uuid::Uuid;

// ==================== SYNC COMMANDS ====================

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
    info!("Starting data sync...");

    // Fetch mock data FIRST (async operations done before touching the mutex)
    let partners = mock_api::fetch_partners().await;
    let products = mock_api::fetch_products().await;

    info!("Fetched {} partners, {} products from API", partners.len(), products.len());

    // Now do all database operations synchronously
    // Clone the data we need since we can't hold MutexGuard across await points
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
                "INSERT OR REPLACE INTO partners (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
                (&partner.id, &partner.name, &partner.created_at, &partner.updated_at),
            )
            .map_err(|e| format!("Failed to save partner: {}", e))?;

            // Save locations
            for location in &partner.locations {
                conn.execute(
                    "INSERT OR REPLACE INTO locations (id, partner_id, name, address) VALUES (?1, ?2, ?3, ?4)",
                    (&location.id, &location.partner_id, &location.name, &location.address),
                )
                .map_err(|e| format!("Failed to save location: {}", e))?;
            }
        }

        // Save products
        for product in &products {
            conn.execute(
                "INSERT OR REPLACE INTO products (id, name, unit_of_measure, price, class) VALUES (?1, ?2, ?3, ?4, ?5)",
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

#[tauri::command]
pub fn check_online_status() -> Result<bool, String> {
    // For now, always return true - the frontend handles online/offline via navigator.onLine
    Ok(true)
}

// ==================== PARTNER COMMANDS ====================

#[tauri::command]
pub fn get_partners(db: State<'_, Database>) -> Result<Vec<PartnerWithLocations>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, name, created_at, updated_at FROM partners ORDER BY name")
        .map_err(|e| e.to_string())?;

    let partners: Vec<(String, String, String, String)> = stmt
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut result = Vec::new();

    for (id, name, created_at, updated_at) in partners {
        let mut loc_stmt = conn
            .prepare("SELECT id, partner_id, name, address FROM locations WHERE partner_id = ?1")
            .map_err(|e| e.to_string())?;

        let locations: Vec<Location> = loc_stmt
            .query_map([&id], |row| {
                Ok(Location {
                    id: row.get(0)?,
                    partner_id: row.get(1)?,
                    name: row.get(2)?,
                    address: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        result.push(PartnerWithLocations {
            id,
            name,
            created_at,
            updated_at,
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
        .prepare("SELECT id, name, created_at, updated_at FROM partners WHERE name LIKE ?1 ORDER BY name")
        .map_err(|e| e.to_string())?;

    let partners: Vec<(String, String, String, String)> = stmt
        .query_map([&search_query], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut result = Vec::new();

    for (id, name, created_at, updated_at) in partners {
        let mut loc_stmt = conn
            .prepare("SELECT id, partner_id, name, address FROM locations WHERE partner_id = ?1")
            .map_err(|e| e.to_string())?;

        let locations: Vec<Location> = loc_stmt
            .query_map([&id], |row| {
                Ok(Location {
                    id: row.get(0)?,
                    partner_id: row.get(1)?,
                    name: row.get(2)?,
                    address: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        result.push(PartnerWithLocations {
            id,
            name,
            created_at,
            updated_at,
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

    // Get location name
    let location_name: String = conn
        .query_row(
            "SELECT name FROM locations WHERE id = ?1",
            [&request.location_id],
            |row| row.get(0),
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

    // Insert invoice
    conn.execute(
        "INSERT INTO invoices (id, partner_id, location_id, status, total_amount, notes, created_at) VALUES (?1, ?2, ?3, 'pending', ?4, ?5, ?6)",
        (&invoice_id, &request.partner_id, &request.location_id, total_amount, &request.notes, &now),
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

    info!(
        "Created invoice {} with {} items, total: {}",
        invoice_id,
        items_to_insert.len(),
        total_amount
    );

    Ok(Invoice {
        id: invoice_id,
        partner_id: request.partner_id,
        partner_name,
        location_id: request.location_id,
        location_name,
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
                i.id, i.partner_id, p.name, i.location_id, l.name,
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
                i.id, i.partner_id, p.name, i.location_id, l.name,
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
                location_id: row.get(3)?,
                location_name: row.get(4)?,
                status: InvoiceStatus::from(row.get::<_, String>(5)?),
                total_amount: row.get(6)?,
                notes: row.get(7)?,
                created_at: row.get(8)?,
                sent_at: row.get(9)?,
                error_message: row.get(10)?,
                item_count: row.get(11)?,
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
                i.id, i.partner_id, p.name, i.location_id, l.name,
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
                    location_id: row.get(3)?,
                    location_name: row.get(4)?,
                    status: InvoiceStatus::from(row.get::<_, String>(5)?),
                    total_amount: row.get(6)?,
                    notes: row.get(7)?,
                    created_at: row.get(8)?,
                    sent_at: row.get(9)?,
                    error_message: row.get(10)?,
                    item_count: row.get(11)?,
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

    match result {
        Ok(()) => {
            conn.execute(
                "UPDATE invoices SET status = 'sent', sent_at = ?1, error_message = NULL WHERE id = ?2",
                [&now, &invoice_id],
            )
            .map_err(|e| e.to_string())?;
            info!("Invoice {} sent successfully", invoice_id);
        }
        Err(ref error) => {
            conn.execute(
                "UPDATE invoices SET status = 'failed', error_message = ?1 WHERE id = ?2",
                [error, &invoice_id],
            )
            .map_err(|e| e.to_string())?;
            info!("Invoice {} failed to send: {}", invoice_id, error);
        }
    }

    // Fetch and return the updated invoice
    let invoice: Invoice = conn
        .query_row(
            r#"
            SELECT
                i.id, i.partner_id, p.name, i.location_id, l.name,
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
                    location_id: row.get(3)?,
                    location_name: row.get(4)?,
                    status: InvoiceStatus::from(row.get::<_, String>(5)?),
                    total_amount: row.get(6)?,
                    notes: row.get(7)?,
                    created_at: row.get(8)?,
                    sent_at: row.get(9)?,
                    error_message: row.get(10)?,
                    item_count: row.get(11)?,
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
