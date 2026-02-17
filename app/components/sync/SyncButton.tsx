"use client";

import { RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useSyncStatus } from "@/hooks/useSyncStatus";
import { useOnlineStatus } from "@/hooks/useOnlineStatus";
import { toast } from "sonner";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";

export function SyncButton() {
  const { status, isSyncing, triggerSync } = useSyncStatus();
  const { isOnline } = useOnlineStatus();

  const handleSync = async () => {
    if (!isOnline) {
      toast.error("Nu există conexiune la internet");
      return;
    }

    try {
      await triggerSync();
      toast.success("Datele au fost actualizate");
    } catch (e) {
      console.error("Sync error:", e);
      toast.error(`Eroare la sincronizare: ${e}`);
    }
  };

  const formatLastSync = (dateStr: string | null) => {
    if (!dateStr) return "Niciodată";
    const date = new Date(dateStr);
    return date.toLocaleString("ro-RO", {
      day: "2-digit",
      month: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="outline"
            size="sm"
            onClick={handleSync}
            disabled={isSyncing || !isOnline}
            className="gap-2"
          >
            <RefreshCw
              className={`h-4 w-4 ${isSyncing ? "animate-spin" : ""}`}
            />
            <span className="hidden sm:inline">
              {isSyncing ? "Se sincronizează..." : "Sincronizare"}
            </span>
          </Button>
        </TooltipTrigger>
        <TooltipContent>
          <div className="space-y-1">
            <p>Sincronizare completă: parteneri, produse, solduri, încasări</p>
            <p>Parteneri: {formatLastSync(status?.partners_synced_at ?? null)}</p>
            <p>Produse: {formatLastSync(status?.products_synced_at ?? null)}</p>
          </div>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
