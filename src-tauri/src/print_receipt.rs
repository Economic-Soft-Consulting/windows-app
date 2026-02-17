use crate::models::Collection;
use crate::print_invoice::KARIN;

pub fn generate_receipt_html(
    collection: &Collection,
    logo_base64: Option<&str>,
    doc_series: &str,
    doc_number: &str,
    issue_date: &str,
    agent: Option<&str>,
    _nume_casa: &str,
    partner_address: Option<&str>,
    partner_localitate: Option<&str>,
    partner_judet: Option<&str>,
    partner_cui: Option<&str>,
    partner_reg_com: Option<&str>,
) -> String {
    let partner_name = collection
        .partner_name
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("N/A");

    let factura_ref = match (&collection.serie_factura, &collection.numar_factura) {
        (Some(serie), Some(numar)) if !serie.trim().is_empty() || !numar.trim().is_empty() => {
            format!("{} {}", serie, numar).trim().to_string()
        }
        _ => "N/A".to_string(),
    };

    let amount_display = format!("{:.2}", collection.valoare).replace('.', ",");
    let cashier_display = agent
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("-");
    let partner_address_display = partner_address
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("... nedefinit ...");
    let partner_localitate_display = partner_localitate
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("... nedefinit ...");
    let partner_judet_display = partner_judet
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("... nedefinit ...");
    let partner_cui_display = partner_cui
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(collection.id_partener.as_str());
    let partner_reg_com_display = partner_reg_com
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("-");
    let city = KARIN
        .localitate
        .split(',')
        .next()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .unwrap_or(KARIN.localitate);
    let county = KARIN
        .localitate
        .split("Jud.")
        .nth(1)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .unwrap_or("-");
    let sediu_line = format!("{}, {} CP.{}", city, KARIN.address, KARIN.cod_postal);

    format!(
        r#"<!DOCTYPE html>
<html lang="ro">
<head>
    <meta charset="UTF-8">
    <title>Chitanta KARIN</title>
    <style>
        @media print {{
            @page {{
                size: 80mm 297mm;
                margin: 3mm 4mm 3mm 3mm;
            }}
            body {{
                margin: 0;
                padding: 0;
            }}
            header, footer {{
                display: none;
            }}
        }}

        body {{
            font-family: Arial, Helvetica, sans-serif;
            font-size: 10.5px;
            font-weight: bold;
            color: #000000;
            line-height: 1.2;
            background: white;
        }}

        .page {{
            width: 100%;
            min-height: 291mm;
            box-sizing: border-box;
            padding: 3mm;
            display: flex;
            flex-direction: column;
            justify-content: flex-start;
            margin: 0 auto;
        }}

        .top {{
            display: flex;
            flex-direction: column;
            align-items: stretch;
            gap: 3mm;
        }}

        .left-meta, .right-meta {{
            white-space: pre-line;
            word-break: break-word;
        }}

        .left-meta {{
            width: 100%;
        }}

        .right-meta {{
            width: 100%;
            text-align: left;
        }}

        .title-wrap {{
            margin-top: 4px;
            margin-bottom: 6px;
        }}

        .title {{
            font-size: 15px;
            letter-spacing: 0.3px;
            margin: 0;
            text-align: center;
        }}

        .section {{
            margin-top: 6mm;
        }}

        .row-label {{
            margin-bottom: 2mm;
            text-decoration: underline;
        }}

        .details {{
            margin-top: 2mm;
            white-space: pre-line;
            word-break: break-word;
        }}

        .cashier {{
            margin-top: 7mm;
            text-align: right;
        }}

        .logo-wrap {{
            margin-top: 5mm;
            text-align: center;
        }}

        .footer-logo {{
            width: 100%;
            max-width: 66mm;
            max-height: 48mm;
            height: auto;
            object-fit: contain;
        }}

        .printed-by {{
            margin-top: 2mm;
            font-size: 9px;
            font-weight: bold;
            text-align: center;
        }}
    </style>
</head>
<body>
    <div class="page">
        <div>
            <div class="top">
                <div class="right-meta">Seria: {}
Numar: {}

                    <div class="title-wrap">
                        <p class="title">CHITANTA</p>
                    </div>

Seria: {}
Numar: {}
DATA: {}</div>

                <div class="left-meta"><span style="text-decoration: underline;">FURNIZOR:</span>
{}
NR..INM. {}
C.U.I.: {}
Sediul: {}
Jud.: {}
Capital social: {}
Tel.: {}
E-mail: {}</div>
            </div>

            <div class="section">
                <div class="row-label">AM PRIMIT DE LA: {}</div>

                <div class="details">Adresa: {}
Localitatea {}, Judetul {}
CUI: {}
Nr. Inm. {}
SUMA DE: {} ({})
Reprezentand: {}</div>
            </div>

            <div class="cashier">CASIER,
{}</div>
        </div>

        <div class="logo-wrap">{}
            <div class="printed-by">printed by eSoft</div>
        </div>
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
        doc_series,
        doc_number,
        doc_series,
        doc_number,
        issue_date,
    KARIN.name,
    KARIN.reg_com,
    KARIN.cif,
    sediu_line,
    county,
    KARIN.capital,
    "0753068450",
    "nasesem@yahoo.com",
        partner_name,
        partner_address_display,
        partner_localitate_display,
        partner_judet_display,
        partner_cui_display,
        partner_reg_com_display,
        amount_display,
        "-",
        format!("Încasare factură {}", factura_ref),
        cashier_display,
        if let Some(logo) = logo_base64 {
            format!(r#"<img src="{}" class="footer-logo" alt="Logo" />"#, logo)
        } else {
            String::new()
        }
    )
}