use log::info;
use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&app_data_dir).ok();
        let db_path = app_data_dir.join("facturi.db");
        info!("Opening database at: {:?}", db_path);

        let conn = Connection::open(db_path)?;

        // Run migrations
        conn.execute_batch(SCHEMA)?;
        
        // Run migrations for new columns
        run_migrations(&conn)?;

        info!("Database initialized successfully");

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

const SCHEMA: &str = r#"
    CREATE TABLE IF NOT EXISTS partners (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        cif TEXT,
        reg_com TEXT,
        cod TEXT,
        blocat TEXT,
        tva_la_incasare TEXT,
        persoana_fizica TEXT,
        cod_extern TEXT,
        cod_intern TEXT,
        observatii TEXT,
        data_adaugarii TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS locations (
        id TEXT PRIMARY KEY,
        partner_id TEXT NOT NULL,
        name TEXT NOT NULL,
        address TEXT,
        cod_sediu TEXT,
        localitate TEXT,
        strada TEXT,
        numar TEXT,
        judet TEXT,
        tara TEXT,
        cod_postal TEXT,
        telefon TEXT,
        email TEXT,
        inactiv TEXT,
        FOREIGN KEY (partner_id) REFERENCES partners(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS products (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        unit_of_measure TEXT NOT NULL,
        price REAL NOT NULL,
        class TEXT
    );

    CREATE TABLE IF NOT EXISTS invoices (
        id TEXT PRIMARY KEY,
        invoice_number INTEGER UNIQUE,
        partner_id TEXT NOT NULL,
        location_id TEXT NOT NULL,
        status TEXT NOT NULL DEFAULT 'pending',
        total_amount REAL NOT NULL DEFAULT 0,
        notes TEXT,
        created_at TEXT NOT NULL,
        sent_at TEXT,
        error_message TEXT,
        FOREIGN KEY (partner_id) REFERENCES partners(id),
        FOREIGN KEY (location_id) REFERENCES locations(id)
    );

    CREATE TABLE IF NOT EXISTS invoice_items (
        id TEXT PRIMARY KEY,
        invoice_id TEXT NOT NULL,
        product_id TEXT NOT NULL,
        quantity REAL NOT NULL,
        unit_price REAL NOT NULL,
        total_price REAL NOT NULL,
        FOREIGN KEY (invoice_id) REFERENCES invoices(id) ON DELETE CASCADE,
        FOREIGN KEY (product_id) REFERENCES products(id)
    );

    CREATE TABLE IF NOT EXISTS sync_metadata (
        entity_type TEXT PRIMARY KEY,
        last_synced_at TEXT
    );

    CREATE INDEX IF NOT EXISTS idx_invoices_status ON invoices(status);
    CREATE INDEX IF NOT EXISTS idx_invoices_partner ON invoices(partner_id);
    CREATE INDEX IF NOT EXISTS idx_invoice_items_invoice ON invoice_items(invoice_id);
    CREATE INDEX IF NOT EXISTS idx_locations_partner ON locations(partner_id);
"#;

fn run_migrations(conn: &rusqlite::Connection) -> Result<()> {
    // Add missing columns to partners table
    let partner_columns = vec![
        "ALTER TABLE partners ADD COLUMN cif TEXT;",
        "ALTER TABLE partners ADD COLUMN reg_com TEXT;",
        "ALTER TABLE partners ADD COLUMN cod TEXT;",
        "ALTER TABLE partners ADD COLUMN blocat TEXT;",
        "ALTER TABLE partners ADD COLUMN tva_la_incasare TEXT;",
        "ALTER TABLE partners ADD COLUMN persoana_fizica TEXT;",
        "ALTER TABLE partners ADD COLUMN cod_extern TEXT;",
        "ALTER TABLE partners ADD COLUMN cod_intern TEXT;",
        "ALTER TABLE partners ADD COLUMN observatii TEXT;",
        "ALTER TABLE partners ADD COLUMN data_adaugarii TEXT;",
    ];
    
    for sql in partner_columns {
        let _ = conn.execute(sql, []).ok();
    }
    
    // Add missing columns to locations table
    let location_columns = vec![
        "ALTER TABLE locations ADD COLUMN cod_sediu TEXT;",
        "ALTER TABLE locations ADD COLUMN localitate TEXT;",
        "ALTER TABLE locations ADD COLUMN strada TEXT;",
        "ALTER TABLE locations ADD COLUMN numar TEXT;",
        "ALTER TABLE locations ADD COLUMN judet TEXT;",
        "ALTER TABLE locations ADD COLUMN tara TEXT;",
        "ALTER TABLE locations ADD COLUMN cod_postal TEXT;",
        "ALTER TABLE locations ADD COLUMN telefon TEXT;",
        "ALTER TABLE locations ADD COLUMN email TEXT;",
        "ALTER TABLE locations ADD COLUMN inactiv TEXT;",
    ];
    
    for sql in location_columns {
        let _ = conn.execute(sql, []).ok();
    }
    
    Ok(())
}

pub fn init_database(app: &AppHandle) -> Result<Database, Box<dyn std::error::Error>> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    let db = Database::new(app_data_dir)?;
    Ok(db)
}
