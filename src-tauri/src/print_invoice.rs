use crate::models::{Invoice, InvoiceItem};

pub struct CompanyInfo {
    pub name: &'static str,
    pub cif: &'static str,
    pub reg_com: &'static str,
    pub address: &'static str,
    pub localitate: &'static str,
    pub cod_postal: &'static str,
    pub bank_name: &'static str,
    pub bank_account: &'static str,
    pub capital: &'static str,
}

// KARIN company details
pub const KARIN: CompanyInfo = CompanyInfo {
    name: "KARIN SRL",
    cif: "RO5379259",
    reg_com: "J24/380/1994",
    address: "Str. Nicolae Balcescu 43",
    localitate: "Seini, Jud. Maramures",
    cod_postal: "435500",
    bank_name: "Banca Transilvania",
    bank_account: "RO03BTRL02501202L70970XX",
    capital: "200020 RON",
};

pub fn generate_invoice_html(
    invoice: &Invoice,
    items: &[InvoiceItem],
    invoice_number: i64,
    logo_base64: Option<&str>,
    payment_term_days: i64,
    delegate_name: Option<&str>,
    delegate_act: Option<&str>,
    car_number: Option<&str>,
    carnet_series: &str,
) -> String {
    log::info!("📄 Generating invoice HTML with payment_term_days: {} for partner: '{}'", 
        payment_term_days, invoice.partner_name);
    
    let due_date = calculate_due_date(&invoice.created_at, payment_term_days);
    log::info!("📄 Calculated due date: {} (created: {}, +{} days)", 
        due_date, invoice.created_at, payment_term_days);
    
    // Calculate total TVA by summing individual product TVAs
    let mut total_without_vat = 0.0;
    let mut total_vat = 0.0;
    
    let products_html = items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            // Use product's TVA or default to 19%
            let vat_rate = item.tva_percent.unwrap_or(19.0) / 100.0;
            
            // Calculate TVA as percentage of price (prices are without VAT)
            let item_vat = (item.total_price * vat_rate * 100.0).round() / 100.0;
            
            total_without_vat += item.total_price;
            total_vat += item_vat;
            
            let tva_display = item.tva_percent
                .map(|t| format!("TVA: {:.0}%", t))
                .unwrap_or_else(|| "TVA: 19%".to_string());
            
            format!(
                r#"        <div class="product-item">
            <span class="prod-name">{}. {}</span>
            <div class="prod-math">
                <span>{} {} x {:.2}</span>
                <span>= {:.2}</span>
            </div>
            <div class="prod-vat-row">
                <span class="tva-percent">{}</span>
                <span class="tva-value">Valoare TVA: {:.2} RON</span>
            </div>
        </div>"#,
                idx + 1,
                item.product_name,
                item.quantity as i32,
                item.unit_of_measure,
                item.unit_price,
                item.total_price,
                tva_display,
                item_vat
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<!DOCTYPE html>
<html lang="ro">
<head>
    <meta charset="UTF-8">
    <title>Factura KARIN</title>
    <style>
        @media print {{
            @page {{
                size: 80mm 297mm;
                margin: 3mm 6mm 3mm 0.5mm;
            }}
            body {{ 
                margin: 0; 
                padding: 0; 
            }}
            header, footer {{ 
                display: none; 
            }}
        }}

        html {{
            height: 100%;
        }}

        body {{
            font-family: Arial, Helvetica, sans-serif;
            width: 68mm;
            margin: 0 auto;
            padding: 2mm;
            font-size: 10.5px;
            font-weight: bold;
            color: #000000;
            line-height: 1.15;
           background: white;
            box-sizing: border-box;
        }}

        h1 {{
            font-size: 18px;
            text-align: center;
            margin: 0 0 5px 0;
            border-bottom: 2px solid #000;
            text-transform: uppercase;
        }}

        .header-meta {{
            text-align: center;
            font-size: 14px;
            margin-bottom: 10px;
            border-bottom: 1px dashed #000;
            padding-bottom: 5px;
        }}

        .section {{
            margin-bottom: 8px;
            border-bottom: 1px dashed #000;
            padding-bottom: 5px;
            word-wrap: break-word;
        }}

        .section-title {{
            text-decoration: underline;
            font-size: 14px;
            display: block;
            margin-bottom: 2px;
        }}

        .compact-row {{
            margin-top: 4px;
            display: block;
        }}

        .row {{
            display: flex;
            justify-content: space-between;
        }}

        .products-container {{
            border-top: 2px solid #000;
            margin-top: 5px;
        }}

        .product-item {{
            border-bottom: 1px dotted #000;
            padding: 4px 0;
        }}

        .prod-name {{
            display: block;
            font-size: 13px;
            margin-bottom: 2px;
        }}

        .prod-math {{
            display: flex;
            justify-content: space-between;
            font-size: 13px;
        }}
        
        .prod-vat-row {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            font-size: 12px;
            font-weight: bold;
            color: #000;
            margin-top: 2px;
        }}
        
        .tva-percent {{
            font-weight: bold;
        }}
        
        .tva-value {{
            font-weight: bold;
        }}

        .totals-section {{
            text-align: right;
            margin-top: 10px;
            font-size: 14px;
        }}

        .grand-total {{
            font-size: 18px;
            border-top: 2px solid #000;
            border-bottom: 2px solid #000;
            margin-top: 5px;
            padding: 5px 0;
        }}

        .legal-note {{
            font-size: 11px;
            text-align: justify;
            margin-top: 10px;
            font-weight: normal;
            color: #000;
        }}

        .signatures {{
            margin-top: 15px;
        }}

        .sig-block {{
            margin-bottom: 15px;
            page-break-inside: avoid;
        }}

        .dots {{
            border-bottom: 1px dotted #000;
            display: inline-block;
            width: 100%;
            height: 15px;
        }}

        .footer-branding {{
            width: 100%;
            text-align: center;
            font-size: 20px;
            color: #000000;
            font-weight: normal;
            font-style: italic;
            border-top: 1px solid #000;
            padding-top: 5px;
            margin-top: 15px;
            background-color: white;
        }}

        .footer-logo {{
            width: 45mm;
            height: auto;
            display: block;
            margin: 0 auto 5px auto;
        }}
    </style>
</head>
<body>

    <h1>FACTURA FISCALA</h1>
    
    <div class="header-meta">
        Seria: {} &nbsp; Nr: {}<br>
        Data emitere: {}<br>
        Data scadenta: {}
    </div>

    <div class="section">
        <span class="section-title">FURNIZOR:</span>
        {}<br>
        CIF: {}<br>
        Reg.Com: {}<br>
        Capital Social: {}<br>
        Localitate: {}<br>
        Sediul: {}<br>
        Cod Postal: {}<br>
        <div class="compact-row">
            Banca: {}<br>
            Cont: {}
        </div>
    </div>

    <div class="section">
        <span class="section-title">CUMPARATOR:</span>
        {}<br>
        CIF: {}<br>
        Reg.Com: {}<br>
        Locatie: {}<br>
        {}
    </div>

    <div class="products-container">
        {}
    </div>

    <div class="totals-section">
        <div class="row">
            <span>Total Valoare:</span>
            <span>{:.2} RON</span>
        </div>
        <div class="row">
            <span>Total TVA:</span>
            <span>{:.2} RON</span>
        </div>
        
        <div class="grand-total">
            TOTAL GENERAL: {:.2} RON
        </div>
    </div>

    <div class="legal-note">
        Produsele din prezenta factura raman proprietatea firmei noastre pana la achitarea lor integrala.<br>
        Prezenta tine loc de contract ferm intre parti in lipsa altui acord scris.<br>
        <strong>Data Scadenta: {}</strong>
    </div>
    {}
    <div class="signatures">
        
        <div class="sig-block">
            Semnatura si stampila Furnizor:<br>
            <span class="dots"></span>
        </div>

        <div class="sig-block">
            Numele Delegatului: {}<br>
            Act Delegat: {}<br>
            Semnatura: <span class="dots" style="width: 50%;"></span>
        </div>

        <div class="sig-block">
            Semnatura de primire:<br>
            <span class="dots"></span>
        </div>

    </div>

    <div class="footer-branding">
        {}
        printed by eSOFT app
    </div>

    <script>
        // Auto-open print dialog when page loads
        function triggerPrint() {{
            window.print();
        }}
        
        // Try multiple triggers to ensure print dialog opens
        if (document.readyState === 'loading') {{
            document.addEventListener('DOMContentLoaded', function() {{
                setTimeout(triggerPrint, 300);
            }});
        }} else {{
            triggerPrint();
        }}
        
        // Fallback after window load
        window.addEventListener('load', function() {{
            setTimeout(triggerPrint, 100);
        }});
    </script>
</body>
</html>"#,
        carnet_series,
        invoice_number,
        format_date(&invoice.created_at),
        due_date.clone(),
        KARIN.name,
        KARIN.cif,
        KARIN.reg_com,
        KARIN.capital,
        KARIN.localitate,
        KARIN.address,
        KARIN.cod_postal,
        KARIN.bank_name,
        KARIN.bank_account,
        invoice.partner_name,
        invoice.partner_cif.as_deref().unwrap_or("N/A"),
        invoice.partner_reg_com.as_deref().unwrap_or("N/A"),
        invoice.location_name,
        format!("Adresa: {}", invoice.location_address.as_deref().unwrap_or("N/A")),
        products_html,
        total_without_vat,
        total_vat,
        total_without_vat + total_vat,  // Total General = Subtotal + TVA
        due_date,        if let Some(car_num) = car_number {
            format!(r#"
    <div class="legal-note" style="margin-top: 10px; border-top: 1px solid #ddd; padding-top: 8px;">
        <strong>Certificăm faptul că mașina cu numărul {} a fost dezinfectată cu Virocid 1% înainte de încărcare.</strong>
    </div>"#, car_num)
        } else {
            String::new()
        },        delegate_name.unwrap_or("........................"),
        delegate_act.unwrap_or("....................................."),
        if let Some(logo) = logo_base64 {
            format!(r#"<img src="{}" class="footer-logo" alt="Logo" />"#, logo)
        } else {
            String::new()
        }
    )
}

fn format_date(iso_date: &str) -> String {
    // Parse ISO date like "2025-01-15T12:34:56Z"
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(iso_date) {
        dt.format("%d-%m-%Y").to_string()
    } else {
        iso_date.to_string()
    }
}

fn calculate_due_date(created_at: &str, days: i64) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(created_at) {
        let due = dt + chrono::Duration::days(days);
        due.format("%d-%m-%Y").to_string()
    } else {
        created_at.to_string()
    }
}
