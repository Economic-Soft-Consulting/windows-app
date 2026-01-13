"use client";

import { useOnlineStatus } from "@/hooks/useOnlineStatus";
import { Wifi, WifiOff } from "lucide-react";
import { cn } from "@/lib/utils";

export function NetworkIndicator() {
  const { isOnline } = useOnlineStatus();

  return (
    <div
      className={cn(
        "flex items-center gap-2 px-3 py-1.5 rounded-full text-sm font-medium transition-colors",
        isOnline
          ? "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400"
          : "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400"
      )}
    >
      {isOnline ? (
        <>
          <Wifi className="h-4 w-4" />
          <span>Online</span>
        </>
      ) : (
        <>
          <WifiOff className="h-4 w-4" />
          <span>Offline</span>
        </>
      )}
    </div>
  );
}
