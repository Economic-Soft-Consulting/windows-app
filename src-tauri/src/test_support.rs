//! Shared fixtures for tests across modules.
//!
//! Databases are built through `Database::new` rather than a hand-written schema, so tests
//! run against the real migrations. A hand-rolled fixture drifts from `database.rs` the
//! moment a migration is added, and then tests pass against a schema that does not ship.

use crate::database::Database;
use std::path::PathBuf;

/// A real database in a temp directory, with every migration applied.
///
/// Returns the directory so the caller can clean it up after dropping the connection.
pub fn temp_db(tag: &str) -> (PathBuf, Database) {
    let dir = std::env::temp_dir().join(format!("karin_{}_{}", tag, uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let db = Database::new(dir.clone()).expect("init db");
    (dir, db)
}

/// Seeds one partner, location, product and invoice, priced so the gross total is 85.91
/// (78.82 net at 9% VAT — the case from the field that motivated the rounding fix).
pub fn seed_invoice(
    conn: &rusqlite::Connection,
    invoice_id: &str,
    number: i64,
    partner: &str,
    series: &str,
) {
    conn.execute(
        "INSERT OR IGNORE INTO partners (id, name, created_at, updated_at) \
         VALUES (?1, 'Test', '2026-01-01', '2026-01-01')",
        [partner],
    )
    .unwrap();
    conn.execute(
        "INSERT OR IGNORE INTO locations (id, partner_id, name) VALUES ('L1', ?1, 'Sediu')",
        [partner],
    )
    .unwrap();
    conn.execute(
        "INSERT OR IGNORE INTO products (id, name, unit_of_measure, price, procent_tva) \
         VALUES ('PR1', 'Oua', 'BUC', 1.0, '9')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO invoices (id, invoice_number, invoice_series, partner_id, location_id, \
                               status, total_amount, total_amount_gross, created_at) \
         VALUES (?1, ?2, ?3, ?4, 'L1', 'sent', 78.82, 85.91, '2026-01-01')",
        rusqlite::params![invoice_id, number, series, partner],
    )
    .unwrap();
}

/// The `rest` column of every balance row for one partner.
pub fn balance_rest(conn: &rusqlite::Connection, partner: &str) -> Vec<f64> {
    let mut query = crate::db::balances::build_client_balances_query();
    query.push_str(" AND TRIM(q.id_partener) = TRIM(?1)");
    let mut stmt = conn.prepare(&query).unwrap();
    let out = stmt
        .query_map([partner], |r| r.get::<_, Option<f64>>(10))
        .unwrap()
        .filter_map(|r| r.ok())
        .map(|v| v.unwrap_or(0.0))
        .collect();
    out
}
