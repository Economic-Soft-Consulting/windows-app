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
import { InvoiceStatusBadge } from "./InvoiceStatusBadge";
import { useInvoiceDetail } from "@/hooks/useInvoices";
import { Skeleton } from "@/components/ui/skeleton";
import { MapPin, Calendar, FileText } from "lucide-react";

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

  return (
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
            <div className="border rounded-lg overflow-hidden">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Produs</TableHead>
                    <TableHead className="text-right w-24">Cantitate</TableHead>
                    <TableHead className="text-right w-28">Preț unitar</TableHead>
                    <TableHead className="text-right w-28">Total</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {detail.items.map((item) => (
                    <TableRow key={item.id}>
                      <TableCell className="font-medium">{item.product_name}</TableCell>
                      <TableCell className="text-right">
                        {item.quantity} {item.unit_of_measure}
                      </TableCell>
                      <TableCell className="text-right">
                        {formatCurrency(item.unit_price)}
                      </TableCell>
                      <TableCell className="text-right font-medium">
                        {formatCurrency(item.total_price)}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
                <TableFooter>
                  <TableRow>
                    <TableCell colSpan={3} className="text-right font-semibold">
                      Total
                    </TableCell>
                    <TableCell className="text-right font-bold text-lg">
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
