use crate::models::{Location, PartnerWithLocations, Product};
use chrono::Utc;
use rand::Rng;
use std::time::Duration;

/// Simulates fetching partners from an external service
pub async fn fetch_partners() -> Vec<PartnerWithLocations> {
    // Simulate network delay (200-800ms)
    let delay = rand::thread_rng().gen_range(200..800);
    tokio::time::sleep(Duration::from_millis(delay)).await;

    let now = Utc::now().to_rfc3339();

    vec![
        PartnerWithLocations {
            id: "p1".to_string(),
            name: "Acme Corporation SRL".to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
            locations: vec![
                Location {
                    id: "l1".to_string(),
                    partner_id: "p1".to_string(),
                    name: "Sediu Central București".to_string(),
                    address: Some("Str. Victoriei 100, Sector 1".to_string()),
                },
                Location {
                    id: "l2".to_string(),
                    partner_id: "p1".to_string(),
                    name: "Sucursala Cluj".to_string(),
                    address: Some("Str. Napoca 25, Cluj-Napoca".to_string()),
                },
            ],
        },
        PartnerWithLocations {
            id: "p2".to_string(),
            name: "TechStart Solutions SRL".to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
            locations: vec![Location {
                id: "l3".to_string(),
                partner_id: "p2".to_string(),
                name: "Birou Principal".to_string(),
                address: Some("Bd. Unirii 50, București".to_string()),
            }],
        },
        PartnerWithLocations {
            id: "p3".to_string(),
            name: "Global Trade Import-Export SA".to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
            locations: vec![
                Location {
                    id: "l4".to_string(),
                    partner_id: "p3".to_string(),
                    name: "Depozit Constanța".to_string(),
                    address: Some("Zona Industrială, Constanța".to_string()),
                },
                Location {
                    id: "l5".to_string(),
                    partner_id: "p3".to_string(),
                    name: "Showroom Timișoara".to_string(),
                    address: Some("Calea Aradului 15, Timișoara".to_string()),
                },
                Location {
                    id: "l6".to_string(),
                    partner_id: "p3".to_string(),
                    name: "Punct de lucru Iași".to_string(),
                    address: Some("Str. Păcurari 80, Iași".to_string()),
                },
            ],
        },
        PartnerWithLocations {
            id: "p4".to_string(),
            name: "Digital Services & Co SRL".to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
            locations: vec![Location {
                id: "l7".to_string(),
                partner_id: "p4".to_string(),
                name: "Sediu Brașov".to_string(),
                address: Some("Str. Republicii 35, Brașov".to_string()),
            }],
        },
        PartnerWithLocations {
            id: "p5".to_string(),
            name: "Construct Expert SRL".to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
            locations: vec![
                Location {
                    id: "l8".to_string(),
                    partner_id: "p5".to_string(),
                    name: "Sediu Central".to_string(),
                    address: Some("Bd. Decebal 10, Oradea".to_string()),
                },
                Location {
                    id: "l9".to_string(),
                    partner_id: "p5".to_string(),
                    name: "Depozit Materiale".to_string(),
                    address: Some("Zona Industrială Vest, Oradea".to_string()),
                },
            ],
        },
    ]
}

/// Simulates fetching products from an external service
pub async fn fetch_products() -> Vec<Product> {
    // Simulate network delay (200-800ms)
    let delay = rand::thread_rng().gen_range(200..800);
    tokio::time::sleep(Duration::from_millis(delay)).await;

    vec![
        Product {
            id: "pr1".to_string(),
            name: "Laptop Dell XPS 15".to_string(),
            unit_of_measure: "buc".to_string(),
            price: 5500.0,
            class: Some("Electronice".to_string()),
        },
        Product {
            id: "pr2".to_string(),
            name: "Monitor LG 27\" 4K".to_string(),
            unit_of_measure: "buc".to_string(),
            price: 1200.0,
            class: Some("Electronice".to_string()),
        },
        Product {
            id: "pr3".to_string(),
            name: "Cablu USB Type-C 2m".to_string(),
            unit_of_measure: "buc".to_string(),
            price: 35.0,
            class: Some("Accesorii".to_string()),
        },
        Product {
            id: "pr4".to_string(),
            name: "Hârtie A4 (500 coli)".to_string(),
            unit_of_measure: "top".to_string(),
            price: 25.0,
            class: Some("Birou".to_string()),
        },
        Product {
            id: "pr5".to_string(),
            name: "Toner HP 26A".to_string(),
            unit_of_measure: "buc".to_string(),
            price: 350.0,
            class: Some("Birou".to_string()),
        },
        Product {
            id: "pr6".to_string(),
            name: "Tastatură Logitech MX Keys".to_string(),
            unit_of_measure: "buc".to_string(),
            price: 450.0,
            class: Some("Periferice".to_string()),
        },
        Product {
            id: "pr7".to_string(),
            name: "Mouse Logitech MX Master 3".to_string(),
            unit_of_measure: "buc".to_string(),
            price: 380.0,
            class: Some("Periferice".to_string()),
        },
        Product {
            id: "pr8".to_string(),
            name: "SSD Samsung 1TB NVMe".to_string(),
            unit_of_measure: "buc".to_string(),
            price: 420.0,
            class: Some("Componente".to_string()),
        },
        Product {
            id: "pr9".to_string(),
            name: "Webcam Logitech C920".to_string(),
            unit_of_measure: "buc".to_string(),
            price: 320.0,
            class: Some("Periferice".to_string()),
        },
        Product {
            id: "pr10".to_string(),
            name: "Casti Audio-Technica ATH-M50x".to_string(),
            unit_of_measure: "buc".to_string(),
            price: 650.0,
            class: Some("Audio".to_string()),
        },
        Product {
            id: "pr11".to_string(),
            name: "Hub USB-C 7-in-1".to_string(),
            unit_of_measure: "buc".to_string(),
            price: 180.0,
            class: Some("Accesorii".to_string()),
        },
        Product {
            id: "pr12".to_string(),
            name: "Stand Laptop Ajustabil".to_string(),
            unit_of_measure: "buc".to_string(),
            price: 150.0,
            class: Some("Accesorii".to_string()),
        },
    ]
}

/// Simulates sending an invoice to an external service
/// Returns Ok(()) with 50% probability, Err with 50% probability
pub async fn send_invoice_to_external() -> Result<(), String> {
    // Simulate network delay (500-1500ms)
    let delay = rand::thread_rng().gen_range(500..1500);
    tokio::time::sleep(Duration::from_millis(delay)).await;

    // 50% chance of failure
    if rand::random::<bool>() {
        Ok(())
    } else {
        Err("Eroare rețea: Conexiunea a fost refuzată".to_string())
    }
}
