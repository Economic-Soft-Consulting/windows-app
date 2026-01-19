import { invoke } from "@tauri-apps/api/core";
import type {
  PartnerWithLocations,
  Product,
  Invoice,
  InvoiceDetail,
  CreateInvoiceRequest,
  SyncStatus,
  InvoiceStatus,
  AgentSettings,
} from "./types";

// ==================== SYNC COMMANDS ====================

export async function clearDatabase(): Promise<void> {
  return invoke<void>("clear_database");
}

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

export async function getProducts(partnerId?: string): Promise<Product[]> {
  return invoke<Product[]>("get_products", { partnerId });
}

export async function searchProducts(query: string, partnerId?: string): Promise<Product[]> {
  return invoke<Product[]>("search_products", { query, partnerId });
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

export async function previewInvoiceJson(invoiceId: string): Promise<string> {
  return invoke<string>("preview_invoice_json", { invoiceId });
}

export async function sendAllPendingInvoices(): Promise<string[]> {
  return invoke<string[]>("send_all_pending_invoices");
}

export async function cancelInvoiceSending(invoiceId: string): Promise<Invoice> {
  return invoke<Invoice>("cancel_invoice_sending", { invoiceId });
}

export async function deleteInvoice(invoiceId: string): Promise<void> {
  return invoke<void>("delete_invoice", { invoiceId });
}

// ==================== PRINT COMMANDS ====================

export async function getAvailablePrinters(): Promise<string[]> {
  return invoke<string[]>("get_available_printers");
}

export async function printInvoiceToHtml(invoiceId: string, printerName?: string): Promise<string> {
  return invoke<string>("print_invoice_to_html", { invoiceId, printerName });
}

// ==================== AGENT SETTINGS COMMANDS ====================

export async function getAgentSettings(): Promise<AgentSettings> {
  return invoke<AgentSettings>("get_agent_settings");
}

export async function saveAgentSettings(
  agentName: string | null,
  carnetSeries: string | null,
  simbolCarnetLivr: string | null,
  simbolGestiuneLivrare: string | null,
  codCarnet: string | null,
  codCarnetLivr: string | null
): Promise<AgentSettings> {
  return invoke<AgentSettings>("save_agent_settings", {
    agentName,
    carnetSeries,
    simbolCarnetLivr,
    simbolGestiuneLivrare,
    codCarnet,
    codCarnetLivr,
  });
}

// ==================== DEBUG COMMANDS ====================

export async function debugDbCounts(): Promise<string> {
  return invoke<string>("debug_db_counts");
}
