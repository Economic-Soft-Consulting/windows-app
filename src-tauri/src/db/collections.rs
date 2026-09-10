//! Receipts (chitanțe) and their dependency on the invoice they pay.

use rusqlite::Connection;

/// The invoice blocking this receipt group, if it pays a locally created invoice that WME has
/// not accepted yet. `None` means the receipt is safe to send.
///
/// A receipt identifies the invoice it pays **by text** — serie and număr, in
/// `DistribuireValoare`. No local identifier crosses the wire. So if WME has not seen that
/// invoice, it cannot allocate the payment, and rejects the receipt with
/// "nu gasesc in baza de date factura X" — a message the collections screen already parses
/// and shows to the agent, telling them to send the invoice first and retry by hand.
///
/// Matching prefers `collections.invoice_id` (populated from v1.0.12 and backfilled by
/// migration 23) and falls back to the trimmed partner + number + series key for older rows,
/// mirroring `LOCAL_COLLECTED` in [`crate::db::balances`].
///
/// Receipts paid against a WME-origin balance resolve to no local invoice and are never
/// blocked — WME already knows those documents.
pub fn blocking_unsent_invoice(
    conn: &Connection,
    receipt_group_id: &str,
) -> Result<Option<String>, String> {
    conn.query_row(
        r#"
        SELECT TRIM(COALESCE(i.invoice_series, '')) || ' ' || CAST(i.invoice_number AS TEXT)
        FROM collections c
        JOIN invoices i ON (
            (c.invoice_id IS NOT NULL AND i.id = c.invoice_id)
            OR (
                c.invoice_id IS NULL
                AND TRIM(i.partner_id) = TRIM(c.id_partener)
                AND CAST(i.invoice_number AS TEXT) = TRIM(COALESCE(c.numar_factura, ''))
                AND TRIM(COALESCE(i.invoice_series, '')) = TRIM(COALESCE(c.serie_factura, ''))
            )
        )
        WHERE COALESCE(c.receipt_group_id, c.id) = ?1
          AND i.status <> 'sent'
        ORDER BY i.invoice_number
        LIMIT 1
        "#,
        [receipt_group_id],
        |row| row.get::<_, String>(0),
    )
    .map(|label| Some(label.trim().to_string()))
    .or_else(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => Ok(None),
        other => Err(other.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use crate::test_support::{seed_invoice, temp_db};

    /// Inserts a receipt for `partner`, optionally linked to `invoice_id`.
    fn seed_receipt(
        conn: &rusqlite::Connection,
        id: &str,
        partner: &str,
        numar: &str,
        serie: &str,
        invoice_id: Option<&str>,
    ) {
        conn.execute(
            "INSERT INTO collections (id, receipt_group_id, id_partener, numar_factura, \
                                      serie_factura, cod_document, valoare, data_incasare, \
                                      status, created_at, invoice_id) \
             VALUES (?1, ?1, ?2, ?3, ?4, ?3, 85.91, '2026-01-02', 'pending', '2026-01-02', ?5)",
            rusqlite::params![id, partner, numar, serie, invoice_id],
        )
        .unwrap();
    }

    #[test]
    fn blocks_a_receipt_whose_invoice_is_still_pending() {
        let (dir, db) = temp_db("coll_pending");
        let conn = db.conn.lock().unwrap();
        seed_invoice(&conn, "I1", 1, "P1", "FONG");
        conn.execute("UPDATE invoices SET status = 'pending' WHERE id = 'I1'", []).unwrap();
        seed_receipt(&conn, "C1", "P1", "1", "FONG", Some("I1"));

        let blocked = super::blocking_unsent_invoice(&conn, "C1").unwrap();
        assert_eq!(blocked.as_deref(), Some("FONG 1"));

        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn allows_the_receipt_once_the_invoice_is_sent() {
        let (dir, db) = temp_db("coll_sent");
        let conn = db.conn.lock().unwrap();
        seed_invoice(&conn, "I1", 1, "P1", "FONG"); // seeded as 'sent'
        seed_receipt(&conn, "C1", "P1", "1", "FONG", Some("I1"));

        assert_eq!(super::blocking_unsent_invoice(&conn, "C1").unwrap(), None);

        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The decision taken with the user: a receipt whose invoice failed waits rather than
    /// being pushed to WME, which would reject it for a document it does not have.
    #[test]
    fn blocks_a_receipt_whose_invoice_failed() {
        let (dir, db) = temp_db("coll_failed");
        let conn = db.conn.lock().unwrap();
        seed_invoice(&conn, "I1", 1, "P1", "FONG");
        conn.execute("UPDATE invoices SET status = 'failed' WHERE id = 'I1'", []).unwrap();
        seed_receipt(&conn, "C1", "P1", "1", "FONG", Some("I1"));

        assert!(super::blocking_unsent_invoice(&conn, "C1").unwrap().is_some());

        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Receipts written before v1.0.12 have no invoice_id and must still be matched.
    #[test]
    fn matches_legacy_receipts_without_invoice_id() {
        let (dir, db) = temp_db("coll_legacy");
        let conn = db.conn.lock().unwrap();
        seed_invoice(&conn, "I1", 7, "P1", "FONG");
        conn.execute("UPDATE invoices SET status = 'pending' WHERE id = 'I1'", []).unwrap();
        seed_receipt(&conn, "C1", "P1", "7", "FONG", None);

        assert_eq!(
            super::blocking_unsent_invoice(&conn, "C1").unwrap().as_deref(),
            Some("FONG 7")
        );

        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A receipt paid against a balance that came from WME has no local invoice at all. WME
    /// already knows that document, so it must never be held back.
    #[test]
    fn never_blocks_a_receipt_for_a_wme_origin_balance() {
        let (dir, db) = temp_db("coll_wme");
        let conn = db.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO partners (id, name, created_at, updated_at) \
             VALUES ('P9', 'Test', '2026-01-01', '2026-01-01')",
            [],
        )
        .unwrap();
        seed_receipt(&conn, "C9", "P9", "4242", "WME", None);

        assert_eq!(super::blocking_unsent_invoice(&conn, "C9").unwrap(), None);

        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A group pays several invoices; one unsent is enough to hold the whole receipt, because
    /// the receipt is posted to WME as a single transaction.
    #[test]
    fn blocks_the_group_when_any_of_its_invoices_is_unsent() {
        let (dir, db) = temp_db("coll_group");
        let conn = db.conn.lock().unwrap();
        seed_invoice(&conn, "I1", 1, "P1", "FONG");
        seed_invoice(&conn, "I2", 2, "P1", "FONG");
        conn.execute("UPDATE invoices SET status = 'pending' WHERE id = 'I2'", []).unwrap();

        seed_receipt(&conn, "G1", "P1", "1", "FONG", Some("I1"));
        conn.execute(
            "INSERT INTO collections (id, receipt_group_id, id_partener, numar_factura, \
                                      serie_factura, cod_document, valoare, data_incasare, \
                                      status, created_at, invoice_id) \
             VALUES ('G1b', 'G1', 'P1', '2', 'FONG', '2', 10.0, '2026-01-02', 'pending', \
                     '2026-01-02', 'I2')",
            [],
        )
        .unwrap();

        assert_eq!(
            super::blocking_unsent_invoice(&conn, "G1").unwrap().as_deref(),
            Some("FONG 2")
        );

        drop(conn);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
