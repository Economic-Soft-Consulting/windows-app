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
import { previewInvoiceCertificate, printInvoiceCertificate } from "@/lib/tauri/commands";
import { InvoiceStatusBadge } from "./InvoiceStatusBadge";
import { toast } from "sonner";
import { useState } from "react";
import { useInvoiceDetail } from "@/hooks/useInvoices";
import { usePrintInvoice } from "@/hooks/usePrintInvoice";
import { Skeleton } from "@/components/ui/skeleton";
import { MapPin, FileText, Printer } from "lucide-react";
import { formatCurrency, formatDateTime, formatDate, calculateDueDate } from "@/lib/utils";

interface InvoiceDetailDialogProps {
  invoiceId: string | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function InvoiceDetailDialog({
  invoiceId,
  open,
  onOpenChange,
}: InvoiceDetailDialogProps) {
  const { detail, isLoading } = useInvoiceDetail(open ? invoiceId : null);
  const { printInvoice, isPrinting, receiptDialog } = usePrintInvoice();
  const [isPrintingCert, setIsPrintingCert] = useState(false);

  const handlePrintCertificate = async () => {
    if (!invoiceId) return;
    setIsPrintingCert(true);
    try {
      const selectedPrinter = typeof window !== "undefined" ? localStorage.getItem("selectedPrinter") : null;
      await printInvoiceCertificate(invoiceId, selectedPrinter || undefined);
      toast.success("Certificat trimis la imprimantă!");
    } catch (error) {
      console.error("Certificate print error:", error);
      toast.error("Eroare la printare certificat: " + error);
    } finally {
      setIsPrintingCert(false);
    }
  };

  const handlePreviewCertificate = async () => {
    try {
      if (!invoiceId) return;
      await previewInvoiceCertificate(invoiceId);
    } catch (error) {
      if (typeof window !== "undefined") {
        console.error("Failed to preview certificate:", error);
      }
    }
  };

  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-3">
              <FileText className="h-5 w-5" />
              Detalii factură
            </DialogTitle>
          </DialogHeader>

          {isLoading ? (
            <div className="space-y-4">
              <Skeleton className="h-6 w-48" />
              <Skeleton className="h-4 w-32" />
              <Skeleton className="h-32 w-full" />
            </div>
          ) : detail ? (
            <div className="space-y-6">
              <div className="flex items-start justify-between">
                <div className="space-y-2">
                  <h3 className="text-lg font-semibold">{detail.invoice.partner_name}</h3>
                  <div className="flex items-center gap-1.5 text-sm text-muted-foreground">
                    <MapPin className="h-3.5 w-3.5" />
                    {detail.invoice.location_name}
                  </div>
                  <div className="grid grid-cols-2 gap-3 mt-2">
                    <div className="bg-muted/50 p-2 rounded">
                      <p className="text-xs text-muted-foreground">Data emitere</p>
                      <p className="text-sm font-medium">{formatDate(detail.invoice.created_at)}</p>
                    </div>
                    <div className="bg-muted/50 p-2 rounded">
                      <p className="text-xs text-muted-foreground">Data scadență</p>
                      <p className="text-sm font-medium">
                        {(() => {
                          // Parse payment term from partner, default to 7
                          const paymentTerm = detail.invoice.partner_payment_term
                            ? parseInt(detail.invoice.partner_payment_term) || 7
                            : 7;
                          return calculateDueDate(detail.invoice.created_at, paymentTerm);
                        })()}
                      </p>
                    </div>
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
                      <TableHead className="text-center whitespace-nowrap hidden sm:table-cell">Preț</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {detail.items.map((item) => {
                      // Use actual TVA from database, default to 19% if not available
                      const vatPercent = item.tva_percent || 19;
                      const vatAmount = item.total_price * (vatPercent / 100);
                      const totalWithVAT = item.total_price + vatAmount;

                      return (
                        <TableRow key={item.id}>
                          <TableCell className="font-medium min-w-0">
                            <span className="line-clamp-2">{item.product_name}</span>
                          </TableCell>
                          <TableCell className="text-right whitespace-nowrap">
                            {item.quantity} {item.unit_of_measure}
                          </TableCell>
                          <TableCell className="text-center whitespace-nowrap hidden sm:table-cell">
                            <div className="space-y-1">
                              <div className="flex items-center justify-between gap-3">
                                <span className="text-xs text-muted-foreground">{formatCurrency(item.total_price)}</span>
                                <span className="text-[10px] text-muted-foreground">Preț fără TVA</span>
                              </div>
                              <div className="flex items-center justify-between gap-3">
                                <span className="text-xs font-medium text-primary">{formatCurrency(vatAmount)}</span>
                                <span className="text-[10px] font-medium text-primary">TVA {vatPercent}%</span>
                              </div>
                              <div className="flex items-center justify-between gap-3 border-t pt-1">
                                <span className="text-sm font-semibold">{formatCurrency(totalWithVAT)}</span>
                                <span className="text-[10px] font-medium">Preț cu TVA</span>
                              </div>
                            </div>
                          </TableCell>
                        </TableRow>
                      )
                    })}
                  </TableBody>
                  <TableFooter>
                    <TableRow>
                      <TableCell colSpan={2} className="text-right font-semibold">
                        Total general cu TVA
                      </TableCell>
                      <TableCell className="text-right font-bold text-lg whitespace-nowrap">
                        {(() => {
                          // Calculate total VAT across all items
                          const totalVAT = detail.items.reduce((sum, item) => {
                            const vatPercent = item.tva_percent || 19;
                            return sum + (item.total_price * vatPercent / 100);
                          }, 0);
                          const grandTotal = detail.invoice.total_amount + totalVAT;

                          return (
                            <>
                              <div className="text-xs text-muted-foreground font-normal">{formatCurrency(detail.invoice.total_amount)}</div>
                              <div>{formatCurrency(grandTotal)}</div>
                            </>
                          );
                        })()}
                      </TableCell>
                    </TableRow>
                  </TableFooter>
                </Table>
              </div>

              {/* Sent timestamp */}
              {detail.invoice.sent_at && (
                <div className="text-sm text-green-600 dark:text-green-400">
                  Trimisă cu succes: {formatDateTime(detail.invoice.sent_at)}
                </div>
              )}

              {/* Print Button */}
              <div className="flex justify-end gap-2 pt-4 border-t">
                <Button
                  onClick={handlePrintCertificate}
                  disabled={isPrintingCert}
                  variant="outline"
                  className="gap-2"
                >
                  <Printer className="h-4 w-4" />
                  {isPrintingCert ? "Se printează..." : "Printează Certificat"}
                </Button>
                <Button
                  onClick={handlePreviewCertificate}
                  variant="outline"
                >
                  Vezi certificat
                </Button>
                <Button
                  onClick={() => invoiceId && printInvoice(invoiceId)}
                  disabled={isPrinting}
                  variant="outline"
                  className="gap-2"
                >
                  <Printer className="h-4 w-4" />
                  {isPrinting ? "Se printează..." : "Printează Factura"}
                </Button>
              </div>
            </div>
          ) : (
            <div className="text-center py-8 text-muted-foreground">
              Nu s-au găsit detalii pentru această factură
            </div>
          )}
        </DialogContent>
      </Dialog>
      {receiptDialog}
    </>
  );
}
