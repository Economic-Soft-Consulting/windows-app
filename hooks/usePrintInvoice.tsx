"use client";

import { useState } from "react";
import {
  getInvoiceDetail,
  printInvoiceToHtml,
  printCollectionToHtml,
  recordCollectionFromInvoice,
  getInvoiceRemainingForCollection,
  sendCollection,
} from "@/lib/tauri/commands";
import { toast } from "sonner";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { useOnlineStatus } from "@/hooks/useOnlineStatus";

export function usePrintInvoice() {
  const { isOnline } = useOnlineStatus();
  const [isPrinting, setIsPrinting] = useState(false);
  const [showReceiptDialog, setShowReceiptDialog] = useState(false);
  const [savingReceipt, setSavingReceipt] = useState(false);
  const [receiptInvoiceId, setReceiptInvoiceId] = useState<string | null>(null);
  const [invoiceTotal, setInvoiceTotal] = useState(0);
  const [paymentMode, setPaymentMode] = useState<"full" | "partial">("full");
  const [partialAmount, setPartialAmount] = useState("");
  const [afterReceiptAction, setAfterReceiptAction] = useState<(() => void) | null>(null);

  const closeReceiptDialog = (runAfterAction = true) => {
    setShowReceiptDialog(false);
    setSavingReceipt(false);
    setReceiptInvoiceId(null);
    setInvoiceTotal(0);
    setPaymentMode("full");
    setPartialAmount("");
    if (runAfterAction && afterReceiptAction) {
      const action = afterReceiptAction;
      setAfterReceiptAction(null);
      action();
    } else {
      setAfterReceiptAction(null);
    }
  };

  const saveReceipt = async () => {
    if (!receiptInvoiceId) return;

    let paidAmount = invoiceTotal;
    if (paymentMode === "partial") {
      paidAmount = Number.parseFloat(partialAmount.replace(",", "."));
      if (!Number.isFinite(paidAmount) || paidAmount <= 0) {
        toast.error("Suma introdusă este invalidă.");
        return;
      }
    }

    setSavingReceipt(true);
    try {
      const collectionId = await recordCollectionFromInvoice(receiptInvoiceId, paidAmount);

      const selectedPrinter = typeof window !== "undefined"
        ? localStorage.getItem("selectedPrinter")
        : null;

      try {
        await printCollectionToHtml(collectionId, selectedPrinter || undefined);
        toast.success("Chitanța a fost salvată și trimisă la imprimantă.");
      } catch (printError) {
        console.error("Receipt print error:", printError);
        toast.warning("Chitanța a fost salvată, dar printarea a eșuat.");
      }

      if (isOnline) {
        try {
          const sentCollection = await sendCollection(collectionId);
          if (sentCollection.status === "synced") {
            toast.success("Chitanța a fost trimisă către Mentor.");
          } else if (sentCollection.status === "failed") {
            toast.warning("Chitanța a fost salvată, dar trimiterea a eșuat. Se va reîncerca automat.");
          } else {
            toast.info("Chitanța a fost salvată. Trimiterea este în așteptare.");
          }
        } catch (sendError) {
          console.error("Receipt send error:", sendError);
          toast.warning("Chitanța a fost salvată, dar nu s-a putut trimite acum. Va fi trimisă automat când revine internetul.");
        }
      } else {
        toast.info("Chitanța a fost salvată în așteptare și va fi trimisă automat când revine internetul.");
      }

      if (typeof window !== "undefined") {
        window.dispatchEvent(new Event("collections-updated"));
      }

      closeReceiptDialog();
    } catch (error) {
      console.error("Receipt save error:", error);
      toast.error(`Eroare la salvarea chitanței: ${error}`);
      setSavingReceipt(false);
    }
  };

  const printInvoice = async (invoiceId: string, onAfterReceipt?: () => void) => {
    setIsPrinting(true);
    setAfterReceiptAction(() => onAfterReceipt ?? null);
    try {
      const selectedPrinter = typeof window !== "undefined"
        ? localStorage.getItem("selectedPrinter")
        : null;
      await printInvoiceToHtml(invoiceId, selectedPrinter || undefined);
      toast.success("Factura s-a trimis la imprimantă!");
      const detail = await getInvoiceDetail(invoiceId);
      const total = detail.invoice.total_amount;
      if (total <= 0) {
        toast.error("Factura are total 0. Nu se poate genera chitanță.");
        if (onAfterReceipt) {
          onAfterReceipt();
        }
        setAfterReceiptAction(null);
        return;
      }

      const remaining = await getInvoiceRemainingForCollection(invoiceId);
      if (remaining <= 0.0001) {
        toast.info("Factura este deja încasată integral.");
        if (onAfterReceipt) {
          onAfterReceipt();
        }
        setAfterReceiptAction(null);
        return;
      }

      if (remaining + 0.0001 < total) {
        toast.info(`Factura are deja încasări. Rest disponibil: ${remaining.toFixed(2)} RON.`);
      }

      setReceiptInvoiceId(invoiceId);
      setInvoiceTotal(remaining);
      setPaymentMode("full");
      setPartialAmount(remaining.toFixed(2));
      setShowReceiptDialog(true);
    } catch (error) {
      console.error("Print error:", error);
      toast.error(`Eroare la imprimare: ${error}`);
    } finally {
      setIsPrinting(false);
    }
  };

  const receiptDialog = (
    <Dialog open={showReceiptDialog} onOpenChange={(open) => !open && closeReceiptDialog()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Generezi și chitanță?</DialogTitle>
          <DialogDescription>
            Alege tipul încasării pentru factura printată.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <div className="text-sm">
            Total factură: <span className="font-semibold">{invoiceTotal.toFixed(2)} RON</span>
          </div>

          <RadioGroup value={paymentMode} onValueChange={(value) => setPaymentMode(value as "full" | "partial")}>
            <div className="flex items-center gap-2">
              <RadioGroupItem value="full" id="full-payment" />
              <Label htmlFor="full-payment">Achitată total</Label>
            </div>
            <div className="flex items-center gap-2">
              <RadioGroupItem value="partial" id="partial-payment" />
              <Label htmlFor="partial-payment">Achitată parțial</Label>
            </div>
          </RadioGroup>

          {paymentMode === "partial" && (
            <div className="space-y-2">
              <Label htmlFor="paid-amount">Suma achitată</Label>
              <Input
                id="paid-amount"
                inputMode="decimal"
                value={partialAmount}
                onChange={(e) => setPartialAmount(e.target.value)}
                placeholder="0.00"
              />
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => closeReceiptDialog()} disabled={savingReceipt}>
            Nu, mulțumesc
          </Button>
          <Button onClick={saveReceipt} disabled={savingReceipt}>
            {savingReceipt ? "Se salvează..." : "Salvează chitanța"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );

  return { printInvoice, isPrinting, receiptDialog };
}