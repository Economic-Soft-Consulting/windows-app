"use client";

import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

interface UpdateInfo {
  version: string;
  current_version: string;
}

interface DownloadProgress {
  downloaded: number;
  total: number | null;
}

type UpdateState = "idle" | "available" | "downloading" | "ready" | "error";

export function UpdateNotification() {
  const [state, setState] = useState<UpdateState>("idle");
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [dismissed, setDismissed] = useState(false);

  useEffect(() => {
    let unlistenAvailable: (() => void) | undefined;
    let unlistenProgress: (() => void) | undefined;
    let unlistenInstalled: (() => void) | undefined;

    const setupListeners = async () => {
      unlistenAvailable = await listen<UpdateInfo>("update-available", (event) => {
        setUpdateInfo(event.payload);
        setState("available");
        setDismissed(false);
      });

      unlistenProgress = await listen<DownloadProgress>("update-download-progress", (event) => {
        const { downloaded, total } = event.payload;
        if (total) {
          setProgress(Math.round((downloaded / total) * 100));
        }
      });

      unlistenInstalled = await listen("update-installed", () => {
        setState("ready");
        setProgress(100);
      });
    };

    setupListeners();

    return () => {
      unlistenAvailable?.();
      unlistenProgress?.();
      unlistenInstalled?.();
    };
  }, []);

  const handleInstall = async () => {
    try {
      setState("downloading");
      setProgress(0);
      setError(null);
      await invoke("install_update");
    } catch (e) {
      setState("error");
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  const handleRestart = async () => {
    await invoke("restart_app");
  };

  const handleDismiss = () => {
    setDismissed(true);
  };

  if (state === "idle" || dismissed) {
    return null;
  }

  return (
    <div className="fixed bottom-4 right-4 z-50 max-w-sm rounded-lg border border-zinc-200 bg-white p-4 shadow-lg dark:border-zinc-700 dark:bg-zinc-800">
      <div className="flex items-start gap-3">
        <div className="flex-1">
          {state === "available" && (
            <>
              <p className="font-medium text-zinc-900 dark:text-zinc-100">
                Update Available
              </p>
              <p className="mt-1 text-sm text-zinc-600 dark:text-zinc-400">
                Version {updateInfo?.version} is ready to install.
              </p>
              <div className="mt-3 flex gap-2">
                <button
                  onClick={handleInstall}
                  className="rounded-md bg-blue-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-blue-700"
                >
                  Install Now
                </button>
                <button
                  onClick={handleDismiss}
                  className="rounded-md px-3 py-1.5 text-sm font-medium text-zinc-600 hover:bg-zinc-100 dark:text-zinc-400 dark:hover:bg-zinc-700"
                >
                  Later
                </button>
              </div>
            </>
          )}

          {state === "downloading" && (
            <>
              <p className="font-medium text-zinc-900 dark:text-zinc-100">
                Downloading Update...
              </p>
              <div className="mt-2 h-2 w-full overflow-hidden rounded-full bg-zinc-200 dark:bg-zinc-700">
                <div
                  className="h-full bg-blue-600 transition-all duration-300"
                  style={{ width: `${progress}%` }}
                />
              </div>
              <p className="mt-1 text-sm text-zinc-600 dark:text-zinc-400">
                {progress}% complete
              </p>
            </>
          )}

          {state === "ready" && (
            <>
              <p className="font-medium text-zinc-900 dark:text-zinc-100">
                Update Ready
              </p>
              <p className="mt-1 text-sm text-zinc-600 dark:text-zinc-400">
                Restart to apply the update.
              </p>
              <div className="mt-3 flex gap-2">
                <button
                  onClick={handleRestart}
                  className="rounded-md bg-green-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-green-700"
                >
                  Restart Now
                </button>
                <button
                  onClick={handleDismiss}
                  className="rounded-md px-3 py-1.5 text-sm font-medium text-zinc-600 hover:bg-zinc-100 dark:text-zinc-400 dark:hover:bg-zinc-700"
                >
                  Later
                </button>
              </div>
            </>
          )}

          {state === "error" && (
            <>
              <p className="font-medium text-red-600 dark:text-red-400">
                Update Failed
              </p>
              <p className="mt-1 text-sm text-zinc-600 dark:text-zinc-400">
                {error || "An error occurred while updating."}
              </p>
              <div className="mt-3">
                <button
                  onClick={handleDismiss}
                  className="rounded-md px-3 py-1.5 text-sm font-medium text-zinc-600 hover:bg-zinc-100 dark:text-zinc-400 dark:hover:bg-zinc-700"
                >
                  Dismiss
                </button>
              </div>
            </>
          )}
        </div>

        {state !== "downloading" && (
          <button
            onClick={handleDismiss}
            className="text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-200"
            aria-label="Dismiss"
          >
            <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        )}
      </div>
    </div>
  );
}
