use crate::models::{Invoice};
use crate::print_invoice::KARIN;

pub fn generate_daily_report_html(
    invoices: &[Invoice],
    date: &str,
    total_sales: f64,
    logo_base64: Option<&str>,
) -> String {
    log::info!("📄 Generating daily sales report HTML for date: {}", date);

    let rows_html = invoices
        .iter()
        .enumerate()
        .map(|(idx, inv)| {
            // Extract last 8 chars of ID for display
            let short_id = if inv.id.len() > 8 {
                &inv.id[inv.id.len() - 8..]
            } else {
                &inv.id
            };
            format!(
                r#"
                <div class="report-row">
                    <div class="col-idx">{}</div>
                    <div class="col-inv">{}</div>
                    <div class="col-partner">{}</div>
                    <div class="col-amount">{:.2}</div>
                </div>
                "#,
                idx + 1,
                short_id,
                inv.partner_name,
                inv.total_amount
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<!DOCTYPE html>
<html lang="ro">
<head>
    <meta charset="UTF-8">
    <title>RAPORT ZILNIC - {}</title>
    <style>
        @media print {{
            @page {{
                size: 80mm 297mm;
                margin: 2mm;
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
            font-family: 'Courier New', Courier, monospace;
            width: 76mm;
            margin: 0;
            padding: 1mm;
            font-size: 9.5px;
            font-weight: bold;
            color: #000000;
            line-height: 1.15;
            background: white;
            box-sizing: border-box;
            overflow-wrap: anywhere;
        }}

        h1 {{
            font-size: 12px;
            text-align: center;
            margin: 3px 0;
            text-transform: uppercase;
            border-bottom: 1px dashed #000;
            padding-bottom: 3px;
        }}

        .header-section {{
            text-align: center;
            margin-bottom: 6px;
            border-bottom: 1px dashed #000;
            padding-bottom: 3px;
            font-size: 9px;
            line-height: 1.1;
        }}

        .report-section {{
            margin-top: 6px;
        }}

        .report-header {{
            display: flex;
            border-bottom: 1px solid #000;
            padding-bottom: 2px;
            margin-bottom: 3px;
            font-size: 8.5px;
        }}

        .report-row {{
            display: flex;
            margin-bottom: 2px;
            font-size: 9px;
            align-items: flex-start;
        }}

        .col-idx {{ width: 4mm; flex: 0 0 4mm; }}
        .col-inv {{ width: 15mm; flex: 0 0 15mm; }}
        .col-partner {{ flex: 1; min-width: 0; word-break: break-word; overflow-wrap: anywhere; padding-right: 1mm; }}
        .col-amount {{ width: 14mm; flex: 0 0 14mm; text-align: right; white-space: nowrap; }}

        .total-section {{
            margin-top: 6px;
            border-top: 2px dashed #000;
            padding-top: 3px;
            text-align: right;
            font-size: 11px;
        }}

        .footer-branding {{
            text-align: center;
            font-size: 8.5px;
            margin-top: 10px;
            font-style: italic;
        }}

        .footer-logo {{
            width: 100%;
            max-width: 66mm;
            height: auto;
            display: block;
            margin: 0 auto 5px auto;
        }}
    </style>
</head>
<body>

    <div class="header-section">
        {}<br>
        CIF: {}<br>
        {}<br>
        DATA: {}
    </div>

    <h1>RAPORT VANZARI ZILNIC</h1>
    
    <div class="report-section">
        <div class="report-header">
            <div class="col-idx">#</div>
            <div class="col-inv">DOC</div>
            <div class="col-partner">CLIENT</div>
            <div class="col-amount">VAL</div>
        </div>
        
        {}
    </div>

    <div class="total-section">
        TOTAL VANZARI:<br>
        {:.2} RON
    </div>

    <div class="footer-branding">
        {}
        <br>
        printed by eSoft
    </div>

    <script>
        function triggerPrint() {{
            window.print();
        }}
        
        if (document.readyState === 'loading') {{
            document.addEventListener('DOMContentLoaded', function() {{
                setTimeout(triggerPrint, 300);
            }});
        }} else {{
            triggerPrint();
        }}
        
        window.addEventListener('load', function() {{
            setTimeout(triggerPrint, 100);
        }});
    </script>
</body>
</html>"#,
        date,
        KARIN.name,
        KARIN.cif,
        KARIN.address,
        date,
        rows_html,
        total_sales,
        if let Some(logo) = logo_base64 {
            format!(r#"<img src="{}" class="footer-logo" alt="Logo" />"#, logo)
        } else {
            String::new()
        }
    )
}
