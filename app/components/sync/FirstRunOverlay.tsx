"use client";

import { useEffect, useState } from "react";
import { Loader2, WifiOff, RefreshCw } from "lucide-react";
import { useSyncStatus } from "@/hooks/useSyncStatus";
import { useOnlineStatus } from "@/hooks/useOnlineStatus";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

interface FirstRunOverlayProps {
  onComplete: () => void;
}

export function FirstRunOverlay({ onComplete }: FirstRunOverlayProps) {
  const { triggerSync, isSyncing, error } = useSyncStatus();
  const { isOnline } = useOnlineStatus();
  const [hasAttempted, setHasAttempted] = useState(false);

  useEffect(() => {
    if (isOnline && !hasAttempted && !isSyncing) {
      setHasAttempted(true);
      triggerSync()
        .then(() => {
          onComplete();
        })
        .catch(() => {
          // Error is handled by the component
        });
    }
  }, [isOnline, hasAttempted, isSyncing, triggerSync, onComplete]);

  const handleRetry = () => {
    setHasAttempted(false);
  };

  return (
    <div className="fixed inset-0 bg-background/95 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <CardTitle className="text-2xl">eSoft Facturi</CardTitle>
          <CardDescription>
            Pregătire aplicație pentru prima utilizare
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col items-center gap-6 py-8">
          {!isOnline ? (
            <>
              <div className="h-16 w-16 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center">
                <WifiOff className="h-8 w-8 text-red-600 dark:text-red-400" />
              </div>
              <div className="text-center space-y-2">
                <p className="font-medium text-lg">Conexiune necesară</p>
                <p className="text-muted-foreground text-sm">
                  Pentru prima utilizare este necesară o conexiune la internet
                  pentru a descărca lista de parteneri și produse.
                </p>
              </div>
              <Button onClick={handleRetry} variant="outline" className="gap-2">
                <RefreshCw className="h-4 w-4" />
                Reîncearcă
              </Button>
            </>
          ) : error ? (
            <>
              <div className="h-16 w-16 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center">
                <WifiOff className="h-8 w-8 text-red-600 dark:text-red-400" />
              </div>
              <div className="text-center space-y-2">
                <p className="font-medium text-lg">Eroare la sincronizare</p>
                <p className="text-muted-foreground text-sm">{error}</p>
              </div>
              <Button onClick={handleRetry} variant="outline" className="gap-2">
                <RefreshCw className="h-4 w-4" />
                Reîncearcă
              </Button>
            </>
          ) : (
            <>
              <div className="h-16 w-16 rounded-full bg-primary/10 flex items-center justify-center">
                <Loader2 className="h-8 w-8 text-primary animate-spin" />
              </div>
              <div className="text-center space-y-2">
                <p className="font-medium text-lg">Se încarcă datele...</p>
                <p className="text-muted-foreground text-sm">
                  Se descarcă lista de parteneri și produse.
                  <br />
                  Vă rugăm așteptați.
                </p>
              </div>
            </>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
