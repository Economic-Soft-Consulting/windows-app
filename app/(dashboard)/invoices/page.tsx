"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import { Plus, FileText, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { InvoiceCard } from "@/app/components/invoices/InvoiceCard";
import { InvoiceDetailDialog } from "@/app/components/invoices/InvoiceDetailDialog";
import { useInvoices } from "@/hooks/useInvoices";
import type { InvoiceStatus } from "@/lib/tauri/types";

type TabValue = "all" | InvoiceStatus;

export default function InvoicesPage() {
  const [activeTab, setActiveTab] = useState<TabValue>("all");
  const [selectedInvoiceId, setSelectedInvoiceId] = useState<string | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);

  const statusFilter = activeTab === "all" ? undefined : activeTab;
  const { invoices, isLoading, send, remove, refresh } = useInvoices(statusFilter);

  // Listen for auto-send updates
  useEffect(() => {
    const handleInvoicesUpdated = () => {
      // Refresh will happen automatically via useInvoices hook
      window.location.reload();
    };

    window.addEventListener('invoices-updated', handleInvoicesUpdated);
    return () => window.removeEventListener('invoices-updated', handleInvoicesUpdated);
  }, []);

  const handleView = (id: string) => {
    setSelectedInvoiceId(id);
    setDialogOpen(true);
  };

  const handleSend = async (id: string) => {
    await send(id);
  };

  const handleDelete = async (id: string) => {
    await remove(id);
  };

  const handleCancel = async (id: string) => {
    // Refresh after cancel to show updated status
    await refresh();
  };

  // Count invoices by status (for badge numbers)
  const allInvoices = useInvoices();
  const counts = {
    all: allInvoices.invoices.length,
    pending: allInvoices.invoices.filter((i) => i.status === "pending").length,
    sent: allInvoices.invoices.filter((i) => i.status === "sent").length,
    failed: allInvoices.invoices.filter((i) => i.status === "failed").length,
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold">Facturi</h1>
        <p className="text-muted-foreground">
          Gestionează și trimite facturile către parteneri
        </p>
      </div>

      {/* Tabs with Button */}
      <div className="flex flex-col sm:flex-row gap-4 items-start sm:items-center justify-between">
      <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as TabValue)} className="flex-1">
        <TabsList className="h-14 flex-wrap sm:flex-nowrap">
          <TabsTrigger value="all" className="h-11 px-3 sm:px-4 gap-1.5 sm:gap-2">
            Toate
            <span className="bg-muted text-muted-foreground px-2 py-0.5 rounded-full text-xs">
              {counts.all}
            </span>
          </TabsTrigger>
          <TabsTrigger value="pending" className="h-11 px-3 sm:px-4 gap-1.5 sm:gap-2">
            <span className="hidden sm:inline">În așteptare</span>
            <span className="sm:hidden">Așteptare</span>
            {counts.pending > 0 && (
              <span className="bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-400 px-2 py-0.5 rounded-full text-xs">
                {counts.pending}
              </span>
            )}
          </TabsTrigger>
          <TabsTrigger value="sent" className="h-11 px-3 sm:px-4 gap-1.5 sm:gap-2">
            Trimise
            {counts.sent > 0 && (
              <span className="bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-400 px-2 py-0.5 rounded-full text-xs">
                {counts.sent}
              </span>
            )}
          </TabsTrigger>
          <TabsTrigger value="failed" className="h-11 px-3 sm:px-4 gap-1.5 sm:gap-2">
            Eșuate
            {counts.failed > 0 && (
              <span className="bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-400 px-2 py-0.5 rounded-full text-xs">
                {counts.failed}
              </span>
            )}
          </TabsTrigger>
        </TabsList>
      </Tabs>
      
      <Link href="/invoices/new">
        <Button size="lg" className="gap-2 h-12 px-6 w-full sm:w-auto">
          <Plus className="h-5 w-5" />
          Factură nouă
        </Button>
      </Link>
      </div>

      {/* Invoice Grid */}
      {isLoading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </div>
      ) : invoices.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-16 text-center">
          <FileText className="h-16 w-16 text-muted-foreground/50 mb-4" />
          <h3 className="text-lg font-medium">Nu există facturi</h3>
          <p className="text-muted-foreground mt-1 mb-6">
            {activeTab === "all"
              ? "Creează prima ta factură pentru a începe"
              : `Nu există facturi cu statusul "${activeTab}"`}
          </p>
          {activeTab === "all" && (
            <Link href="/invoices/new">
              <Button className="gap-2">
                <Plus className="h-4 w-4" />
                Creează factură
              </Button>
            </Link>
          )}
        </div>
      ) : (
        <div className="grid gap-3 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
          {invoices.map((invoice) => (
            <InvoiceCard
              key={invoice.id}
              invoice={invoice}
              onSend={handleSend}
              onDelete={handleDelete}
              onView={handleView}
              onCancel={handleCancel}
            />
          ))}
        </div>
      )}

      {/* Invoice Detail Dialog */}
      <InvoiceDetailDialog
        invoiceId={selectedInvoiceId}
        open={dialogOpen}
        onOpenChange={setDialogOpen}
      />
    </div>
  );
}
