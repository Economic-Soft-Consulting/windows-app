"use client";

import { useState, useEffect, useCallback } from "react";
import { getSyncStatus, syncAllData, checkFirstRun } from "@/lib/tauri/commands";
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
    try {
      const newStatus = await syncAllData();
      setStatus(newStatus);
      setIsFirstRun(false);
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
