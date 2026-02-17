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
  ClientBalance,
  Collection,
  CreateCollectionGroupRequest,
  SalesReportItem,
  SalesPrintItem,
  SalesProductReportItem,
  CollectionsReportItem,
} from "./types";

// ==================== SYNC COMMANDS ====================

export async function clearDatabase(): Promise<void> {
  return invoke<void>("clear_database");
}

export async function deletePartnersAndLocations(): Promise<string> {
  return invoke<string>("delete_partners_and_locations");
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

export async function previewInvoiceCertificate(invoiceId: string): Promise<string> {
  return invoke<string>("preview_invoice_certificate", { invoiceId });
}

export async function printCollectionToHtml(collectionId: string, printerName?: string): Promise<string> {
  return invoke<string>("print_collection_to_html", { collectionId, printerName });
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
  tipContabil: string | null,
  codCarnet: string | null,
  codCarnetLivr: string | null,
  codDelegat: string | null,
  delegateName: string | null,
  delegateAct: string | null,
  carNumber: string | null,
  invoiceNumberStart: number | null,
  invoiceNumberEnd: number | null,
  invoiceNumberCurrent: number | null,
  marcaAgent: string | null,
  numeCasa: string | null,
  autoSyncCollectionsEnabled: boolean | null,
  autoSyncCollectionsTime: string | null
): Promise<AgentSettings> {
  return invoke<AgentSettings>("save_agent_settings", {
    agentName,
    carnetSeries,
    simbolCarnetLivr,
    simbolGestiuneLivrare,
    tipContabil,
    codCarnet,
    codCarnetLivr,
    codDelegat,
    delegateName,
    delegateAct,
    carNumber,
    invoiceNumberStart,
    invoiceNumberEnd,
    invoiceNumberCurrent,
    marcaAgent,
    numeCasa,
    autoSyncCollectionsEnabled,
    autoSyncCollectionsTime,
  });
}

// ==================== COLLECTION COMMANDS ====================

export async function syncClientBalances(): Promise<string> {
  return invoke<string>("sync_client_balances");
}

export async function getClientBalances(partnerId?: string): Promise<ClientBalance[]> {
  return invoke<ClientBalance[]>("get_client_balances", { partnerId });
}

export async function recordCollection(collection: Collection): Promise<string> {
  return invoke<string>("record_collection", { collection });
}

export async function recordCollectionGroup(request: CreateCollectionGroupRequest): Promise<string> {
  return invoke<string>("record_collection_group", { request });
}

export async function recordCollectionFromInvoice(invoiceId: string, paidAmount: number): Promise<string> {
  return invoke<string>("record_collection_from_invoice", { invoiceId, paidAmount });
}

export async function getCollections(statusFilter?: string): Promise<Collection[]> {
  return invoke<Collection[]>("get_collections", { statusFilter });
}

export async function syncCollections(): Promise<SyncStatus> {
  return invoke<SyncStatus>("sync_collections");
}

export async function sendCollection(collectionId: string): Promise<Collection> {
  return invoke<Collection>("send_collection", { collectionId });
}

export async function deleteCollection(collectionId: string): Promise<void> {
  return invoke<void>("delete_collection", { collectionId });
}

export async function getSalesReport(startDate?: string, endDate?: string): Promise<SalesReportItem[]> {
  return invoke<SalesReportItem[]>("get_sales_report", { startDate, endDate });
}

export async function getSalesPrintReport(startDate?: string, endDate?: string): Promise<SalesPrintItem[]> {
  return invoke<SalesPrintItem[]>("get_sales_print_report", { startDate, endDate });
}

export async function getSalesProductsReport(startDate?: string, endDate?: string): Promise<SalesProductReportItem[]> {
  return invoke<SalesProductReportItem[]>("get_sales_products_report", { startDate, endDate });
}

export async function getCollectionsReport(startDate?: string, endDate?: string): Promise<CollectionsReportItem[]> {
  return invoke<CollectionsReportItem[]>("get_collections_report", { startDate, endDate });
}

export async function printDailyReport(date?: string, printerName?: string): Promise<string> {
  return invoke<string>("print_daily_report", { date, printerName });
}

export async function saveReportHtml(reportName: string, htmlContent: string): Promise<string> {
  return invoke<string>("save_report_html", { reportName, htmlContent });
}

export async function printReportHtml(reportName: string, htmlContent: string, printerName?: string): Promise<string> {
  return invoke<string>("print_report_html", { reportName, htmlContent, printerName });
}

// ==================== DEBUG COMMANDS ====================

export async function debugDbCounts(): Promise<string> {
  return invoke<string>("debug_db_counts");
}

export async function openExternalLink(url: string): Promise<void> {
  return invoke<void>("open_external_link", { url });
}
