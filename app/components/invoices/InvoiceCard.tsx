"use client";

import { useState } from "react";
import { Card, CardContent, CardFooter, CardHeader } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { InvoiceStatusBadge } from "./InvoiceStatusBadge";
import { Send, Trash2, Eye, RotateCcw, MapPin, Package, Printer, Loader2 } from "lucide-react";
import type { Invoice } from "@/lib/tauri/types";
import { printInvoiceToHtml } from "@/lib/tauri/commands";
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

export function InvoiceCard({ invoice, onSend, onDelete, onView }: InvoiceCardProps) {
  const [isPrinting, setIsPrinting] = useState(false);
  const canSend = invoice.status === "pending" || invoice.status === "failed";
  const canDelete = invoice.status === "pending" || invoice.status === "failed";
  const isSending = invoice.status === "sending";

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
    <Card className="flex flex-col">
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between gap-2">
          <div className="flex-1 min-w-0">
            <h3 className="font-semibold text-lg truncate">{invoice.partner_name}</h3>
            <div className="flex items-center gap-1.5 text-sm text-muted-foreground mt-1">
              <MapPin className="h-3.5 w-3.5 flex-shrink-0" />
              <span className="truncate">{invoice.location_name}</span>
            </div>
          </div>
          <InvoiceStatusBadge status={invoice.status} />
        </div>
      </CardHeader>

      <CardContent className="flex-1 pb-3">
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-1.5 text-sm text-muted-foreground">
              <Package className="h-4 w-4" />
              <span>{invoice.item_count} produse</span>
            </div>
            <span className="text-xl font-bold">{formatCurrency(invoice.total_amount)}</span>
          </div>

          <div className="text-sm text-muted-foreground">
            {formatDate(invoice.created_at)}
          </div>

          {invoice.status === "failed" && invoice.error_message && (
            <div className="text-sm text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/20 p-2 rounded">
              {invoice.error_message}
            </div>
          )}

          {invoice.status === "sent" && invoice.sent_at && (
            <div className="text-sm text-green-600 dark:text-green-400">
              Trimisă: {formatDate(invoice.sent_at)}
            </div>
          )}
        </div>
      </CardContent>

      <CardFooter className="pt-3 border-t gap-2">
        <Button
          variant="outline"
          className="flex-1 h-11"
          onClick={() => onView(invoice.id)}
        >
          <Eye className="h-5 w-5 mr-1.5" />
          Detalii
        </Button>

        <Button
          variant="outline"
          className="h-11 w-11 p-0"
          onClick={handlePrint}
          disabled={isPrinting}
          title="Imprimare"
        >
          {isPrinting ? (
            <Loader2 className="h-5 w-5 animate-spin" />
          ) : (
            <Printer className="h-5 w-5" />
          )}
        </Button>

        {canSend && (
          <Button
            variant={invoice.status === "failed" ? "outline" : "default"}
            className="flex-1 h-11"
            onClick={() => onSend(invoice.id)}
            disabled={isSending}
          >
            {invoice.status === "failed" ? (
              <>
                <RotateCcw className="h-5 w-5 mr-1.5" />
                Reîncearcă
              </>
            ) : (
              <>
                <Send className="h-5 w-5 mr-1.5" />
                Trimite
              </>
            )}
          </Button>
        )}

        {canDelete && (
          <AlertDialog>
            <AlertDialogTrigger asChild>
              <Button
                variant="ghost"
                className="h-11 w-11 p-0 text-red-600 hover:text-red-700 hover:bg-red-50 dark:text-red-400 dark:hover:bg-red-900/20"
              >
                <Trash2 className="h-5 w-5" />
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
