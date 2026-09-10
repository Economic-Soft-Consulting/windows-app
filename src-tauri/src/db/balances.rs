//! Client balances (solduri): the WME-synced snapshot and the query that reconciles it
//! against locally created invoices and the receipts collected so far.

use log::info;
use rusqlite::{params, Connection};

use crate::api_client;

/// Replaces the whole `client_balances` snapshot in one transaction.
///
/// Previously this ran `DELETE` and then inserted row by row with no transaction, despite a
/// comment claiming otherwise. Two things went wrong:
///
/// * Every insert was its own implicit transaction, so each one paid an fsync while the
///   global connection mutex was held — every other command in the app blocked behind it.
///   Opening Setări waits on `get_agent_settings`, which is why that screen took seconds.
/// * `client_balances` has UNIQUE(id_partener, cod_document, serie, numar). WME does return
///   the same document twice — the collections screen already de-duplicates on that key — so
///   a duplicate aborted the loop *after* the DELETE had committed, leaving the agent with a
///   truncated balance list and nothing on screen to say so.
///
/// Wrapped in a transaction, a mid-loop failure now rolls back to the previous snapshot.
pub fn replace_client_balances(
    conn: &Connection,
    solduri: &[api_client::SoldInfo],
    now: &str,
) -> Result<usize, String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    tx.execute("DELETE FROM client_balances", [])
        .map_err(|e| e.to_string())?;

    let mut inserted = 0usize;
    {
        let mut stmt = tx
            .prepare(
                "INSERT INTO client_balances (
                    id_partener, cod_fiscal, denumire, tip_document, cod_document,
                    serie, numar, data, valoare, rest, termen, moneda,
                    sediu, id_sediu, curs, observatii, cod_obligatie, marca_agent, synced_at
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19
                )",
            )
            .map_err(|e| e.to_string())?;

        for s in solduri {
            stmt.execute(params![
                s.id_partener.as_ref().map(|v| v.trim().to_string()),
                s.cod_fiscal,
                s.denumire,
                s.tip_document,
                s.cod_document,
                s.serie,
                s.numar,
                s.data,
                api_client::parse_f64(&s.valoare),
                api_client::parse_f64(&s.rest),
                s.termen,
                s.moneda,
                s.sediu,
                s.id_sediu,
                api_client::parse_f64(&s.curs),
                s.observatii,
                s.cod_obligatie,
                s.marca_agent,
                now
            ])
            .map_err(|e| e.to_string())?;
            inserted += 1;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;
    info!("[SOLDURI] replaced snapshot with {} rows", inserted);
    Ok(inserted)
}

/// Builds the client-balances query: WME-synced balances UNION locally created invoices,
/// each with the amount already collected subtracted.
///
/// Extracted so the SQL can be parsed in a test — it is assembled by text substitution, and a
/// malformed result would only surface as a runtime error on the collections screen.
pub fn build_client_balances_query() -> String {
    "SELECT
        q.id, q.id_partener, q.cod_fiscal, q.denumire, q.tip_document, q.cod_document,
        q.serie, q.numar, q.data, q.valoare, q.rest, q.termen, q.moneda,
        q.sediu, q.id_sediu, q.curs, q.observatii, q.cod_obligatie, q.marca_agent, q.synced_at
        FROM (
            -- WME invoices (from client_balances), excluding ones we created locally
            SELECT
                cb.id, cb.id_partener, cb.cod_fiscal, cb.denumire, cb.tip_document, cb.cod_document,
                cb.serie, cb.numar, cb.data, cb.valoare,
                CASE
                    WHEN ROUND(COALESCE(cb.rest, 0) - WME_COLLECTED, 2) > 0
                        THEN ROUND(COALESCE(cb.rest, 0) - WME_COLLECTED, 2)
                    ELSE 0
                END AS rest,
                cb.termen, cb.moneda,
                cb.sediu, cb.id_sediu, cb.curs, cb.observatii, cb.cod_obligatie, cb.marca_agent, cb.synced_at
            FROM client_balances cb
            -- Exclude invoices that exist locally — those are handled by the invoices branch
            WHERE NOT EXISTS (
                SELECT 1 FROM invoices i_local
                WHERE TRIM(i_local.partner_id) = TRIM(cb.id_partener)
                  AND i_local.invoice_number = CAST(COALESCE(cb.numar, '0') AS INTEGER)
                  AND (
                      TRIM(COALESCE(i_local.invoice_series, '')) = TRIM(COALESCE(cb.serie, ''))
                      OR trim(COALESCE(i_local.invoice_series, '')) = ''
                      OR trim(COALESCE(cb.serie, '')) = ''
                  )
            )
            -- Exclude invoices hidden by the agent (phantom test invoices).
            -- hide_client_balance trims before storing, so match trimmed on both sides or
            -- hiding a padded row silently does nothing.
            AND NOT EXISTS (
                SELECT 1 FROM ignored_balances ib
                WHERE TRIM(ib.id_partener) = TRIM(cb.id_partener)
                  AND TRIM(ib.cod_document) = TRIM(COALESCE(cb.cod_document, ''))
                  AND TRIM(ib.serie) = TRIM(COALESCE(cb.serie, ''))
                  AND TRIM(ib.numar) = TRIM(COALESCE(cb.numar, ''))
            )

            UNION ALL

            -- Local invoices (created in this app) — always use local collection data
            SELECT
                NULL AS id,
                i.partner_id AS id_partener,
                p.cif AS cod_fiscal,
                p.name AS denumire,
                'FACTURA' AS tip_document,
                CAST(i.invoice_number AS TEXT) AS cod_document,
                i.invoice_series AS serie,
                CAST(i.invoice_number AS TEXT) AS numar,
                strftime('%d/%m/%Y', replace(substr(i.created_at, 1, 19), 'T', ' ')) AS data,
                LOCAL_GROSS_TOTAL AS valoare,
                CASE
                    WHEN ROUND(LOCAL_GROSS_TOTAL - LOCAL_COLLECTED, 2) > 0
                        THEN ROUND(LOCAL_GROSS_TOTAL - LOCAL_COLLECTED, 2)
                    ELSE 0
                END AS rest,
                strftime(
                    '%d/%m/%Y',
                    datetime(
                        replace(substr(i.created_at, 1, 19), 'T', ' '),
                        '+' || COALESCE(NULLIF(trim(p.scadenta_la_vanzare), ''), '30') || ' days'
                    )
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
            WHERE i.status IN ('pending', 'sending', 'sent', 'failed')
            -- Exclude invoices hidden by the agent. This filter used to exist only on the
            -- WME branch, so hiding a locally created ghost did nothing: the card was
            -- dropped optimistically in the UI and came back on the next load.
            AND NOT EXISTS (
                SELECT 1 FROM ignored_balances ib
                WHERE TRIM(ib.id_partener) = TRIM(i.partner_id)
                  AND TRIM(ib.numar) = CAST(i.invoice_number AS TEXT)
            )
        ) q
        WHERE COALESCE(q.rest, 0) > 0"
        .replace("LOCAL_GROSS_TOTAL", LOCAL_GROSS_TOTAL)
        .replace("LOCAL_COLLECTED", LOCAL_COLLECTED)
        .replace("WME_COLLECTED", WME_COLLECTED)
}

/// The VAT-inclusive total of a locally created invoice, always at 2 decimals.
/// Prefers the value stored at creation time; falls back to recomputing it per line
/// (rounding each line, as WME does) for rows the migration could not backfill.
const LOCAL_GROSS_TOTAL: &str = "COALESCE(i.total_amount_gross, (
                    SELECT ROUND(COALESCE(SUM(ROUND(ii.total_price * (1.0 + COALESCE(CAST(pg.procent_tva AS REAL), 0) / 100.0), 2)), 0), 2)
                    FROM invoice_items ii
                    JOIN products pg ON pg.id = ii.product_id
                    WHERE ii.invoice_id = i.id
                ))";

/// Amount already collected against a WME balance row, matched on the 4-part document key.
/// Every side is trimmed: the three tables are written by different code paths that did not
/// agree on whitespace, and a single stray space left the invoice looking unpaid.
const WME_COLLECTED: &str = "(
                        SELECT COALESCE(SUM(c.valoare), 0)
                        FROM collections c
                        WHERE TRIM(c.id_partener) = TRIM(cb.id_partener)
                          AND TRIM(COALESCE(c.serie_factura, '')) = TRIM(COALESCE(cb.serie, ''))
                          AND TRIM(COALESCE(c.numar_factura, '')) = TRIM(COALESCE(cb.numar, ''))
                          AND TRIM(COALESCE(c.cod_document, '')) = TRIM(COALESCE(cb.cod_document, ''))
                          AND c.status IN ('pending', 'sending', 'synced')
                    )";

/// Amount already collected against a locally created invoice.
///
/// Prefers the direct invoice_id link written since v1.0.11; older receipts fall back to
/// matching partner + number + series as strings. A correlated subquery rather than a
/// GROUP BY join, so an invoice can never be duplicated by matching two receipt groups.
const LOCAL_COLLECTED: &str = "COALESCE((
                        SELECT SUM(c.valoare)
                        FROM collections c
                        WHERE c.status IN ('pending', 'sending', 'synced')
                          AND (
                            c.invoice_id = i.id
                            OR (
                                c.invoice_id IS NULL
                                AND TRIM(c.id_partener) = TRIM(i.partner_id)
                                AND TRIM(COALESCE(c.numar_factura, '')) = CAST(i.invoice_number AS TEXT)
                                AND TRIM(COALESCE(c.serie_factura, '')) = TRIM(COALESCE(i.invoice_series, ''))
                            )
                          )
                    ), 0)";

#[cfg(test)]
mod tests {
    use crate::test_support::{balance_rest, seed_invoice, temp_db};

    /// The balance query is assembled by text substitution, so a malformed result would
    /// only show up as a runtime error on the collections screen. Parse it against the
    /// real schema instead.
    #[test]
    fn client_balances_query_is_valid_sql() {
        let (dir, db) = temp_db("sql");
        let conn = db.conn.lock().unwrap();

        let mut query = super::build_client_balances_query();
        query.push_str(" AND TRIM(q.id_partener) = TRIM(?1)");
        query.push_str(" ORDER BY date(q.termen) ASC");

        let mut stmt = conn.prepare(&query).expect("balance query must be valid SQL");
        // 20 columns are read back by get_client_balances.
        assert_eq!(stmt.column_count(), 20);

        let rows: Vec<i64> = stmt
            .query_map(["x"], |_| Ok(1))
            .expect("query must execute")
            .filter_map(|r| r.ok())
            .collect();
        assert!(rows.is_empty());

        drop(stmt);
        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }
    /// A fully paid invoice must leave the balance list.
    #[test]
    fn paid_invoice_disappears_from_balances() {
        let (dir, db) = temp_db("paid");
        let conn = db.conn.lock().unwrap();
        seed_invoice(&conn, "I1", 1, "P1", "FONG");

        assert_eq!(balance_rest(&conn, "P1"), vec![85.91], "unpaid invoice must be listed");

        conn.execute(
            "INSERT INTO collections (id, id_partener, numar_factura, serie_factura, cod_document, valoare, data_incasare, status, created_at, invoice_id)
             VALUES ('C1', 'P1', '1', 'FONG', '1', 85.91, '2026-01-02', 'pending', '2026-01-02', 'I1')",
            [],
        ).unwrap();

        assert!(balance_rest(&conn, "P1").is_empty(), "paid invoice must disappear");
        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }
    /// Ghost #2: partner ids written with different whitespace by different code paths.
    /// The invoice_id link, and TRIM on the fallback, must both survive it.
    #[test]
    fn receipt_still_matches_when_partner_id_is_padded() {
        let (dir, db) = temp_db("pad");
        let conn = db.conn.lock().unwrap();
        seed_invoice(&conn, "I1", 1, "P1 ", "FONG");

        // Receipt written by a path that trims, invoice by one that does not.
        conn.execute(
            "INSERT INTO collections (id, id_partener, numar_factura, serie_factura, cod_document, valoare, data_incasare, status, created_at, invoice_id)
             VALUES ('C1', 'P1', '1', 'FONG', '1', 85.91, '2026-01-02', 'pending', '2026-01-02', NULL)",
            [],
        ).unwrap();

        assert!(balance_rest(&conn, "P1").is_empty(), "padded partner id must not create a ghost");
        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }
    /// Ghost #5: the receipt carries a different series than the invoice (the agent changed
    /// the carnet in Settings). The invoice_id link must still match it.
    #[test]
    fn receipt_matches_invoice_despite_series_drift() {
        let (dir, db) = temp_db("series");
        let conn = db.conn.lock().unwrap();
        seed_invoice(&conn, "I1", 1, "P1", "FONG");

        conn.execute(
            "INSERT INTO collections (id, id_partener, numar_factura, serie_factura, cod_document, valoare, data_incasare, status, created_at, invoice_id)
             VALUES ('C1', 'P1', '1', 'ALTASERIE', '1', 85.91, '2026-01-02', 'synced', '2026-01-02', 'I1')",
            [],
        ).unwrap();

        assert!(balance_rest(&conn, "P1").is_empty(), "invoice_id link must survive a series change");
        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }
    /// Hiding a locally created invoice must stick. The ignored_balances filter used to
    /// exist only on the WME branch, so the card came back on the next load.
    #[test]
    fn hiding_a_local_invoice_persists() {
        let (dir, db) = temp_db("hide");
        let conn = db.conn.lock().unwrap();
        seed_invoice(&conn, "I1", 7, "P1", "FONG");

        assert_eq!(balance_rest(&conn, "P1").len(), 1);

        conn.execute(
            "INSERT INTO ignored_balances (id_partener, cod_document, serie, numar) VALUES ('P1', '7', 'FONG', '7')",
            [],
        ).unwrap();

        assert!(balance_rest(&conn, "P1").is_empty(), "hidden local invoice must stay hidden");
        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }
    /// A partly paid invoice keeps only the remainder, at 2 decimals.
    #[test]
    fn partial_payment_leaves_rounded_remainder() {
        let (dir, db) = temp_db("partial");
        let conn = db.conn.lock().unwrap();
        seed_invoice(&conn, "I1", 1, "P1", "FONG");

        conn.execute(
            "INSERT INTO collections (id, id_partener, numar_factura, serie_factura, cod_document, valoare, data_incasare, status, created_at, invoice_id)
             VALUES ('C1', 'P1', '1', 'FONG', '1', 50.0, '2026-01-02', 'pending', '2026-01-02', 'I1')",
            [],
        ).unwrap();

        assert_eq!(balance_rest(&conn, "P1"), vec![35.91]);
        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn sold(partner: &str, cod: &str) -> crate::api_client::SoldInfo {
        serde_json::from_value(serde_json::json!({
            "IDPartener": partner, "CodDocument": cod, "Serie": "FONG", "Numar": "1",
            "Valoare": "100", "Rest": "100"
        }))
        .expect("SoldInfo fixture")
    }

    /// The rebuild used to run DELETE and then insert row by row with no transaction, so a
    /// duplicate key aborted the loop *after* the DELETE had committed and the agent's
    /// solduri simply vanished. client_balances has UNIQUE(id_partener, cod_document,
    /// serie, numar), and WME does return the same document twice.
    #[test]
    fn a_duplicate_row_leaves_the_previous_snapshot_intact() {
        let (dir, db) = temp_db("balances_tx");
        let conn = db.conn.lock().unwrap();

        super::replace_client_balances(&conn, &[sold("P1", "A"), sold("P1", "B")], "2026-01-01")
            .expect("seed snapshot");
        let before: i64 = conn
            .query_row("SELECT COUNT(*) FROM client_balances", [], |r| r.get(0))
            .unwrap();
        assert_eq!(before, 2);

        // Second and third entries collide on the unique key.
        let batch = [sold("P2", "X"), sold("P2", "Y"), sold("P2", "Y")];
        let result = super::replace_client_balances(&conn, &batch, "2026-01-02");

        assert!(result.is_err(), "a duplicate key must fail the whole rebuild");

        // Assert the surviving rows are the ORIGINAL ones, not a partial rebuild. Counting
        // alone is not enough: a partial rebuild can leave the same number of rows while
        // having replaced every one of them.
        let mut stmt = conn
            .prepare("SELECT id_partener, cod_document FROM client_balances ORDER BY cod_document")
            .unwrap();
        let rows: Vec<(String, String)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        drop(stmt);

        assert_eq!(
            rows,
            vec![("P1".to_string(), "A".to_string()), ("P1".to_string(), "B".to_string())],
            "the previous snapshot must survive a failed rebuild untouched"
        );

        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_clean_batch_replaces_the_whole_snapshot() {
        let (dir, db) = temp_db("balances_replace");
        let conn = db.conn.lock().unwrap();

        super::replace_client_balances(&conn, &[sold("P1", "A")], "2026-01-01").unwrap();
        let n = super::replace_client_balances(&conn, &[sold("P2", "B"), sold("P2", "C")], "2026-01-02")
            .expect("clean rebuild");

        assert_eq!(n, 2);
        let partners: Vec<String> = conn
            .prepare("SELECT DISTINCT id_partener FROM client_balances")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert_eq!(partners, vec!["P2".to_string()], "old rows must be gone");

        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
