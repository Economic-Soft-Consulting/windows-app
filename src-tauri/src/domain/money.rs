//! Money arithmetic.
//!
//! WME keeps money at 2 decimals. Anything stored locally, compared against a stored value,
//! or sent to the ERP must agree with that, or receipts cannot be reconciled against the
//! invoices they pay. This module is the only place that decides how money is rounded.

/// Rounds a monetary amount to 2 decimals (bani).
///
/// Every value that is stored, compared against a stored value, or sent to WME must go
/// through this. WME keeps money at 2 decimals, so an unrounded f64 such as
/// 78.82 * 1.09 = 85.9138 cannot be reconciled against the invoice it pays.
pub fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// VAT-inclusive total of one invoice line, rounded the way WME rounds it: per line.
///
/// Summing unrounded line grosses and rounding once at the end drifts from WME by a few
/// bani on multi-line invoices, which is enough to leave a phantom balance.
pub fn line_gross(total_price: f64, procent_tva: f64) -> f64 {
    round2(total_price * (1.0 + procent_tva / 100.0))
}

/// VAT-inclusive total of one invoice, at 2 decimals — the single source of truth for
/// "how much does this invoice cost", used by the balance list, the receipt validation and
/// the remaining-amount lookup so the three can never disagree.
///
/// Prefers the value stored at creation time; recomputes per line only when it is missing.
pub fn invoice_gross_total(conn: &rusqlite::Connection, invoice_id: &str) -> Result<f64, String> {
    let stored: Option<f64> = conn
        .query_row(
            "SELECT total_amount_gross FROM invoices WHERE id = ?1",
            [invoice_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Factura nu a fost găsită: {}", e))?;

    if let Some(value) = stored {
        return Ok(round2(value));
    }

    let mut stmt = conn
        .prepare(
            "SELECT ii.total_price, p.procent_tva \
             FROM invoice_items ii \
             JOIN products p ON ii.product_id = p.id \
             WHERE ii.invoice_id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let total: f64 = stmt
        .query_map([invoice_id], |row| {
            let price: f64 = row.get(0)?;
            let tva_str: Option<String> = row.get(1)?;
            let tva_percent = tva_str
                .and_then(|s| s.trim().parse::<f64>().ok())
                .unwrap_or(0.0);
            Ok(line_gross(price, tva_percent))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .sum();

    Ok(round2(total))
}

#[cfg(test)]
mod tests {
    use super::{line_gross, round2};

    #[test]
    fn round2_rounds_to_bani() {
        assert_eq!(round2(85.9138), 85.91);
        assert_eq!(round2(85.915), 85.92);
        assert_eq!(round2(85.91000000000001), 85.91);
        assert_eq!(round2(0.0), 0.0);
    }

    /// The reported defect: 78.82 net at 9% VAT produced 85.9138, which reached WME as
    /// "85,914" while WME had rounded the invoice itself to 85,91.
    #[test]
    fn line_gross_matches_wme_rounding() {
        assert_eq!(line_gross(78.82, 9.0), 85.91);
        assert_ne!(line_gross(78.82, 9.0), 85.9138);
    }

    #[test]
    fn line_gross_handles_zero_and_standard_rates() {
        assert_eq!(line_gross(100.0, 0.0), 100.0);
        assert_eq!(line_gross(100.0, 19.0), 119.0);
        assert_eq!(line_gross(33.33, 19.0), 39.66);
    }

    /// Rounding per line then summing is what WME does; summing raw then rounding once
    /// drifts on multi-line invoices and leaves a phantom balance of a few bani.
    #[test]
    fn per_line_rounding_is_used_for_invoice_totals() {
        // Every line's gross lands just under half a ban, so each one rounds down on its
        // own. Summing the raw products first instead rounds up, overstating the invoice by
        // a ban — enough to leave a phantom balance that never clears.
        let lines = [(0.05, 9.0), (0.05, 9.0), (0.05, 9.0)];
        let per_line: f64 = round2(lines.iter().map(|(p, t)| line_gross(*p, *t)).sum::<f64>());
        let naive: f64 = round2(lines.iter().map(|(p, t)| p * (1.0 + t / 100.0)).sum::<f64>());

        assert_eq!(per_line, 0.15);
        assert_eq!(naive, 0.16);
    }
}
