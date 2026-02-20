"use client";

import { useState, useEffect, useCallback } from "react";
import {
  getSyncStatus,
  syncAllData,
  sendAllPendingInvoices,
  checkFirstRun,
  syncClientBalances,
  syncCollections,
} from "@/lib/tauri/commands";
import { toast } from "sonner";
import type { SyncStatus } from "@/lib/tauri/types";

export function useSyncStatus() {
  const [status, setStatus] = useState<SyncStatus | null>(null);
  const [isSyncing, setIsSyncing] = useState(false);
  const [isFirstRun, setIsFirstRun] = useState<boolean | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const newStatus = await getSyncStatus();
      setStatus(newStatus);
      setIsFirstRun(newStatus.is_first_run);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const triggerSync = useCallback(async () => {
    setIsSyncing(true);
    setError(null);
    // Dispatch event when sync starts
    window.dispatchEvent(new CustomEvent('sync-started'));
    try {
      await syncAllData();

      // Send invoices and show a summary toast
      const invoiceResults = await sendAllPendingInvoices();
      if (invoiceResults.length > 0) {
        const failed = invoiceResults.filter((r) => r.toLowerCase().includes("eroare") || r.toLowerCase().includes("error") || r.toLowerCase().includes("eșuat")).length;
        const sent = invoiceResults.length - failed;
        if (failed > 0) {
          toast.warning(`Facturi: ${sent} trimise, ${failed} eșuate.`);
        } else {
          toast.success(`${sent} factur${sent === 1 ? "ă trimisă" : "i trimise"} cu succes.`);
        }
      }

      // Sync balances (non-blocking)
      try {
        await syncClientBalances();
      } catch (balanceError) {
        console.warn("Sync solduri eșuat, continuăm cu sincronizarea chitanțelor.", balanceError);
        toast.warning("Soldurile nu au putut fi sincronizate acum. Chitanțele continuă să se sincronizeze.");
      }

      // Sync collections and show toast
      try {
        await syncCollections();
        toast.success("Chitanțele au fost sincronizate.");
      } catch (collErr) {
        console.warn("Sync chitante eșuat:", collErr);
        toast.warning("Chitanțele nu au putut fi sincronizate acum.");
      }

      const newStatus = await getSyncStatus();
      setStatus(newStatus);
      setIsFirstRun(false);
      // Dispatch event when sync completes
      window.dispatchEvent(new CustomEvent('sync-completed'));
      return newStatus;
    } catch (e) {
      setError(String(e));
      throw e;
    } finally {
      setIsSyncing(false);
    }
  }, []);

  const checkIsFirstRun = useCallback(async () => {
    try {
      const result = await checkFirstRun();
      setIsFirstRun(result);
      return result;
    } catch (e) {
      setError(String(e));
      return true;
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return {
    status,
    isSyncing,
    isFirstRun,
    error,
    triggerSync,
    refresh,
    checkIsFirstRun,
  };
}
