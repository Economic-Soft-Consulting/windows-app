"use client";

import React, { createContext, useContext, useState, useEffect, useRef, useCallback } from "react";
import { sendPendingDocuments } from "@/lib/tauri/commands";
import { toast } from "sonner";

const LOG_PREFIX = "[AUTO-SEND]";

interface OnlineStatusContextValue {
    isOnline: boolean;
}

const OnlineStatusContext = createContext<OnlineStatusContextValue>({ isOnline: true });

export function OnlineStatusProvider({ children }: { children: React.ReactNode }) {
    const [isOnline, setIsOnline] = useState(true);
    const isOnlineRef = useRef(true);
    const isSendingRef = useRef(false);
    const checkCountRef = useRef(0);

    const dispatchSyncUpdates = () => {
        window.dispatchEvent(new Event("invoices-updated"));
        window.dispatchEvent(new Event("collections-updated"));
    };

    const triggerAutoSend = useCallback(async () => {
        if (isSendingRef.current) {
            console.log(`${LOG_PREFIX} Skipping auto-send — already in progress`);
            return;
        }
        if (!isOnlineRef.current) {
            console.log(`${LOG_PREFIX} Skipping auto-send — offline`);
            return;
        }

        isSendingRef.current = true;
        console.log(`${LOG_PREFIX} ===== Starting auto-send cycle =====`);

        try {
            // One backend call owns the order: all invoices, then balances, then receipts.
            // It also reports what actually happened, instead of the old before/after count
            // delta — which reported 0 for a receipt that moved pending -> failed, so
            // failures were invisible.
            const outcome = await sendPendingDocuments();

            const {
                invoices_sent, invoices_failed,
                receipts_sent, receipts_failed,
                receipts_waiting_for_invoice,
            } = outcome;

            console.log(`${LOG_PREFIX} Result:`, outcome);

            const sentParts: string[] = [];
            if (invoices_sent > 0) sentParts.push(`${invoices_sent} ${invoices_sent === 1 ? "factură" : "facturi"}`);
            if (receipts_sent > 0) sentParts.push(`${receipts_sent} ${receipts_sent === 1 ? "chitanță" : "chitanțe"}`);
            if (sentParts.length > 0) {
                toast.success(`${sentParts.join(" și ")} trimise automat.`);
            }

            // Previously a cycle where every document failed produced no feedback at all.
            const failedTotal = invoices_failed + receipts_failed;
            if (failedTotal > 0) {
                toast.warning(
                    `${failedTotal} ${failedTotal === 1 ? "document a rămas" : "documente au rămas"} în așteptare. Se reîncearcă automat.`
                );
            }

            if (receipts_waiting_for_invoice > 0) {
                toast.info(
                    `${receipts_waiting_for_invoice} ${receipts_waiting_for_invoice === 1 ? "chitanță așteaptă" : "chitanțe așteaptă"} trimiterea facturii.`
                );
            }

            if (invoices_sent > 0 || receipts_sent > 0) {
                dispatchSyncUpdates();
            }

            console.log(`${LOG_PREFIX} ===== Auto-send cycle complete =====`);
        } catch (error) {
            console.error(`${LOG_PREFIX} Auto-send FAILED:`, error);
        } finally {
            isSendingRef.current = false;
        }
    }, []);

    const checkConnectivity = useCallback(async () => {
        const checkNum = ++checkCountRef.current;
        console.log(`${LOG_PREFIX} [Check #${checkNum}] Checking connectivity... (isOnline=${isOnlineRef.current}, isSending=${isSendingRef.current})`);

        try {
            const controller = new AbortController();
            const timeoutId = setTimeout(() => controller.abort(), 3000);

            await fetch("https://www.google.com/generate_204", {
                method: "HEAD",
                mode: "no-cors",
                cache: "no-store",
                signal: controller.signal,
            });

            clearTimeout(timeoutId);

            const wasOffline = !isOnlineRef.current;
            isOnlineRef.current = true;
            setIsOnline(true);

            console.log(`${LOG_PREFIX} [Check #${checkNum}] Online ✓ (wasOffline=${wasOffline})`);

            if (wasOffline) {
                toast.success("Conexiune restabilită! Se trimit documentele în așteptare...");
            }

            triggerAutoSend();
        } catch {
            isOnlineRef.current = false;
            setIsOnline(false);
            console.warn(`${LOG_PREFIX} [Check #${checkNum}] Offline ✗`);
        }
    }, [triggerAutoSend]);

    useEffect(() => {
        console.log(`${LOG_PREFIX} OnlineStatusProvider mounted — single instance, checks every 30s`);
        checkConnectivity();

        const handleOnline = () => {
            console.log(`${LOG_PREFIX} Browser 'online' event`);
            checkConnectivity();
        };
        const handleOffline = () => {
            console.log(`${LOG_PREFIX} Browser 'offline' event`);
            isOnlineRef.current = false;
            setIsOnline(false);
        };

        window.addEventListener("online", handleOnline);
        window.addEventListener("offline", handleOffline);

        const intervalId = setInterval(() => {
            console.log(`${LOG_PREFIX} Interval tick`);
            checkConnectivity();
        }, 30000);

        return () => {
            window.removeEventListener("online", handleOnline);
            window.removeEventListener("offline", handleOffline);
            clearInterval(intervalId);
        };
    }, [checkConnectivity]);

    return (
        <OnlineStatusContext.Provider value={{ isOnline }}>
            {children}
        </OnlineStatusContext.Provider>
    );
}

export function useOnlineStatus() {
    return useContext(OnlineStatusContext);
}
