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
                simbol_clasa: None,
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
