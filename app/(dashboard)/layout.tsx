"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { Sidebar } from "../components/layout/Sidebar";
import { Header } from "../components/layout/Header";
import { FirstRunOverlay } from "../components/sync/FirstRunOverlay";
import { useSyncStatus } from "@/hooks/useSyncStatus";
import { useAuth } from "@/app/contexts/AuthContext";

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const { checkIsFirstRun } = useSyncStatus();
  const { isAuthenticated } = useAuth();
  const router = useRouter();
  const [showFirstRun, setShowFirstRun] = useState<boolean | null>(null);
  const [isChecking, setIsChecking] = useState(true);
  const [sidebarOpen, setSidebarOpen] = useState(true);

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
