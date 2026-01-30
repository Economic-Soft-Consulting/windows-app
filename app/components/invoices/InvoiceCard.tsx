"use client";

import { useState } from "react";
import { Card, CardContent, CardFooter, CardHeader } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { InvoiceStatusBadge } from "./InvoiceStatusBadge";
import { Send, Trash2, Eye, RotateCcw, MapPin, Package, Printer, Loader2, XCircle } from "lucide-react";
import type { Invoice } from "@/lib/tauri/types";
import { printInvoiceToHtml, cancelInvoiceSending } from "@/lib/tauri/commands";
import { toast } from "sonner";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";

interface InvoiceCardProps {
  invoice: Invoice;
  onSend: (id: string) => void;
  onDelete: (id: string) => void;
  onView: (id: string) => void;
  onCancel?: (id: string) => void;
  isAdmin?: boolean; // Add admin check
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

export function InvoiceCard({ invoice, onSend, onDelete, onView, onCancel, isAdmin = true }: InvoiceCardProps) {
  const [isPrinting, setIsPrinting] = useState(false);
  const [isCancelling, setIsCancelling] = useState(false);
  const canSend = invoice.status === "pending" || invoice.status === "failed";
  const canDelete = isAdmin && (invoice.status === "pending" || invoice.status === "failed"); // Only admin can delete
  const isSending = invoice.status === "sending";

  const handleCancel = async () => {
    if (!onCancel) return;
    setIsCancelling(true);
    try {
      await cancelInvoiceSending(invoice.id);
      onCancel(invoice.id);
      toast.success("Trimitere anulată!");
    } catch (error) {
      console.error("Cancel error:", error);
      toast.error(`Eroare la anulare: ${error}`);
    } finally {
      setIsCancelling(false);
    }
  };

  const handlePrint = async () => {
    setIsPrinting(true);
    try {
      const selectedPrinter = typeof window !== "undefined" ? localStorage.getItem("selectedPrinter") : null;
      await printInvoiceToHtml(invoice.id, selectedPrinter || undefined);
      toast.success("Factura s-a trimis la imprimantă!");
    } catch (error) {
      console.error("Print error:", error);
      toast.error(`Eroare la imprimare: ${error}`);
    } finally {
      setIsPrinting(false);
    }
  };

  return (
    <Card className="flex flex-col text-sm">
      <CardHeader className="pb-2 pt-3 px-3">
        <div className="space-y-2">
          <h3 className="font-semibold text-base leading-tight">{invoice.partner_name}</h3>
          <div className="flex items-center justify-between gap-2">
            <div className="flex items-center gap-1 text-xs text-muted-foreground min-w-0 flex-1">
              <MapPin className="h-3 w-3 flex-shrink-0" />
              <span className="truncate">{invoice.location_name}</span>
            </div>
            <InvoiceStatusBadge status={invoice.status} />
          </div>
        </div>
      </CardHeader>

      <CardContent className="flex-1 pb-2 px-3">
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-1 text-xs text-muted-foreground">
              <Package className="h-3 w-3" />
              <span>{invoice.item_count} produse</span>
            </div>
            <span className="text-base font-bold">{formatCurrency(invoice.total_amount)}</span>
          </div>

          <div className="text-xs text-muted-foreground">
            {formatDate(invoice.created_at)}
          </div>

          {invoice.status === "failed" && invoice.error_message && (
            <div className="text-xs text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/20 p-1.5 rounded leading-tight">
              {invoice.error_message}
            </div>
          )}

          {invoice.status === "sent" && invoice.sent_at && (
            <div className="text-xs text-green-600 dark:text-green-400">
              Trimisă: {formatDate(invoice.sent_at)}
            </div>
          )}
        </div>
      </CardContent>

      <CardFooter className="pt-2 px-3 pb-3 border-t gap-2 justify-center flex-wrap">
        <Button
          variant="outline"
          className="h-9 px-3 text-xs"
          onClick={() => onView(invoice.id)}
        >
          <Eye className="h-3.5 w-3.5 mr-1" />
          Detalii
        </Button>

        <Button
          variant="outline"
          className="h-9 w-9 p-0 flex-shrink-0"
          onClick={handlePrint}
          disabled={isPrinting}
          title="Imprimare"
        >
          {isPrinting ? (
            <Loader2 className="h-3.5 w-3.5 animate-spin" />
          ) : (
            <Printer className="h-3.5 w-3.5" />
          )}
        </Button>

        {canSend && (
          <Button
            variant={invoice.status === "failed" ? "outline" : "default"}
            className="h-9 px-3 text-xs"
            onClick={() => onSend(invoice.id)}
            disabled={isSending}
          >
            {invoice.status === "failed" ? (
              <>
                <RotateCcw className="h-3.5 w-3.5 mr-1" />
                Reîncearcă
              </>
            ) : (
              <>
                <Send className="h-3.5 w-3.5 mr-1" />
                Trimite
              </>
            )}
          </Button>
        )}

        {isSending && onCancel && (
          <Button
            variant="outline"
            className="h-9 px-3 text-xs text-orange-600 hover:text-orange-700 hover:bg-orange-50 dark:text-orange-400 dark:hover:bg-orange-900/20"
            onClick={handleCancel}
            disabled={isCancelling}
          >
            {isCancelling ? (
              <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />
            ) : (
              <XCircle className="h-3.5 w-3.5 mr-1" />
            )}
            Anulează
          </Button>
        )}

        {canDelete && (
          <AlertDialog>
            <AlertDialogTrigger asChild>
              <Button
                variant="ghost"
                className="h-9 w-9 p-0 flex-shrink-0 text-red-600 hover:text-red-700 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-900/20"
                title="Șterge factura"
              >
                <Trash2 className="h-3.5 w-3.5" />
              </Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>Șterge factura?</AlertDialogTitle>
                <AlertDialogDescription>
                  Această acțiune nu poate fi anulată. Factura va fi ștearsă permanent.
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>Anulează</AlertDialogCancel>
                <AlertDialogAction
                  onClick={() => onDelete(invoice.id)}
                  className="bg-red-600 hover:bg-red-700"
                >
                  Șterge
                </AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        )}
      </CardFooter>
    </Card>
  );
}
