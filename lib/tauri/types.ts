// Types matching the Rust backend models

export interface Partner {
  id: string;
  name: string;
  cif?: string;
  reg_com?: string;
  cod?: string;
  blocat?: string;
  tva_la_incasare?: string;
  persoana_fizica?: string;
  cod_extern?: string;
  cod_intern?: string;
  observatii?: string;
  data_adaugarii?: string;
  clasa?: string;
  simbol_clasa?: string;
  cod_clasa?: string;
  inactiv?: string;
  categorie_pret_implicita?: string;
  simbol_categorie_pret?: string;
  scadenta_la_vanzare?: string;
  scadenta_la_cumparare?: string;
  credit_client?: string;
  discount_fix?: string;
  tip_partener?: string;
  mod_aplicare_discount?: string;
  moneda?: string;
  data_nastere?: string;
  created_at: string;
  updated_at: string;
}

export interface Location {
  id: string;
  partner_id: string;
  name: string;
  address: string | null;
  cod_sediu?: string;
  localitate?: string;
  strada?: string;
  numar?: string;
  bloc?: string; // Not in backend explicitly but handled in UI
  judet?: string;
  tara?: string;
  cod_postal?: string;
  telefon?: string;
  email?: string;
  inactiv?: string;
}

export interface PartnerWithLocations extends Partner {
  locations: Location[];
}

export interface Product {
  id: string;
  name: string;
  unit_of_measure: string;
  price: number;
  class: string | null;
  tva_percent: number | null;
}

export type InvoiceStatus = "pending" | "sending" | "sent" | "failed";

export interface Invoice {
  id: string;
  partner_id: string;
  partner_name: string;
  partner_cif?: string;
  partner_reg_com?: string;
  location_id: string;
  location_name: string;
  location_address?: string | null;
  status: InvoiceStatus;
  total_amount: number;
  item_count: number;
  notes: string | null;
  created_at: string;
  sent_at: string | null;
  error_message: string | null;
  partner_payment_term: string | null;
}

export interface InvoiceItem {
  id: string;
  invoice_id: string;
  product_id: string;
  product_name: string;
  quantity: number;
  unit_price: number;
  unit_of_measure: string;
  total_price: number;
  tva_percent: number | null;
}

export interface CreateInvoiceRequest {
  partner_id: string;
  location_id: string;
  notes?: string;
  items: CreateInvoiceItemRequest[];
}

export interface CreateInvoiceItemRequest {
  product_id: string;
  quantity: number;
}

export interface InvoiceDetail {
  invoice: Invoice;
  items: InvoiceItem[];
}

export interface SyncStatus {
  is_first_run: boolean;
  partners_synced_at: string | null;
  products_synced_at: string | null;
  is_syncing: boolean;
}

export interface AgentSettings {
  agent_name: string | null;
  carnet_series: string | null;
  simbol_carnet_livr: string | null;
  simbol_gestiune_livrare: string | null;
  cod_carnet: string | null;
  cod_carnet_livr: string | null;
  delegate_name: string | null;
  delegate_act: string | null;
  car_number: string | null;
  invoice_number_start: number | null;
  invoice_number_end: number | null;
  invoice_number_current: number | null;
}

// Cart item for invoice creation wizard
export interface CartItem {
  product: Product;
  quantity: number;
}
