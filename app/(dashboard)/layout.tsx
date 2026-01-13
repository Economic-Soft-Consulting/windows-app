"use client";

import { useEffect, useState } from "react";
import { Sidebar } from "../components/layout/Sidebar";
import { Header } from "../components/layout/Header";
import { FirstRunOverlay } from "../components/sync/FirstRunOverlay";
import { useSyncStatus } from "@/hooks/useSyncStatus";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const { checkIsFirstRun, isFirstRun } = useSyncStatus();
  const [showFirstRun, setShowFirstRun] = useState<boolean | null>(null);
  const [isChecking, setIsChecking] = useState(true);

  useEffect(() => {
    checkIsFirstRun().then((result) => {
      setShowFirstRun(result);
      setIsChecking(false);
    });
  }, [checkIsFirstRun]);

  // Show nothing while checking first run status
  if (isChecking || showFirstRun === null) {
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
      <div className="h-screen flex bg-background">
        <Sidebar />
        <div className="flex-1 flex flex-col overflow-hidden">
          <Header />
          <main className="flex-1 overflow-auto p-6">{children}</main>
        </div>
      </div>
    </>
  );
}
