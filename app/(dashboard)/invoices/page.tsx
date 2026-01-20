"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import { Plus, FileText, Loader2, LayoutGrid, Table as TableIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { InvoiceCard } from "@/app/components/invoices/InvoiceCard";
import { InvoiceDetailDialog } from "@/app/components/invoices/InvoiceDetailDialog";
import { useInvoices } from "@/hooks/useInvoices";
import type { InvoiceStatus } from "@/lib/tauri/types";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { InvoiceStatusBadge } from "@/app/components/invoices/InvoiceStatusBadge";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  DropdownMenuSeparator,
} from "@/components/ui/dropdown-menu";
import { Eye, Send, MoreHorizontal, Trash2 } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { useAuth } from "@/app/contexts/AuthContext";

type TabValue = "all" | InvoiceStatus;
type ViewMode = "grid" | "table";

function formatCurrency(amount: number): string {
  return new Intl.NumberFormat("ro-RO", {
    style: "decimal",
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(amount) + " RON";
}

function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleDateString("ro-RO", {
    day: "2-digit",
    month: "2-digit",
    year: "numeric",
  });
}

function formatTime(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleTimeString("ro-RO", {
    hour: "2-digit",
    minute: "2-digit",
  });
}

export default function InvoicesPage() {
  const [activeTab, setActiveTab] = useState<TabValue>("all");
  const [selectedInvoiceId, setSelectedInvoiceId] = useState<string | null>(null);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [viewMode, setViewMode] = useState<ViewMode>("table");
  const { isAdmin } = useAuth(); // Get admin status

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

      {/* Tabs with Button and View Toggle */}
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

        <div className="flex gap-2 w-full sm:w-auto">
          <div className="flex border rounded-lg p-1">
            <Button
              variant={viewMode === "table" ? "secondary" : "ghost"}
              size="sm"
              onClick={() => setViewMode("table")}
              className="h-9 px-3"
            >
              <TableIcon className="h-4 w-4" />
            </Button>
            <Button
              variant={viewMode === "grid" ? "secondary" : "ghost"}
              size="sm"
              onClick={() => setViewMode("grid")}
              className="h-9 px-3"
            >
              <LayoutGrid className="h-4 w-4" />
            </Button>
          </div>

          <Link href="/invoices/new" className="flex-1 sm:flex-none">
            <Button size="lg" className="gap-2 h-12 px-6 w-full sm:w-auto">
              <Plus className="h-5 w-5" />
              Factură nouă
            </Button>
          </Link>
        </div>
      </div>

      {/* Invoice Display */}
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
      ) : viewMode === "table" ? (
        <div className="border rounded-lg overflow-hidden">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-[180px]">Partener</TableHead>
                <TableHead className="w-[150px]">Locație</TableHead>
                <TableHead className="w-[100px]">Data</TableHead>
                <TableHead className="w-[70px]">Ora</TableHead>
                <TableHead className="text-right w-[120px]">Valoare</TableHead>
                <TableHead className="w-[100px]">Status</TableHead>
                <TableHead className="text-right w-[80px]">Acțiuni</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {invoices.map((invoice) => (
                <TableRow key={invoice.id} className="cursor-pointer hover:bg-muted/50">
                  <TableCell className="font-medium">{invoice.partner_name}</TableCell>
                  <TableCell className="text-sm text-muted-foreground truncate max-w-[150px]">
                    {invoice.location_name}
                  </TableCell>
                  <TableCell className="text-sm">{formatDate(invoice.created_at)}</TableCell>
                  <TableCell className="text-sm text-muted-foreground">{formatTime(invoice.created_at)}</TableCell>
                  <TableCell className="text-right font-medium">{formatCurrency(invoice.total_amount)}</TableCell>
                  <TableCell>
                    <InvoiceStatusBadge status={invoice.status} />
                  </TableCell>
                  <TableCell className="text-right">
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button variant="ghost" size="sm" className="h-8 w-8 p-0">
                          <MoreHorizontal className="h-4 w-4" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        <DropdownMenuItem onClick={() => handleView(invoice.id)}>
                          <Eye className="mr-2 h-4 w-4" />
                          Detalii
                        </DropdownMenuItem>
                        {(invoice.status === "pending" || invoice.status === "failed") && (
                          <DropdownMenuItem onClick={() => handleSend(invoice.id)}>
                            <Send className="mr-2 h-4 w-4" />
                            Trimite
                          </DropdownMenuItem>
                        )}
                        {isAdmin && (
                          <>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem
                              onClick={() => handleDelete(invoice.id)}
                              className="text-red-600"
                            >
                              <Trash2 className="mr-2 h-4 w-4" />
                              Șterge
                            </DropdownMenuItem>
                          </>
                        )}
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      ) : (
        <div className="grid gap-3" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))" }}>
          {invoices.map((invoice) => (
            <InvoiceCard
              key={invoice.id}
              invoice={invoice}
              onSend={handleSend}
              onDelete={handleDelete}
              onView={handleView}
              onCancel={handleCancel}
              isAdmin={isAdmin}
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
