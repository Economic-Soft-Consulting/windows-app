use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Partner {
    pub id: String,
    pub name: String,
    pub cif: Option<String>,
    pub reg_com: Option<String>,
    pub cod: Option<String>,
    pub blocat: Option<String>,
    pub tva_la_incasare: Option<String>,
    pub persoana_fizica: Option<String>,
    pub cod_extern: Option<String>,
    pub cod_intern: Option<String>,
    pub observatii: Option<String>,
    pub data_adaugarii: Option<String>,
    pub clasa: Option<String>,
    pub simbol_clasa: Option<String>,
    pub cod_clasa: Option<String>,
    pub inactiv: Option<String>,
    pub categorie_pret_implicita: Option<String>,
    pub simbol_categorie_pret: Option<String>,
    pub scadenta_la_vanzare: Option<String>,
    pub scadenta_la_cumparare: Option<String>,
    pub credit_client: Option<String>,
    pub discount_fix: Option<String>,
    pub tip_partener: Option<String>,
    pub mod_aplicare_discount: Option<String>,
    pub moneda: Option<String>,
    pub data_nastere: Option<String>,
    pub caracterizare_contabila_denumire: Option<String>,
    pub caracterizare_contabila_simbol: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub partner_id: String,
    pub name: String,
    pub address: Option<String>,
    pub cod_sediu: Option<String>,
    pub localitate: Option<String>,
    pub strada: Option<String>,
    pub numar: Option<String>,
    pub judet: Option<String>,
    pub tara: Option<String>,
    pub cod_postal: Option<String>,
    pub telefon: Option<String>,
    pub email: Option<String>,
    pub inactiv: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartnerWithLocations {
    pub id: String,
    pub name: String,
    pub cif: Option<String>,
    pub reg_com: Option<String>,
    pub cod: Option<String>,
    pub blocat: Option<String>,
    pub tva_la_incasare: Option<String>,
    pub persoana_fizica: Option<String>,
    pub cod_extern: Option<String>,
    pub cod_intern: Option<String>,
    pub observatii: Option<String>,
    pub data_adaugarii: Option<String>,
    pub clasa: Option<String>,
    pub simbol_clasa: Option<String>,
    pub cod_clasa: Option<String>,
    pub inactiv: Option<String>,
    pub categorie_pret_implicita: Option<String>,
    pub simbol_categorie_pret: Option<String>,
    pub scadenta_la_vanzare: Option<String>,
    pub scadenta_la_cumparare: Option<String>,
    pub credit_client: Option<String>,
    pub discount_fix: Option<String>,
    pub tip_partener: Option<String>,
    pub mod_aplicare_discount: Option<String>,
    pub moneda: Option<String>,
    pub data_nastere: Option<String>,
    pub caracterizare_contabila_denumire: Option<String>,
    pub caracterizare_contabila_simbol: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub locations: Vec<Location>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub unit_of_measure: String,
    pub price: f64,
    pub class: Option<String>,
    pub tva_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InvoiceStatus {
    Pending,
    Sending,
    Sent,
    Failed,
}

impl ToString for InvoiceStatus {
    fn to_string(&self) -> String {
        match self {
            InvoiceStatus::Pending => "pending".to_string(),
            InvoiceStatus::Sending => "sending".to_string(),
            InvoiceStatus::Sent => "sent".to_string(),
            InvoiceStatus::Failed => "failed".to_string(),
        }
    }
}

impl From<String> for InvoiceStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "pending" => InvoiceStatus::Pending,
            "sending" => InvoiceStatus::Sending,
            "sent" => InvoiceStatus::Sent,
            "failed" => InvoiceStatus::Failed,
            _ => InvoiceStatus::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub partner_id: String,
    pub partner_name: String,
    pub partner_cif: Option<String>,
    pub partner_reg_com: Option<String>,
    pub location_id: String,
    pub location_name: String,
    pub location_address: Option<String>,
    pub status: InvoiceStatus,
    pub total_amount: f64,
    pub item_count: i32,
    pub notes: Option<String>,
    pub created_at: String,
    pub sent_at: Option<String>,
    pub error_message: Option<String>,
    pub partner_payment_term: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceItem {
    pub id: String,
    pub invoice_id: String,
    pub product_id: String,
    pub product_name: String,
    pub quantity: f64,
    pub unit_price: f64,
    pub unit_of_measure: String,
    pub total_price: f64,
    pub tva_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvoiceRequest {
    pub partner_id: String,
    pub location_id: String,
    pub notes: Option<String>,
    pub items: Vec<CreateInvoiceItemRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvoiceItemRequest {
    pub product_id: String,
    pub quantity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceDetail {
    pub invoice: Invoice,
    pub items: Vec<InvoiceItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub is_first_run: bool,
    pub partners_synced_at: Option<String>,
    pub products_synced_at: Option<String>,
    pub is_syncing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSettings {
    pub agent_name: Option<String>,
    pub carnet_series: Option<String>,
    pub simbol_carnet_livr: Option<String>,
    pub simbol_gestiune_livrare: Option<String>,
    pub cod_carnet: Option<String>,
    pub cod_carnet_livr: Option<String>,
    pub delegate_name: Option<String>,
    pub delegate_act: Option<String>,    pub car_number: Option<String>,    pub invoice_number_start: Option<i32>,
    pub invoice_number_end: Option<i32>,
    pub invoice_number_current: Option<i32>,
}
