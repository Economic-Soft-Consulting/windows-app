"use client";

import { useState, useEffect, useCallback } from "react";
import { getCollections, sendAllPendingInvoices, syncClientBalances, syncCollections } from "@/lib/tauri/commands";
import { toast } from "sonner";

export function useOnlineStatus() {
  const [isOnline, setIsOnline] = useState(true);
  const [wasOffline, setWasOffline] = useState(false);

  const dispatchSyncUpdates = () => {
    window.dispatchEvent(new Event("invoices-updated"));
    window.dispatchEvent(new Event("collections-updated"));
  };

  // Actively check connectivity by trying to fetch a small resource
  const checkConnectivity = useCallback(async () => {
    try {
      // Try to fetch a tiny resource with a short timeout
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 3000);

      // Use a reliable, fast endpoint - Google's generate_204 endpoint
      await fetch("https://www.google.com/generate_204", {
        method: "HEAD",
        mode: "no-cors",
        cache: "no-store",
        signal: controller.signal,
      });

      clearTimeout(timeoutId);
      
      // Connection restored - try to send pending invoices
      if (!isOnline || wasOffline) {
        setIsOnline(true);
        setWasOffline(false);
        
        // Auto-send pending invoices and collections in background
        try {
          const pendingBefore = await getCollections("pending");
          const failedBefore = await getCollections("failed");

          const sentIds = await sendAllPendingInvoices();
          try {
            await syncClientBalances();
          } catch (balanceError) {
            console.warn("Sync solduri eșuat la reconectare, continuăm cu sincronizarea chitanțelor.", balanceError);
            toast.warning("Soldurile nu au putut fi sincronizate la reconectare. Chitanțele continuă să se sincronizeze.");
          }
          await syncCollections();

          const pendingAfter = await getCollections("pending");
          const failedAfter = await getCollections("failed");

          const collectionsProcessed = Math.max(0, pendingBefore.length - pendingAfter.length);
          const collectionsFailedNow = Math.max(0, failedAfter.length - failedBefore.length);

          if (sentIds.length > 0) {
            toast.success(`Conexiune restabilită! ${sentIds.length} facturi trimise automat.`);
          }

          if (collectionsProcessed > 0 || collectionsFailedNow > 0) {
            toast.success(
              `Conexiune restabilită! Chitanțe procesate: ${collectionsProcessed}${collectionsFailedNow > 0 ? `, eșuate: ${collectionsFailedNow}` : ""}.`
            );
          }

          dispatchSyncUpdates();
        } catch (error) {
          console.error("Failed to auto-send pending invoices/collections:", error);
        }
      } else {
        setIsOnline(true);
      }
    } catch {
      if (isOnline) {
        setWasOffline(true);
      }
      setIsOnline(false);
    }
  }, [isOnline, wasOffline]);

  useEffect(() => {
    // Initial check
    checkConnectivity();

    // Browser events (may not always fire in Tauri)
    const handleOnline = () => {
      checkConnectivity();
    };
    const handleOffline = () => {
      setIsOnline(false);
    };

    window.addEventListener("online", handleOnline);
    window.addEventListener("offline", handleOffline);

    // Periodic check every 5 seconds
    const intervalId = setInterval(checkConnectivity, 5000);

    return () => {
      window.removeEventListener("online", handleOnline);
      window.removeEventListener("offline", handleOffline);
      clearInterval(intervalId);
    };
  }, [checkConnectivity]);

  return { isOnline };
}
