"use client";

import { useEffect, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import { Sidebar } from "../components/layout/Sidebar";
import { Header } from "../components/layout/Header";
import { FirstRunOverlay } from "../components/sync/FirstRunOverlay";
import { useSyncStatus } from "@/hooks/useSyncStatus";
import { OnlineStatusProvider, useOnlineStatus } from "@/app/contexts/OnlineStatusContext";
import { useAuth } from "@/app/contexts/AuthContext";
import { toast } from "sonner";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <OnlineStatusProvider>
      <DashboardLayoutInner>{children}</DashboardLayoutInner>
    </OnlineStatusProvider>
  );
}

function DashboardLayoutInner({
  children,
}: {
  children: React.ReactNode;
}) {
  const { checkIsFirstRun, triggerSync, isSyncing } = useSyncStatus();
  const { isOnline } = useOnlineStatus();
  const { isAuthenticated } = useAuth();
  const router = useRouter();
  const [showFirstRun, setShowFirstRun] = useState<boolean | null>(null);
  const [isChecking, setIsChecking] = useState(true);
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const autoSyncRunningRef = useRef(false);
  const lastAutoSyncDateRef = useRef<string | null>(null);
  const pendingRetryRef = useRef(false);
  const lastRetryAttemptAtRef = useRef(0);

  // Redirect to login if not authenticated
  useEffect(() => {
    if (!isAuthenticated) {
      router.push("/login");
    }
  }, [isAuthenticated, router]);

  useEffect(() => {
    if (isAuthenticated) {
      checkIsFirstRun().then((result) => {
        setShowFirstRun(result);
        setIsChecking(false);
      });
    }
  }, [checkIsFirstRun, isAuthenticated]);

  useEffect(() => {
    if (!isAuthenticated) {
      return;
    }

    const RETRY_INTERVAL_MS = 5 * 60 * 1000;

    const toLocalDayKey = (date: Date) => {
      const year = date.getFullYear();
      const month = String(date.getMonth() + 1).padStart(2, "0");
      const day = String(date.getDate()).padStart(2, "0");
      return `${year}-${month}-${day}`;
    };

    const runFullAutoSync = async (now: Date) => {
      if (autoSyncRunningRef.current || isSyncing) {
        return;
      }

      if (!isOnline) {
        pendingRetryRef.current = true;
        return;
      }

      // Check WME host before syncing — skip silently if not configured
      try {
        const { getAgentSettings } = await import("@/lib/tauri/commands");
        const settings = await getAgentSettings();
        if (!settings.wme_host?.trim()) {
          console.warn("Auto-sync skipped: WME host not configured.");
          pendingRetryRef.current = false;
          return;
        }
      } catch {
        // If we can't read settings, skip sync
        return;
      }

      autoSyncRunningRef.current = true;
      try {
        await triggerSync();
        lastAutoSyncDateRef.current = toLocalDayKey(now);
        pendingRetryRef.current = false;
        toast.success("Sincronizarea automată completă a fost executată.");
      } catch (error) {
        pendingRetryRef.current = true;
        console.error("Auto-sync failed:", error);
      } finally {
        autoSyncRunningRef.current = false;
      }
    };

    const checkAutoSync = async () => {
      // First, get settings dynamically to check if enabled and get the time
      let syncEnabled = false;
      let syncHour = 23;
      let syncMinute = 59;

      try {
        const { getAgentSettings } = await import("@/lib/tauri/commands");
        const settings = await getAgentSettings();
        syncEnabled = settings.auto_sync_collections_enabled || false;

        if (settings.auto_sync_collections_time) {
          const [h, m] = settings.auto_sync_collections_time.split(":");
          if (h && m) {
            syncHour = parseInt(h, 10);
            syncMinute = parseInt(m, 10);
          }
        }
      } catch (e) {
        console.warn("Could not read auto-sync settings", e);
      }

      // If user disabled it in settings, don't queue a sync
      if (!syncEnabled) {
        pendingRetryRef.current = false;
        return;
      }

      const now = new Date();
      const todayKey = toLocalDayKey(now);

      const target = new Date(now);
      target.setHours(syncHour, syncMinute, 0, 0);

      const dueToday =
        now.getTime() >= target.getTime() &&
        lastAutoSyncDateRef.current !== todayKey;

      if (dueToday) {
        pendingRetryRef.current = true;
      }

      const canRetryNow =
        pendingRetryRef.current &&
        now.getTime() - lastRetryAttemptAtRef.current >= RETRY_INTERVAL_MS;

      if (!canRetryNow) {
        return;
      }

      lastRetryAttemptAtRef.current = now.getTime();
      await runFullAutoSync(now);
    };

    checkAutoSync();
    const intervalId = window.setInterval(checkAutoSync, 60 * 1000);

    return () => {
      window.clearInterval(intervalId);
    };
  }, [isAuthenticated, triggerSync, isOnline, isSyncing]);

  // Show nothing while checking or not authenticated
  if (!isAuthenticated || isChecking || showFirstRun === null) {
    return (
      <div className="h-screen flex items-center justify-center bg-background">
        <div className="animate-pulse text-muted-foreground">Se încarcă...</div>
      </div>
    );
  }

  return (
    <>
      {showFirstRun && (
        <FirstRunOverlay onComplete={() => setShowFirstRun(false)} />
      )}
      <div className="fixed inset-0 flex bg-background">
        <Sidebar
          isOpen={sidebarOpen}
          onClose={() => setSidebarOpen(false)}
        />
        <div className="flex-1 flex flex-col overflow-hidden">
          <Header onMenuClick={() => setSidebarOpen(true)} />
          <main className="flex-1 overflow-hidden p-3 bg-muted/40">
            <div className="max-w-7xl mx-auto h-full">
              {children}
            </div>
          </main>
        </div>
      </div>
    </>
  );
}
