use crate::models::{Invoice, InvoiceItem};

pub struct CompanyInfo {
    pub name: &'static str,
    pub cif: &'static str,
    pub reg_com: &'static str,
    pub address: &'static str,
    pub bank_name: &'static str,
    pub bank_account: &'static str,
    pub capital: &'static str,
}

// KARIN company details
pub const KARIN: CompanyInfo = CompanyInfo {
    name: "KARIN FASHION SRL",
    cif: "12345678",
    reg_com: "J40/123456/2023",
    address: "Str. Designului 25, Sector 4, BUCURESTI",
    bank_name: "BRD - Groupe Société Générale",
    bank_account: "RO12BRDE445SV20475833001 (RON)",
    capital: "200 RON",
};

pub fn generate_invoice_html(
    invoice: &Invoice,
    items: &[InvoiceItem],
    invoice_number: i64,
) -> String {
    let vat_rate = 0.24;
    let total_without_vat = invoice.total_amount / (1.0 + vat_rate);
    let total_vat = invoice.total_amount - total_without_vat;
    
    let due_date = calculate_due_date(&invoice.created_at, 10); // 10 days payment term

    let products_html = items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let item_vat = item.total_price * vat_rate;
            format!(
                r#"        <div class="product-item">
            <span class="prod-name">{}. {}</span>
            <div class="prod-math">
                <span>{} {} x {:.2}</span>
                <span>= {:.2}</span>
            </div>
            <div class="prod-vat">Valoare TVA: {:.2}</div>
        </div>"#,
                idx + 1,
                item.product_name,
                item.quantity as i32,
                item.unit_of_measure,
                item.unit_price,
                item.total_price,
                item_vat
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let invoice_series = format!("KARIN-F-{:06}", invoice_number);

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
                margin: 0;
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
            width: 76mm;
            margin: 0 auto;
            padding: 2mm;
            padding-bottom: 50px; 
            font-size: 13px;
            font-weight: bold;
            color: #000000;
            line-height: 1.2;
            background: white;
            position: relative;
            min-height: 290mm;
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

        .prod-vat {{
            text-align: right;
            font-size: 12px;
            font-weight: normal;
            color: #000;
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
            position: absolute;
            bottom: 5mm;
            left: 0;
            width: 100%;
            text-align: center;
            font-size: 10px;
            color: #000000;
            font-weight: normal;
            font-style: italic;
            border-top: 1px solid #000;
            padding-top: 5px;
            background-color: white;
        }}
    </style>
</head>
<body>

    <h1>FACTURA FISCALA</h1>
    
    <div class="header-meta">
        Seria: {} &nbsp; Nr: {}<br>
        Data (zi/luna/an): {}
    </div>

    <div class="section">
        <span class="section-title">FURNIZOR:</span>
        {}<br>
        CIF: {}<br>
        Reg.Com: {}<br>
        Sediul: {}<br>
        
        <div class="compact-row">
            Banca: {}<br>
            Cont: {}
        </div>
        
        <div class="compact-row">
            Capital Social: {}
        </div>
    </div>

    <div class="section">
        <span class="section-title">CUMPARATOR:</span>
        {}<br>
        Locatie: {}<br>
        {}
    </div>

    <div style="font-size: 12px; margin-bottom: 5px;">
        Cota TVA: 24% (TVA la incasare)
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
        Pt depasirea termenului scadent se percepe o penalizare de 0.5% pe zi din valoarea totala.<br>
        Prezenta tine loc de contract ferm intre parti in lipsa altui acord scris.<br>
        <br>
        <strong>Data Scadenta: {}</strong>
    </div>

    <div class="signatures">
        
        <div class="sig-block">
            Semnatura si stampila Furnizor:<br>
            <span class="dots"></span>
        </div>

        <div class="sig-block">
            Numele Delegatului: ........................<br>
            Act Delegat: .....................................<br>
            Semnatura: <span class="dots" style="width: 50%;"></span>
        </div>

        <div class="sig-block">
            Semnatura de primire:<br>
            <span class="dots"></span>
        </div>

    </div>

    <div class="footer-branding">
        printed by karin app
    </div>

</body>
</html>"#,
        invoice_series,
        invoice_number,
        format_date(&invoice.created_at),
        KARIN.name,
        KARIN.cif,
        KARIN.reg_com,
        KARIN.address,
        KARIN.bank_name,
        KARIN.bank_account,
        KARIN.capital,
        invoice.partner_name,
        invoice.location_name,
        format!("Adresa: {}", invoice.partner_name),
        products_html,
        total_without_vat,
        total_vat,
        invoice.total_amount,
        due_date
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
