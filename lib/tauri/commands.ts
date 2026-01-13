import { invoke } from "@tauri-apps/api/core";
import type {
  PartnerWithLocations,
  Product,
  Invoice,
  InvoiceDetail,
  CreateInvoiceRequest,
  SyncStatus,
  InvoiceStatus,
} from "./types";

// ==================== SYNC COMMANDS ====================

export async function checkFirstRun(): Promise<boolean> {
  return invoke<boolean>("check_first_run");
}

export async function getSyncStatus(): Promise<SyncStatus> {
  return invoke<SyncStatus>("get_sync_status");
}

export async function syncAllData(): Promise<SyncStatus> {
  return invoke<SyncStatus>("sync_all_data");
}

export async function checkOnlineStatus(): Promise<boolean> {
  return invoke<boolean>("check_online_status");
}

// ==================== PARTNER COMMANDS ====================

export async function getPartners(): Promise<PartnerWithLocations[]> {
  return invoke<PartnerWithLocations[]>("get_partners");
}

export async function searchPartners(
  query: string
): Promise<PartnerWithLocations[]> {
  return invoke<PartnerWithLocations[]>("search_partners", { query });
}

// ==================== PRODUCT COMMANDS ====================

export async function getProducts(): Promise<Product[]> {
  return invoke<Product[]>("get_products");
}

export async function searchProducts(query: string): Promise<Product[]> {
  return invoke<Product[]>("search_products", { query });
}

// ==================== INVOICE COMMANDS ====================

export async function createInvoice(
  request: CreateInvoiceRequest
): Promise<Invoice> {
  return invoke<Invoice>("create_invoice", { request });
}

export async function getInvoices(
  statusFilter?: InvoiceStatus
): Promise<Invoice[]> {
  return invoke<Invoice[]>("get_invoices", { statusFilter });
}

export async function getInvoiceDetail(
  invoiceId: string
): Promise<InvoiceDetail> {
  return invoke<InvoiceDetail>("get_invoice_detail", { invoiceId });
}

export async function sendInvoice(invoiceId: string): Promise<Invoice> {
  return invoke<Invoice>("send_invoice", { invoiceId });
}

export async function deleteInvoice(invoiceId: string): Promise<void> {
  return invoke<void>("delete_invoice", { invoiceId });
}
