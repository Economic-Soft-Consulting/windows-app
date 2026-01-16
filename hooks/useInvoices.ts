"use client";

import { useState, useEffect, useCallback } from "react";
import {
  getInvoices,
  sendInvoice,
  deleteInvoice,
  getInvoiceDetail,
  printInvoiceToHtml,
} from "@/lib/tauri/commands";
import type { Invoice, InvoiceStatus, InvoiceDetail } from "@/lib/tauri/types";
import { toast } from "sonner";

export function useInvoices(statusFilter?: InvoiceStatus) {
  const [invoices, setInvoices] = useState<Invoice[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setIsLoading(true);
    try {
      const data = await getInvoices(statusFilter);
      setInvoices(data);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsLoading(false);
    }
  }, [statusFilter]);

  const send = useCallback(
    async (invoiceId: string) => {
      try {
        // Update local state to show sending
        setInvoices((prev) =>
          prev.map((inv) =>
            inv.id === invoiceId ? { ...inv, status: "sending" as InvoiceStatus } : inv
          )
        );

        const updated = await sendInvoice(invoiceId);

        // Update local state with result
        setInvoices((prev) =>
          prev.map((inv) => (inv.id === invoiceId ? updated : inv))
        );

        if (updated.status === "sent") {
          toast.success("Factura a fost trimisă cu succes!");
        } else if (updated.status === "failed") {
          toast.error(`Eroare la trimitere: ${updated.error_message}`);
        }

        return updated;
      } catch (e) {
        toast.error(String(e));
        await refresh();
        throw e;
      }
    },
    [refresh]
  );

  const remove = useCallback(
    async (invoiceId: string) => {
      try {
        await deleteInvoice(invoiceId);
        setInvoices((prev) => prev.filter((inv) => inv.id !== invoiceId));
        toast.success("Factura a fost ștearsă");
      } catch (e) {
        toast.error(String(e));
      }
    },
    []
  );

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { invoices, isLoading, error, refresh, send, remove };
}

export function useInvoiceDetail(invoiceId: string | null) {
  const [detail, setDetail] = useState<InvoiceDetail | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!invoiceId) {
      setDetail(null);
      return;
    }

    setIsLoading(true);
    try {
      const data = await getInvoiceDetail(invoiceId);
      setDetail(data);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setIsLoading(false);
    }
  }, [invoiceId]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { detail, isLoading, error, refresh };
}
