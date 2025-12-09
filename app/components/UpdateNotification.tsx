"use client";

import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

type UpdateState = "checking" | "downloading" | "done";

export function UpdateNotification() {
  const [state, setState] = useState<UpdateState>("done");
  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    let unlistenChecking: (() => void) | undefined;
    let unlistenDownloading: (() => void) | undefined;
    let unlistenDone: (() => void) | undefined;

    const setupListeners = async () => {
      unlistenChecking = await listen("update-checking", () => {
        setState("checking");
      });

      unlistenDownloading = await listen<string>("update-downloading", (event) => {
        setState("downloading");
        setVersion(event.payload);
      });

      unlistenDone = await listen("update-done", () => {
        setState("done");
      });
    };

    setupListeners();

    return () => {
      unlistenChecking?.();
      unlistenDownloading?.();
      unlistenDone?.();
    };
  }, []);

  if (state === "done") {
    return null;
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-white dark:bg-zinc-900">
      <div className="flex flex-col items-center gap-4">
        {/* Spinner */}
        <div className="h-12 w-12 animate-spin rounded-full border-4 border-zinc-200 border-t-blue-600 dark:border-zinc-700 dark:border-t-blue-500" />

        {state === "checking" && (
          <p className="text-lg font-medium text-zinc-700 dark:text-zinc-300">
            Checking for updates...
          </p>
        )}

        {state === "downloading" && (
          <div className="flex flex-col items-center gap-2">
            <p className="text-lg font-medium text-zinc-700 dark:text-zinc-300">
              Downloading update...
            </p>
            {version && (
              <p className="text-sm text-zinc-500 dark:text-zinc-400">
                Version {version}
              </p>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
