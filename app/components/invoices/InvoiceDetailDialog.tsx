"use client";

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  TableFooter,
} from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { InvoiceStatusBadge } from "./InvoiceStatusBadge";
import { useInvoiceDetail } from "@/hooks/useInvoices";
import { Skeleton } from "@/components/ui/skeleton";
import { MapPin, Calendar, FileText, Printer } from "lucide-react";
import { printInvoiceToHtml } from "@/lib/tauri/commands";
import { toast } from "sonner";
import { useState } from "react";

interface InvoiceDetailDialogProps {
  invoiceId: string | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

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
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function InvoiceDetailDialog({
  invoiceId,
  open,
  onOpenChange,
}: InvoiceDetailDialogProps) {
  const { detail, isLoading } = useInvoiceDetail(open ? invoiceId : null);
  const [isPrinting, setIsPrinting] = useState(false);

  const handlePrint = async () => {
    if (!invoiceId) return;
    
    setIsPrinting(true);
    try {
      const html = await printInvoiceToHtml(invoiceId);
      
      // Open print dialog with the HTML
      const printWindow = window.open("", "_blank");
      if (printWindow) {
        printWindow.document.write(html);
        printWindow.document.close();
        printWindow.focus();
        // Trigger print dialog after a brief delay to ensure content is rendered
        setTimeout(() => {
          printWindow.print();
        }, 250);
      }
      
      toast.success("Factură pregătită pentru imprimare");
    } catch (error) {
      console.error("Print error:", error);
      toast.error("Eroare la generarea facturilor pentru imprimare");
    } finally {
      setIsPrinting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <div className="flex items-center justify-between gap-3">
            <DialogTitle className="flex items-center gap-3">
              <FileText className="h-5 w-5" />
              Detalii factură
            </DialogTitle>
            {detail && detail.invoice.status === "sent" && (
              <Button
                variant="outline"
                size="sm"
                onClick={handlePrint}
                disabled={isPrinting}
                className="gap-2"
              >
                <Printer className="h-4 w-4" />
                {isPrinting ? "Se imprimă..." : "Imprimare"}
              </Button>
            )}
          </div>
        </DialogHeader>

        {isLoading ? (
          <div className="space-y-4">
            <Skeleton className="h-6 w-48" />
            <Skeleton className="h-4 w-32" />
            <Skeleton className="h-32 w-full" />
          </div>
        ) : detail ? (
          <div className="space-y-6">
            {/* Invoice Header */}
            <div className="flex items-start justify-between">
              <div>
                <h3 className="text-lg font-semibold">{detail.invoice.partner_name}</h3>
                <div className="flex items-center gap-1.5 text-sm text-muted-foreground mt-1">
                  <MapPin className="h-3.5 w-3.5" />
                  {detail.invoice.location_name}
                </div>
                <div className="flex items-center gap-1.5 text-sm text-muted-foreground mt-1">
                  <Calendar className="h-3.5 w-3.5" />
                  {formatDate(detail.invoice.created_at)}
                </div>
              </div>
              <InvoiceStatusBadge status={detail.invoice.status} />
            </div>

            {/* Notes */}
            {detail.invoice.notes && (
              <div className="bg-muted/50 p-3 rounded-lg">
                <p className="text-sm font-medium mb-1">Note:</p>
                <p className="text-sm text-muted-foreground">{detail.invoice.notes}</p>
              </div>
            )}

            {/* Error Message */}
            {detail.invoice.status === "failed" && detail.invoice.error_message && (
              <div className="bg-red-50 dark:bg-red-900/20 p-3 rounded-lg border border-red-200 dark:border-red-800">
                <p className="text-sm font-medium text-red-800 dark:text-red-400">
                  Eroare: {detail.invoice.error_message}
                </p>
              </div>
            )}

            {/* Items Table */}
            <div className="border rounded-lg overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="min-w-0">Produs</TableHead>
                    <TableHead className="text-right whitespace-nowrap">Cantitate</TableHead>
                    <TableHead className="text-right whitespace-nowrap hidden sm:table-cell">Preț unitar</TableHead>
                    <TableHead className="text-right whitespace-nowrap">Total</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {detail.items.map((item) => (
                    <TableRow key={item.id}>
                      <TableCell className="font-medium min-w-0">
                        <span className="line-clamp-2">{item.product_name}</span>
                      </TableCell>
                      <TableCell className="text-right whitespace-nowrap">
                        {item.quantity} {item.unit_of_measure}
                      </TableCell>
                      <TableCell className="text-right whitespace-nowrap hidden sm:table-cell">
                        {formatCurrency(item.unit_price)}
                      </TableCell>
                      <TableCell className="text-right font-medium whitespace-nowrap">
                        {formatCurrency(item.total_price)}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
                <TableFooter>
                  <TableRow>
                    <TableCell colSpan={2} className="text-right font-semibold sm:hidden">
                      Total
                    </TableCell>
                    <TableCell colSpan={3} className="text-right font-semibold hidden sm:table-cell">
                      Total
                    </TableCell>
                    <TableCell className="text-right font-bold text-lg whitespace-nowrap">
                      {formatCurrency(detail.invoice.total_amount)}
                    </TableCell>
                  </TableRow>
                </TableFooter>
              </Table>
            </div>

            {/* Sent timestamp */}
            {detail.invoice.sent_at && (
              <div className="text-sm text-green-600 dark:text-green-400">
                Trimisă cu succes: {formatDate(detail.invoice.sent_at)}
              </div>
            )}
          </div>
        ) : (
          <div className="text-center py-8 text-muted-foreground">
            Nu s-au găsit detalii pentru această factură
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
