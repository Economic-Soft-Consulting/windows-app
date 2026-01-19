"use client";

import { useState, useEffect, useCallback } from "react";
import { sendAllPendingInvoices } from "@/lib/tauri/commands";
import { toast } from "sonner";

export function useOnlineStatus() {
  const [isOnline, setIsOnline] = useState(true);
  const [wasOffline, setWasOffline] = useState(false);

  // Actively check connectivity by trying to fetch a small resource
  const checkConnectivity = useCallback(async () => {
    try {
      // Try to fetch a tiny resource with a short timeout
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 3000);

      // Use a reliable, fast endpoint - Google's generate_204 endpoint
      const response = await fetch("https://www.google.com/generate_204", {
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
        
        // Auto-send pending invoices in background
        try {
          const sentIds = await sendAllPendingInvoices();
          if (sentIds.length > 0) {
            toast.success(`Conexiune restabilită! ${sentIds.length} facturi trimise automat.`);
            // Trigger refresh event for invoice list
            window.dispatchEvent(new Event('invoices-updated'));
          }
        } catch (error) {
          console.error("Failed to auto-send pending invoices:", error);
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
