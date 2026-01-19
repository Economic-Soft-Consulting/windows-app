use log::info;
use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use chrono::Utc;

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

    pub fn clear_sync_data(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        info!("Clearing partners, locations, products and sync metadata...");
        
        // Disable foreign key constraints temporarily
        conn.execute("PRAGMA foreign_keys = OFF", [])?;
        
        // Delete all sync data
        conn.execute("DELETE FROM offer_items", [])?;
        conn.execute("DELETE FROM offers", [])?;
        conn.execute("DELETE FROM locations", [])?;
        conn.execute("DELETE FROM partners", [])?;
        conn.execute("DELETE FROM products", [])?;
        conn.execute("DELETE FROM sync_metadata", [])?;
        
        // Re-enable foreign key constraints
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        
        info!("✅ Successfully cleared all sync data");
        
        Ok(())
    }
}

const SCHEMA: &str = r#"
    CREATE TABLE IF NOT EXISTS partners (
        id TEXT PRIMARY KEY,
        cod TEXT,
        name TEXT NOT NULL,
        cif TEXT,
        reg_com TEXT,
        blocat TEXT,
        tva_la_incasare TEXT,
        persoana_fizica TEXT,
        cod_extern TEXT,
        cod_intern TEXT,
        observatii TEXT,
        data_adaugarii TEXT,
        clasa TEXT,
        simbol_clasa TEXT,
        cod_clasa TEXT,
        inactiv TEXT,
        categorie_pret_implicita TEXT,
        simbol_categorie_pret TEXT,
        scadenta_la_vanzare TEXT,
        scadenta_la_cumparare TEXT,
        credit_client TEXT,
        discount_fix TEXT,
        tip_partener TEXT,
        mod_aplicare_discount TEXT,
        moneda TEXT,
        data_nastere TEXT,
        caracterizare_contabila_denumire TEXT,
        caracterizare_contabila_simbol TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS locations (
        id TEXT PRIMARY KEY,
        partner_id TEXT NOT NULL,
        id_sediu TEXT,
        cod_sediu TEXT,
        name TEXT NOT NULL,
        localitate TEXT,
        strada TEXT,
        numar TEXT,
        numar2 TEXT,
        bloc TEXT,
        scara TEXT,
        etaj TEXT,
        apartament TEXT,
        judet TEXT,
        tara TEXT,
        sector TEXT,
        cod_postal TEXT,
        cod_siruta TEXT,
        telefon TEXT,
        email TEXT,
        gln TEXT,
        latitudine TEXT,
        longitudine TEXT,
        traseu_livrare TEXT,
        poz_traseu_livrare TEXT,
        traseu_vizitare TEXT,
        poz_traseu_vizitare TEXT,
        gestiune_livrare TEXT,
        simbol_gest_livrare TEXT,
        cod_subunitate TEXT,
        subunitate TEXT,
        tip_sediu TEXT,
        scadenta_la_vanzare TEXT,
        zile_depasire TEXT,
        inactiv TEXT,
        cod_client TEXT,
        denumire_superior TEXT,
        agent_marca TEXT,
        agent_nume TEXT,
        agent_prenume TEXT,
        address TEXT,
        FOREIGN KEY (partner_id) REFERENCES partners(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS products (
        id TEXT PRIMARY KEY,
        cod_obiect TEXT,
        cod_articol TEXT,
        name TEXT NOT NULL,
        unit_of_measure TEXT NOT NULL,
        price REAL NOT NULL,
        pret_cu_tva REAL,
        pret_valuta REAL,
        pret_minim REAL,
        pret_referinta REAL,
        class TEXT,
        simbol_clasa TEXT,
        cod_clasa TEXT,
        clasa_web TEXT,
        simbol_clasa_web TEXT,
        cod_clasa_web TEXT,
        clasa_stat TEXT,
        simbol_clasa_stat TEXT,
        cod_clasa_stat TEXT,
        producator TEXT,
        id_producator TEXT,
        gestiune_implicita TEXT,
        simbol_cont_implicit TEXT,
        simbol_tip_cont_implicit TEXT,
        cod_locatie_implicita TEXT,
        cod_ext_locatie_implicita TEXT,
        den_locatie_implicita TEXT,
        cod_extern TEXT,
        cod_intern TEXT,
        procent_tva TEXT,
        um_implicita TEXT,
        paritate_um_implicita TEXT,
        um_specifica TEXT,
        um_alternativa TEXT,
        relatie_um_spec TEXT,
        masa TEXT,
        volum TEXT,
        greutate_specifica TEXT,
        serviciu TEXT,
        are_data_expirare TEXT,
        cod_vamal TEXT,
        cod_d394 TEXT,
        data_adaugarii TEXT,
        vizibil_comenzi_online TEXT,
        inactiv_comenzi_online TEXT,
        cod_catalog TEXT,
        promotie TEXT,
        discount_promo TEXT,
        zile_plata TEXT,
        inactiv TEXT,
        blocat TEXT,
        descriere TEXT,
        dci TEXT,
        tip_serie TEXT,
        cod_cnas TEXT,
        coef_cnas TEXT,
        check_autenticitate TEXT,
        d1 TEXT,
        d2 TEXT,
        d3 TEXT,
        simbol_centru_cost TEXT,
        cod_cpv TEXT,
        constructie_noua TEXT,
        risc_fiscal TEXT,
        luni_garantie TEXT,
        caract_suplim TEXT,
        zile_valabil TEXT,
        adaos_exceptie TEXT,
        nefacturabil TEXT,
        simbol_cont_serv TEXT,
        fara_stoc TEXT,
        voucher_cadou TEXT,
        categorie_pret_implicita TEXT
    );

    CREATE TABLE IF NOT EXISTS offers (
        id TEXT PRIMARY KEY,
        id_client TEXT,
        numar TEXT,
        data_inceput TEXT,
        data_sfarsit TEXT,
        anulata TEXT,
        client TEXT,
        tip_oferta TEXT,
        furnizor TEXT,
        id_furnizor TEXT,
        cod_fiscal TEXT,
        simbol_clasa TEXT,
        moneda TEXT,
        observatii TEXT,
        extensie_document TEXT
    );

    CREATE TABLE IF NOT EXISTS offer_items (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        offer_id TEXT NOT NULL,
        id_client TEXT,
        product_id TEXT,
        denumire TEXT,
        um TEXT,
        cant_minima TEXT,
        cant_maxima TEXT,
        cant_optima TEXT,
        pret REAL,
        discount TEXT,
        proc_adaos TEXT,
        pret_ref TEXT,
        pret_cu_proc_adaos TEXT,
        observatii TEXT,
        cod_oferta1 TEXT,
        extensie_linie TEXT,
        FOREIGN KEY (offer_id) REFERENCES offers(id) ON DELETE CASCADE
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

    CREATE TABLE IF NOT EXISTS agent_settings (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        agent_name TEXT,
        carnet_series TEXT,
        simbol_carnet_livr TEXT,
        simbol_gestiune_livrare TEXT,
        cod_carnet TEXT,
        cod_carnet_livr TEXT,
        updated_at TEXT
    );

    CREATE TABLE IF NOT EXISTS db_migrations (
        version INTEGER PRIMARY KEY,
        applied_at TEXT NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_invoices_status ON invoices(status);
    CREATE INDEX IF NOT EXISTS idx_invoices_partner ON invoices(partner_id);
    CREATE INDEX IF NOT EXISTS idx_invoice_items_invoice ON invoice_items(invoice_id);
    CREATE INDEX IF NOT EXISTS idx_locations_partner ON locations(partner_id);
    CREATE INDEX IF NOT EXISTS idx_offer_items_client_product ON offer_items(id_client, product_id);
"#;

fn run_migrations(conn: &rusqlite::Connection) -> Result<()> {
    // Check current migration version
    let current_version: i32 = conn
        .query_row("SELECT COALESCE(MAX(version), 0) FROM db_migrations", [], |row| row.get(0))
        .unwrap_or(0);

    info!("Current database migration version: {}", current_version);

    // Migration 1: Add partner columns (v0.1.0 - v0.2.0)
    if current_version < 1 {
        info!("Applying migration 1: Partner columns");
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
            "ALTER TABLE partners ADD COLUMN clasa TEXT;",
            "ALTER TABLE partners ADD COLUMN simbol_clasa TEXT;",
            "ALTER TABLE partners ADD COLUMN cod_clasa TEXT;",
            "ALTER TABLE partners ADD COLUMN inactiv TEXT;",
            "ALTER TABLE partners ADD COLUMN categorie_pret_implicita TEXT;",
            "ALTER TABLE partners ADD COLUMN simbol_categorie_pret TEXT;",
            "ALTER TABLE partners ADD COLUMN scadenta_la_vanzare TEXT;",
            "ALTER TABLE partners ADD COLUMN scadenta_la_cumparare TEXT;",
            "ALTER TABLE partners ADD COLUMN credit_client TEXT;",
            "ALTER TABLE partners ADD COLUMN discount_fix TEXT;",
            "ALTER TABLE partners ADD COLUMN tip_partener TEXT;",
            "ALTER TABLE partners ADD COLUMN mod_aplicare_discount TEXT;",
            "ALTER TABLE partners ADD COLUMN moneda TEXT;",
            "ALTER TABLE partners ADD COLUMN data_nastere TEXT;",
            "ALTER TABLE partners ADD COLUMN caracterizare_contabila_denumire TEXT;",
            "ALTER TABLE partners ADD COLUMN caracterizare_contabila_simbol TEXT;",
        ];
        
        for sql in partner_columns {
            let _ = conn.execute(sql, []).ok();
        }

        conn.execute("INSERT INTO db_migrations (version, applied_at) VALUES (1, ?1)", [&Utc::now().to_rfc3339()])?;
        info!("Migration 1 completed");
    }
    
    // Migration 2: Add location columns (v0.2.0 - v0.3.0)
    if current_version < 2 {
        info!("Applying migration 2: Location columns");
        let location_columns = vec![
            "ALTER TABLE locations ADD COLUMN id_sediu TEXT;",
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

        conn.execute("INSERT INTO db_migrations (version, applied_at) VALUES (2, ?1)", [&Utc::now().to_rfc3339()])?;
        info!("Migration 2 completed");
    }

    // Migration 3: Add agent settings columns (v0.3.0)
    if current_version < 3 {
        info!("Applying migration 3: Agent settings columns");
        let agent_columns = vec![
            "ALTER TABLE agent_settings ADD COLUMN cod_carnet TEXT;",
            "ALTER TABLE agent_settings ADD COLUMN cod_carnet_livr TEXT;",
        ];
        
        for sql in agent_columns {
            let _ = conn.execute(sql, []).ok();
        }

        conn.execute("INSERT INTO db_migrations (version, applied_at) VALUES (3, ?1)", [&Utc::now().to_rfc3339()])?;
        info!("Migration 3 completed");
    }

    // Migration 4: Change agent settings cod_carnet columns from INTEGER to TEXT (v0.4.0)
    if current_version < 4 {
        info!("Applying migration 4: Change agent settings cod_carnet columns to TEXT");
        
        // SQLite doesn't support ALTER COLUMN, so we need to recreate the table
        let _ = conn.execute_batch(r#"
            -- Create new table with TEXT columns
            CREATE TABLE IF NOT EXISTS agent_settings_new (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                agent_name TEXT,
                carnet_series TEXT,
                cod_carnet TEXT,
                cod_carnet_livr TEXT,
                updated_at TEXT
            );
            
            -- Copy data, converting INTEGER to TEXT
            INSERT INTO agent_settings_new (id, agent_name, carnet_series, cod_carnet, cod_carnet_livr, updated_at)
            SELECT id, agent_name, carnet_series, CAST(cod_carnet AS TEXT), CAST(cod_carnet_livr AS TEXT), updated_at
            FROM agent_settings;
            
            -- Drop old table
            DROP TABLE agent_settings;
            
            -- Rename new table
            ALTER TABLE agent_settings_new RENAME TO agent_settings;
        "#).ok();

        conn.execute("INSERT INTO db_migrations (version, applied_at) VALUES (4, ?1)", [&Utc::now().to_rfc3339()])?;
        info!("Migration 4 completed");
    }

    // Migration 5: Add simbol_carnet_livr column (v0.4.0)
    if current_version < 5 {
        info!("Applying migration 5: Add simbol_carnet_livr column");
        let _ = conn.execute("ALTER TABLE agent_settings ADD COLUMN simbol_carnet_livr TEXT;", []).ok();
        conn.execute("INSERT INTO db_migrations (version, applied_at) VALUES (5, ?1)", [&Utc::now().to_rfc3339()])?;
        info!("Migration 5 completed");
    }

    // Migration 6: Add simbol_gestiune_livrare column (v0.5.0)
    if current_version < 6 {
        info!("Applying migration 6: Add simbol_gestiune_livrare column");
        let _ = conn.execute("ALTER TABLE agent_settings ADD COLUMN simbol_gestiune_livrare TEXT;", []).ok();
        conn.execute("INSERT INTO db_migrations (version, applied_at) VALUES (6, ?1)", [&Utc::now().to_rfc3339()])?;
        info!("Migration 6 completed");
    }
    
    info!("All migrations completed successfully");
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
