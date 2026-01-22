use serde::{Deserialize, Serialize};
use log::{info, error};

// ==================== API CONFIGURATION ====================

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub base_url: String,
    #[allow(dead_code)]
    pub username: Option<String>,
}

impl ApiConfig {
    pub fn new(ip: &str, port: u16, username: Option<String>) -> Self {
        Self {
            base_url: format!("http://{}:{}/datasnap/rest/TServerMethods", ip, port),
            username,
        }
    }

    pub fn from_env_or_default() -> Self {
        // Default configuration - can be changed via settings
        Self::new("10.200.1.94", 8089, None)
    }
}

// ==================== API REQUEST/RESPONSE STRUCTURES ====================

#[derive(Debug, Serialize)]
pub struct PartnerFilter {
    #[serde(rename = "DataReferinta", skip_serializing_if = "Option::is_none")]
    pub data_referinta: Option<String>,
    #[serde(rename = "Denumire", skip_serializing_if = "Option::is_none")]
    pub denumire: Option<String>,
    #[serde(rename = "Telefon", skip_serializing_if = "Option::is_none")]
    pub telefon: Option<String>,
    #[serde(rename = "MarcaAgent", skip_serializing_if = "Option::is_none")]
    pub marca_agent: Option<String>,
    #[serde(rename = "CodFiscal", skip_serializing_if = "Option::is_none")]
    pub cod_fiscal: Option<String>,
    #[serde(rename = "Email", skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(rename = "SimbolClasa", skip_serializing_if = "Option::is_none")]
    pub simbol_clasa: Option<String>,
    #[serde(rename = "Paginare", skip_serializing_if = "Option::is_none")]
    pub paginare: Option<Pagination>,
}

#[derive(Debug, Serialize)]
pub struct ArticleFilter {
    #[serde(rename = "DataReferinta", skip_serializing_if = "Option::is_none")]
    pub data_referinta: Option<String>,
    #[serde(rename = "Denumire", skip_serializing_if = "Option::is_none")]
    pub denumire: Option<String>,
    #[serde(rename = "Clasa", skip_serializing_if = "Option::is_none")]
    pub clasa: Option<String>,
    #[serde(rename = "SimbolClasa", skip_serializing_if = "Option::is_none")]
    pub simbol_clasa: Option<Vec<String>>,
    #[serde(rename = "VizibilComenziOnline", skip_serializing_if = "Option::is_none")]
    pub vizibil_comenzi_online: Option<String>,
    #[serde(rename = "Inactiv", skip_serializing_if = "Option::is_none")]
    pub inactiv: Option<String>,
    #[serde(rename = "Blocat", skip_serializing_if = "Option::is_none")]
    pub blocat: Option<String>,
    #[serde(rename = "Paginare", skip_serializing_if = "Option::is_none")]
    pub paginare: Option<Pagination>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Pagination {
    #[serde(rename = "Pagina", skip_serializing_if = "Option::is_none")]
    pub pagina: Option<String>,
    #[serde(rename = "Inregistrari", skip_serializing_if = "Option::is_none")]
    pub inregistrari: Option<String>,
    #[serde(rename = "TotalPagini", skip_serializing_if = "Option::is_none")]
    pub total_pagini: Option<String>,
}

// ==================== PARTNER API STRUCTURES ====================

#[derive(Debug, Deserialize)]
pub struct PartnerResponse {
    #[serde(rename = "Result")]
    #[allow(dead_code)]
    pub result: Option<String>,
    #[serde(rename = "Paginare")]
    pub paginare: Option<Pagination>,
    #[serde(rename = "InfoParteneri")]
    pub info_parteneri: Vec<PartnerInfo>,
}

#[derive(Debug, Deserialize)]
pub struct PartnerInfo {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Cod")]
    pub cod: Option<String>,
    #[serde(rename = "Denumire")]
    pub denumire: String,
    #[serde(rename = "CodFiscal")]
    pub cod_fiscal: Option<String>,
    #[serde(rename = "RegistruComert")]
    pub registru_comert: Option<String>,
    #[serde(rename = "Blocat")]
    pub blocat: Option<String>,
    #[serde(rename = "TVALaIncasare")]
    pub tva_la_incasare: Option<String>,
    #[serde(rename = "PersoanaFizica")]
    pub persoana_fizica: Option<String>,
    #[serde(rename = "CodExtern")]
    pub cod_extern: Option<String>,
    #[serde(rename = "CodIntern")]
    pub cod_intern: Option<String>,
    #[serde(rename = "Observatii")]
    pub observatii: Option<String>,
    #[serde(rename = "DataAdaugarii")]
    pub data_adaugarii: Option<String>,
    // New fields for extended schema
    #[serde(rename = "Clasa")]
    pub clasa: Option<String>,
    #[serde(rename = "SimbolClasa")]
    pub simbol_clasa: Option<String>,
    #[serde(rename = "CodClasa")]
    pub cod_clasa: Option<String>,
    #[serde(rename = "CategoriePretImplicita")]
    pub categorie_pret_implicita: Option<String>,
    #[serde(rename = "SimbolCategoriePret")]
    pub simbol_categorie_pret: Option<String>,
    #[serde(rename = "ScadentaLaVanzare")]
    pub scadenta_la_vanzare: Option<String>,
    #[serde(rename = "ScadentaLaCumparare")]
    pub scadenta_la_cumparare: Option<String>,
    #[serde(rename = "DiscountFix")]
    pub discount_fix: Option<String>,
    #[serde(rename = "TipPartener")]
    pub tip_partener: Option<String>,
    #[serde(rename = "ModAplicareDiscount")]
    pub mod_aplicare_discount: Option<String>,
    #[serde(rename = "Moneda")]
    pub moneda: Option<String>,
    #[serde(rename = "DataNastere")]
    pub data_nastere: Option<String>,
    #[serde(rename = "CaracterizareContabilaDenumire")]
    pub caracterizare_contabila_denumire: Option<String>,
    #[serde(rename = "CaracterizareContabilaSimbol")]
    pub caracterizare_contabila_simbol: Option<String>,
    #[serde(rename = "Inactiv")]
    pub inactiv: Option<String>,
    #[serde(rename = "CreditClient")]
    pub credit_client: Option<String>,
    #[serde(rename = "Sedii")]
    pub sedii: Vec<SediuInfo>,
}

#[derive(Debug, Deserialize)]
pub struct SediuInfo {
    #[serde(rename = "IDSediu")]
    pub id_sediu: String,
    #[serde(rename = "CodSediu")]
    pub cod_sediu: Option<String>,
    #[serde(rename = "Denumire")]
    pub denumire: String,
    #[serde(rename = "Localitate")]
    pub localitate: Option<String>,
    #[serde(rename = "Strada")]
    pub strada: Option<String>,
    #[serde(rename = "Numar")]
    pub numar: Option<String>,
    #[serde(rename = "Judet")]
    pub judet: Option<String>,
    #[serde(rename = "Tara")]
    pub tara: Option<String>,
    #[serde(rename = "CodPostal")]
    pub cod_postal: Option<String>,
    #[serde(rename = "Telefon")]
    pub telefon: Option<String>,
    #[serde(rename = "eMail")]
    pub email: Option<String>,
    #[serde(rename = "Inactiv")]
    pub inactiv: Option<String>,
}

// ==================== ARTICLE API STRUCTURES ====================

#[derive(Debug, Deserialize)]
pub struct ArticleResponse {
    #[serde(rename = "result")]
    #[allow(dead_code)]
    pub result: Option<String>,
    #[serde(rename = "Paginare")]
    pub paginare: Option<Pagination>,
    #[serde(rename = "InfoArticole")]
    pub info_articole: Vec<ArticleInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ArticleInfo {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "CodObiect")]
    #[allow(dead_code)]
    pub cod_obiect: Option<String>,
    #[serde(rename = "Denumire")]
    pub denumire: String,
    #[serde(rename = "UM")]
    pub um: String,
    #[serde(rename = "PretVanzare")]
    pub pret_vanzare: Option<String>,
    #[serde(rename = "PretCuTVA")]
    #[allow(dead_code)]
    pub pret_cu_tva: Option<String>,
    #[serde(rename = "ProcentTVA")]
    #[allow(dead_code)]
    pub procent_tva: Option<String>,
    #[serde(rename = "CodExtern")]
    #[allow(dead_code)]
    pub cod_extern: Option<String>,
    #[serde(rename = "CodIntern")]
    #[allow(dead_code)]
    pub cod_intern: Option<String>,
    #[serde(rename = "Clasa")]
    pub clasa: Option<String>,
    #[serde(rename = "SimbolClasa")]
    #[allow(dead_code)]
    pub simbol_clasa: Option<String>,
    #[serde(rename = "Serviciu")]
    #[allow(dead_code)]
    pub serviciu: Option<String>,
    #[serde(rename = "Inactiv")]
    #[allow(dead_code)]
    pub inactiv: Option<String>,
    #[serde(rename = "Blocat")]
    #[allow(dead_code)]
    pub blocat: Option<String>,
    #[serde(rename = "DataAdaugarii")]
    #[allow(dead_code)]
    pub data_adaugarii: Option<String>,
    #[serde(rename = "Descriere")]
    #[allow(dead_code)]
    pub descriere: Option<String>,
}

// ==================== API CLIENT ====================

pub struct ApiClient {
    config: ApiConfig,
    client: reqwest::Client,
}

impl ApiClient {
    pub fn new(config: ApiConfig) -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        Ok(Self { config, client })
    }

    pub fn from_default() -> Result<Self, String> {
        Self::new(ApiConfig::from_env_or_default())
    }

    // Get all partners (with pagination)
    pub async fn get_partners(&self, filter: Option<PartnerFilter>) -> Result<PartnerResponse, String> {
        let url = format!("{}/\"GetInfoParteneri\"", self.config.base_url);
        
        info!("Fetching partners from API: {}", url);

        let filter = filter.unwrap_or(PartnerFilter {
            data_referinta: None,
            denumire: None,
            telefon: None,
            marca_agent: None,
            cod_fiscal: None,
            email: None,
            simbol_clasa: None,
            paginare: None,
        });

        let response = self.client
            .post(&url)
            .json(&filter)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch partners: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API returned error status: {}", response.status()));
        }

        let partner_response: PartnerResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse partner response: {}", e))?;

        info!("Successfully fetched {} partners", partner_response.info_parteneri.len());

        Ok(partner_response)
    }

    // Get all articles (with pagination)
    pub async fn get_articles(&self, filter: Option<ArticleFilter>) -> Result<ArticleResponse, String> {
        let url = format!("{}/\"GetInfoArticole\"", self.config.base_url);
        
        info!("Fetching articles from API: {}", url);

        let filter = filter.unwrap_or(ArticleFilter {
            data_referinta: None,
            denumire: None,
            clasa: None,
            simbol_clasa: None,
            vizibil_comenzi_online: None,
            inactiv: Some("NU".to_string()), // Only active articles
            blocat: Some("NU".to_string()),   // Only non-blocked articles
            paginare: None,
        });

        let response = self.client
            .post(&url)
            .json(&filter)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch articles: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API returned error status: {}", response.status()));
        }

        let article_response: ArticleResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse article response: {}", e))?;

        info!("Successfully fetched {} articles", article_response.info_articole.len());

        Ok(article_response)
    }

    // Fetch all partners with automatic pagination
    pub async fn get_all_partners(&self) -> Result<Vec<PartnerInfo>, String> {
        let mut all_partners = Vec::new();
        let mut page = 1;
        let per_page = 100;

        loop {
            let filter = PartnerFilter {
                data_referinta: None,
                denumire: None,
                telefon: None,
                marca_agent: None,
                cod_fiscal: None,
                email: None,
                simbol_clasa: Some("AGENTI".to_string()),
                paginare: Some(Pagination {
                    pagina: Some(page.to_string()),
                    inregistrari: Some(per_page.to_string()),
                    total_pagini: None,
                }),
            };

            match self.get_partners(Some(filter)).await {
                Ok(response) => {
                    let count = response.info_parteneri.len();
                    
                    if count == 0 {
                        info!("No more partners to fetch on page {}", page);
                        break;
                    }
                    
                    all_partners.extend(response.info_parteneri);

                    info!("Fetched page {} with {} partners (total so far: {})", page, count, all_partners.len());

                    // Check pagination info from response
                    let should_continue = if let Some(paginare) = &response.paginare {
                        info!("Pagination info: {:?}", paginare);
                        
                        if let Some(total_pages_str) = &paginare.total_pagini {
                            if let Ok(total_pages) = total_pages_str.parse::<i32>() {
                                info!("Total pages from API: {}, current page: {}", total_pages, page);
                                page < total_pages
                            } else {
                                // Can't parse total_pages, continue if we got results
                                count > 0
                            }
                        } else {
                            // No total_pages info, continue if we got results
                            count > 0
                        }
                    } else {
                        // No pagination info, continue if we got results
                        count > 0
                    };

                    if !should_continue {
                        info!("Stopping pagination: reached last page or no pagination info");
                        break;
                    }

                    page += 1;
                }
                Err(e) => {
                    error!("Failed to fetch partners page {}: {}", page, e);
                    return Err(e);
                }
            }
        }

        info!("✅ Total partners fetched: {}", all_partners.len());
        Ok(all_partners)
    }

    // Fetch all articles with automatic pagination
    pub async fn get_all_articles(&self) -> Result<Vec<ArticleInfo>, String> {
        let mut all_articles = Vec::new();
        let mut page = 1;
        let per_page = 100;

        loop {
            let filter = ArticleFilter {
                data_referinta: None,
                denumire: None,
                clasa: None,
                simbol_clasa: Some(vec!["OUA".to_string()]),
                vizibil_comenzi_online: None,
                inactiv: Some("NU".to_string()),
                blocat: Some("NU".to_string()),
                paginare: Some(Pagination {
                    pagina: Some(page.to_string()),
                    inregistrari: Some(per_page.to_string()),
                    total_pagini: None,
                }),
            };

            match self.get_articles(Some(filter)).await {
                Ok(response) => {
                    let count = response.info_articole.len();
                    
                    if count == 0 {
                        info!("No more articles to fetch on page {}", page);
                        break;
                    }
                    
                    all_articles.extend(response.info_articole);

                    info!("Fetched page {} with {} articles (total so far: {})", page, count, all_articles.len());

                    // Check pagination info from response
                    let should_continue = if let Some(paginare) = &response.paginare {
                        info!("Pagination info: {:?}", paginare);
                        
                        if let Some(total_pages_str) = &paginare.total_pagini {
                            if let Ok(total_pages) = total_pages_str.parse::<i32>() {
                                info!("Total pages from API: {}, current page: {}", total_pages, page);
                                page < total_pages
                            } else {
                                // Can't parse total_pages, continue if we got results
                                count > 0
                            }
                        } else {
                            // No total_pages info, continue if we got results
                            count > 0
                        }
                    } else {
                        // No pagination info, continue if we got results
                        count > 0
                    };

                    if !should_continue {
                        info!("Stopping pagination: reached last page or no pagination info");
                        break;
                    }

                    page += 1;
                }
                Err(e) => {
                    error!("Failed to fetch articles page {}: {}", page, e);
                    return Err(e);
                }
            }
        }

        info!("✅ Total articles fetched: {}", all_articles.len());
        Ok(all_articles)
    }
}

// Helper function to parse string boolean
#[allow(dead_code)]
pub fn parse_bool(s: &Option<String>) -> bool {
    s.as_ref()
        .map(|val| val.to_uppercase() == "DA" || val.to_uppercase() == "YES")
        .unwrap_or(false)
}

// Helper function to parse string number
#[allow(dead_code)]
pub fn parse_f64(s: &Option<String>) -> f64 {
    s.as_ref()
        .and_then(|val| val.parse::<f64>().ok())
        .unwrap_or(0.0)
}

// ==================== WME INVOICE STRUCTURES ====================

#[derive(Debug, Serialize)]
pub struct OfferFilter {
    #[serde(rename = "DataReferinta", skip_serializing_if = "Option::is_none")]
    pub data_referinta: Option<String>,
    #[serde(rename = "DataAnaliza", skip_serializing_if = "Option::is_none")]
    pub data_analiza: Option<String>,
    #[serde(rename = "CodPartener", skip_serializing_if = "Option::is_none")]
    pub cod_partener: Option<String>,
    #[serde(rename = "Furnizori", skip_serializing_if = "Option::is_none")]
    pub furnizori: Option<Vec<String>>,
    #[serde(rename = "CodSubunit", skip_serializing_if = "Option::is_none")]
    pub cod_subunit: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OfferResponse {
    #[serde(rename = "InfoOferte")]
    #[serde(default)]
    pub info_oferte: Vec<OfferInfo>,
}

#[derive(Debug, Deserialize)]
pub struct OfferInfo {
    #[serde(rename = "ID")]
    pub id: Option<String>,
    #[serde(rename = "IDClient")]
    pub id_client: Option<String>,
    #[serde(rename = "Numar")]
    pub numar: Option<String>,
    #[serde(rename = "DataInceput")]
    pub data_inceput: Option<String>,
    #[serde(rename = "DataSfarsit")]
    pub data_sfarsit: Option<String>,
    #[serde(rename = "Anulata")]
    pub anulata: Option<String>,
    #[serde(rename = "Client")]
    pub client: Option<String>,
    #[serde(rename = "TipOferta")]
    pub tip_oferta: Option<String>,
    #[serde(rename = "Furnizor")]
    pub furnizor: Option<String>,
    #[serde(rename = "IDFurnizor")]
    pub id_furnizor: Option<String>,
    #[serde(rename = "CodFiscal")]
    pub cod_fiscal: Option<String>,
    #[serde(rename = "SimbolClasa")]
    pub simbol_clasa: Option<String>,
    #[serde(rename = "Moneda")]
    pub moneda: Option<String>,
    #[serde(rename = "Observatii")]
    pub observatii: Option<String>,
    #[serde(rename = "ExtensieDocument")]
    pub extensie_document: Option<String>,
    #[serde(rename = "Items")]
    pub items: Option<Vec<OfferItem>>,
}

#[derive(Debug, Deserialize)]
pub struct OfferItem {
    #[serde(rename = "ID")]
    pub id: Option<String>,
    #[serde(rename = "Denumire")]
    pub denumire: Option<String>,
    #[serde(rename = "UM")]
    pub um: Option<String>,
    #[serde(rename = "CantMinima")]
    pub cant_minima: Option<String>,
    #[serde(rename = "CantMaxima")]
    pub cant_maxima: Option<String>,
    #[serde(rename = "CantOptima")]
    pub cant_optima: Option<String>,
    #[serde(rename = "Cantitate")]
    pub cantitate: Option<f64>,
    #[serde(rename = "Pret")]
    pub pret: Option<String>,
    #[serde(rename = "Discount")]
    pub discount: Option<String>,
    #[serde(rename = "ProcAdaos")]
    pub proc_adaos: Option<String>,
    #[serde(rename = "PretRef")]
    pub pret_ref: Option<String>,
    #[serde(rename = "PretCuProcAdaos")]
    pub pret_cu_proc_adaos: Option<String>,
    #[serde(rename = "Observatii")]
    pub observatii: Option<String>,
    #[serde(rename = "CodOferta1")]
    pub cod_oferta1: Option<String>,
    #[serde(rename = "EXTENSIELINIE")]
    pub extensie_linie: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WmeInvoiceItem {
    #[serde(rename = "IDArticol")]
    pub id_articol: String,
    #[serde(rename = "Cant")]
    pub cant: f64,
    #[serde(rename = "Pret")]
    pub pret: f64,
    #[serde(rename = "UM", skip_serializing_if = "Option::is_none")]
    pub um: Option<String>,
    #[serde(rename = "Gestiune", skip_serializing_if = "Option::is_none")]
    pub gestiune: Option<String>,
    #[serde(rename = "Observatii", skip_serializing_if = "Option::is_none")]
    pub observatii: Option<String>,
    #[serde(rename = "TVA", skip_serializing_if = "Option::is_none")]
    pub tva: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct WmeDocument {
    #[serde(rename = "TipDocument")]
    pub tip_document: String,
    #[serde(rename = "SimbolGestiune")]
    pub simbol_gestiune: String,
    #[serde(rename = "NumeGestiune")]
    pub nume_gestiune: String,
    #[serde(rename = "NumerotareAutomata", skip_serializing_if = "Option::is_none")]
    pub numerotare_automata: Option<WmeNumerotareAutomata>,
    #[serde(rename = "SerieDocument", skip_serializing_if = "Option::is_none")]
    pub serie_document: Option<String>,
    #[serde(rename = "NrDoc", skip_serializing_if = "Option::is_none")]
    pub numar_document: Option<String>,
    #[serde(rename = "SimbolCarnet", skip_serializing_if = "Option::is_none")]
    pub simbol_carnet: Option<String>,
    #[serde(rename = "SimbolCarnetLivr", skip_serializing_if = "Option::is_none")]
    pub simbol_carnet_livr: Option<String>,
    #[serde(rename = "SimbolGestiuneLivrare", skip_serializing_if = "Option::is_none")]
    pub simbol_gestiune_livrare: Option<String>,
    #[serde(rename = "Data", skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(rename = "DataLivr", skip_serializing_if = "Option::is_none")]
    pub data_livr: Option<String>,
    #[serde(rename = "CodClient", skip_serializing_if = "Option::is_none")]
    pub cod_client: Option<String>,
    #[serde(rename = "IDSediu", skip_serializing_if = "Option::is_none")]
    pub id_sediu: Option<String>,
    #[serde(rename = "Agent", skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(rename = "Observatii", skip_serializing_if = "Option::is_none")]
    pub observatii: Option<String>,
    #[serde(rename = "Items", skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<WmeInvoiceItem>>,
}

#[derive(Debug, Serialize)]
pub struct WmeNumerotareAutomata {
    #[serde(rename = "SimbolCarnet")]
    pub simbol_carnet: String,
    #[serde(rename = "CodCarnet")]
    pub cod_carnet: String,
    #[serde(rename = "CodCarnetLivr", skip_serializing_if = "Option::is_none")]
    pub cod_carnet_livr: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WmeInvoiceRequest {
    #[serde(rename = "CodPartener")]
    pub cod_partener: String,
    #[serde(rename = "CodSediu", skip_serializing_if = "Option::is_none")]
    pub cod_sediu: Option<String>,
    #[serde(rename = "NumeDelegate")]
    pub nume_delegate: String,
    #[serde(rename = "ActDelegate")]
    pub act_delegate: String,
    #[serde(rename = "TipDocument", skip_serializing_if = "Option::is_none")]
    pub tip_document: Option<String>,
    #[serde(rename = "AnLucru", skip_serializing_if = "Option::is_none")]
    pub an_lucru: Option<String>,
    #[serde(rename = "LunaLucru", skip_serializing_if = "Option::is_none")]
    pub luna_lucru: Option<String>,
    #[serde(rename = "CodSubunitate", skip_serializing_if = "Option::is_none")]
    pub cod_subunitate: Option<String>,
    #[serde(rename = "Documente")]
    pub documente: Vec<WmeDocument>,
    #[serde(rename = "Articole")]
    pub articole: Vec<WmeInvoiceItem>,
}

#[derive(Debug, Deserialize)]
pub struct WmeInvoiceResponse {
    #[serde(rename = "Result")]
    pub result: Option<String>,
    #[serde(rename = "NumarDocumente")]
    pub numar_documente: Option<String>,
    #[serde(rename = "DocumenteImportate")]
    #[serde(default)]
    pub documente_importate: Vec<WmeDocumentImport>,
}

#[derive(Debug, Deserialize)]
pub struct WmeDocumentImport {
    #[serde(rename = "Numar")]
    pub numar: Option<String>,
    #[serde(rename = "Serie")]
    pub serie: Option<String>,
    #[serde(rename = "Operat")]
    pub operat: Option<String>,
    #[serde(rename = "CodIes")]
    pub cod_ies: Option<String>,
}

impl ApiClient {
    // Get offers for a partner
    pub async fn get_offers(&self, filter: OfferFilter) -> Result<OfferResponse, String> {
        let url = format!("{}/\"GetInfoOferteClienti\"", self.config.base_url);
        
        info!("Fetching offers from API: {}", url);

        let response = self.client
            .post(&url)
            .json(&filter)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch offers: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API returned error status: {}", response.status()));
        }

        let offer_response: OfferResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse offer response: {}", e))?;

        info!("Successfully fetched {} offers", offer_response.info_oferte.len());

        Ok(offer_response)
    }

    // Send invoice to WME
    pub async fn send_invoice_to_wme(&self, request: WmeInvoiceRequest) -> Result<WmeInvoiceResponse, String> {
        let url = format!("{}/IesiriClienti", self.config.base_url);
        
        info!("Sending invoice to WME API: {}", url);

        // Serialize request to JSON for debugging
        if let Ok(json_body) = serde_json::to_string_pretty(&request) {
             info!("Request Payload:\n{}", json_body);
        }

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Failed to send invoice: {}", e))?;

        let status = response.status();
        let body = response.text().await.map_err(|e| format!("Failed to read response body: {}", e))?;

        info!("Response Status: {}", status);
        info!("Response Body: {}", body);

        if !status.is_success() {
             return Err(format!("API returned error status: {}. Body: {}", status, body));
        }

        let wme_response: WmeInvoiceResponse = serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse invoice response: {}", e))?;

        info!("Successfully sent invoice to WME");

        Ok(wme_response)
    }
}
