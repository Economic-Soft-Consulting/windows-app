use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partner {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub partner_id: String,
    pub name: String,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartnerWithLocations {
    pub id: String,
    pub name: String,
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
    pub location_id: String,
    pub location_name: String,
    pub status: InvoiceStatus,
    pub total_amount: f64,
    pub item_count: i32,
    pub notes: Option<String>,
    pub created_at: String,
    pub sent_at: Option<String>,
    pub error_message: Option<String>,
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
