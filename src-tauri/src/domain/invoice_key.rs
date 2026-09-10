//! Document identity.
//!
//! A receipt is matched to the invoice it pays by comparing partner, series, number and
//! document code as strings. Those four values are written by several code paths, so they
//! must be normalised in exactly one way — divergent normalisation is what produced the
//! ghost invoices fixed in v1.0.12.

pub fn normalize_opt_key(value: &Option<String>) -> String {
    value
        .as_ref()
        .map(|v| v.trim().to_string())
        .unwrap_or_default()
}

pub fn build_invoice_key(id_partener: &str, serie_factura: &Option<String>, numar_factura: &Option<String>, cod_document: &Option<String>) -> String {
    format!(
        "{}|{}|{}|{}",
        id_partener.trim(),
        normalize_opt_key(serie_factura),
        normalize_opt_key(numar_factura),
        normalize_opt_key(cod_document)
    )
}
