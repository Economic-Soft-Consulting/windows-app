"use client";

import React, { createContext, useContext, useState, useEffect, useRef, useCallback } from "react";
import { getCollections, sendAllPendingInvoices, syncClientBalances, syncCollections } from "@/lib/tauri/commands";
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
            const pendingBefore = await getCollections("pending");
            const failedBefore = await getCollections("failed");
            console.log(`${LOG_PREFIX} Collections before: pending=${pendingBefore.length}, failed=${failedBefore.length}`);

            console.log(`${LOG_PREFIX} Calling sendAllPendingInvoices...`);
            const sentIds = await sendAllPendingInvoices();
            console.log(`${LOG_PREFIX} sendAllPendingInvoices done. Sent ${sentIds.length} invoices:`, sentIds);

            console.log(`${LOG_PREFIX} Calling syncClientBalances...`);
            try {
                await syncClientBalances();
                console.log(`${LOG_PREFIX} syncClientBalances done`);
            } catch (balanceError) {
                console.warn(`${LOG_PREFIX} syncClientBalances FAILED:`, balanceError);
            }

            console.log(`${LOG_PREFIX} Calling syncCollections...`);
            await syncCollections();
            console.log(`${LOG_PREFIX} syncCollections done`);

            const pendingAfter = await getCollections("pending");
            const failedAfter = await getCollections("failed");
            console.log(`${LOG_PREFIX} Collections after: pending=${pendingAfter.length}, failed=${failedAfter.length}`);

            const collectionsProcessed = Math.max(0, (pendingBefore.length + failedBefore.length) - (pendingAfter.length + failedAfter.length));

            console.log(`${LOG_PREFIX} Result: invoices_sent=${sentIds.length}, collections_processed=${collectionsProcessed}`);

            if (sentIds.length > 0) {
                toast.success(`${sentIds.length} facturi trimise automat.`);
            }
            if (collectionsProcessed > 0) {
                toast.success(`${collectionsProcessed} chitanțe procesate automat.`);
            }
            if (sentIds.length > 0 || collectionsProcessed > 0) {
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
