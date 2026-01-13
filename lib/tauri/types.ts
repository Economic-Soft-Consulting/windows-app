// Types matching the Rust backend models

export interface Partner {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export interface Location {
  id: string;
  partner_id: string;
  name: string;
  address: string | null;
}

export interface PartnerWithLocations {
  id: string;
  name: string;
  created_at: string;
  updated_at: string;
  locations: Location[];
}

export interface Product {
  id: string;
  name: string;
  unit_of_measure: string;
  price: number;
  class: string | null;
}

export type InvoiceStatus = "pending" | "sending" | "sent" | "failed";

export interface Invoice {
  id: string;
  partner_id: string;
  partner_name: string;
  location_id: string;
  location_name: string;
  status: InvoiceStatus;
  total_amount: number;
  item_count: number;
  notes: string | null;
  created_at: string;
  sent_at: string | null;
  error_message: string | null;
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

// Cart item for invoice creation wizard
export interface CartItem {
  product: Product;
  quantity: number;
}
