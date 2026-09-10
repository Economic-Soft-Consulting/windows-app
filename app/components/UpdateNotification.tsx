"use client";

import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";

type UpdateState = "checking" | "downloading" | "failed" | "done";

interface DownloadProgress {
  downloaded: number;
  /** Null when the server sends no content length; only bytes so far can be shown. */
  total: number | null;
}

interface UpdateFailure {
  message: string;
  current_version: string;
}

/**
 * How long the check may sit on screen before offering a way past it.
 *
 * A healthy check finishes well inside this, so the escape hatch stays invisible in normal
 * use and only appears when something is actually wrong.
 */
const ESCAPE_AFTER_MS = 5_000;

/**
 * Hard stop for the checking state.
 *
 * The backend bounds its own request, but this overlay covers the entire app, so it must not
 * depend on the backend emitting anything at all to disappear.
 */
const CHECK_GIVE_UP_MS = 20_000;

/** How long a failure notice stays up before the app continues on its own. */
const FAILURE_DISMISS_MS = 10_000;

function formatMb(bytes: number): string {
  return `${(bytes / 1_048_576).toFixed(1)} MB`;
}

export function UpdateNotification() {
  const [state, setState] = useState<UpdateState>("done");
  const [version, setVersion] = useState<string | null>(null);
  const [progress, setProgress] = useState<DownloadProgress | null>(null);
  const [failure, setFailure] = useState<string | null>(null);
  // Only ever set true, by a timer; whether the button shows is derived from the state below,
  // so leaving "checking" hides it without needing to reset this.
  const [skipOffered, setSkipOffered] = useState(false);

  // Set once the user chooses to continue, so a late event cannot put the overlay back up.
  const dismissed = useRef(false);

  useEffect(() => {
    // Only setup Tauri listeners if running in Tauri context
    if (typeof window === "undefined" || !(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__) {
      return;
    }

    const unlisteners: Array<() => void> = [];

    const setupListeners = async () => {
      unlisteners.push(
        await listen("update-checking", () => {
          if (!dismissed.current) setState("checking");
        })
      );

      unlisteners.push(
        await listen<string>("update-downloading", (event) => {
          if (dismissed.current) return;
          setState("downloading");
          setVersion(event.payload);
          setProgress(null);
        })
      );

      unlisteners.push(
        await listen<DownloadProgress>("update-progress", (event) => {
          if (!dismissed.current) setProgress(event.payload);
        })
      );

      unlisteners.push(
        await listen<UpdateFailure>("update-failed", (event) => {
          if (dismissed.current) return;
          setFailure(event.payload.message);
          setState("failed");
        })
      );

      unlisteners.push(
        await listen("update-done", () => {
          setState("done");
        })
      );
    };

    setupListeners();

    return () => {
      unlisteners.forEach((off) => off());
    };
  }, []);

  // Offer a way out of a check that is taking too long, and give up entirely if it never
  // returns. Downloading is deliberately excluded: interrupting an install is the failure
  // mode this whole screen exists to prevent.
  useEffect(() => {
    if (state !== "checking") return;

    const showSkip = window.setTimeout(() => setSkipOffered(true), ESCAPE_AFTER_MS);
    const giveUp = window.setTimeout(() => {
      dismissed.current = true;
      setState("done");
    }, CHECK_GIVE_UP_MS);

    return () => {
      window.clearTimeout(showSkip);
      window.clearTimeout(giveUp);
    };
  }, [state]);

  // A failure notice must be seen, but must never strand the user behind it.
  useEffect(() => {
    if (state !== "failed") return;
    const timer = window.setTimeout(() => setState("done"), FAILURE_DISMISS_MS);
    return () => window.clearTimeout(timer);
  }, [state]);

  if (state === "done") {
    return null;
  }

  const skip = () => {
    dismissed.current = true;
    setState("done");
  };

  const percent =
    progress && progress.total
      ? Math.min(100, Math.round((progress.downloaded / progress.total) * 100))
      : null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-white dark:bg-zinc-900">
      <div className="flex w-full max-w-sm flex-col items-center gap-4 px-6 text-center">
        {state !== "failed" && (
          <div className="h-12 w-12 animate-spin rounded-full border-4 border-zinc-200 border-t-blue-600 dark:border-zinc-700 dark:border-t-blue-500" />
        )}

        {state === "checking" && (
          <>
            <p className="text-lg font-medium text-zinc-700 dark:text-zinc-300">
              Se verifică actualizările...
            </p>
            {skipOffered && (
              <button
                onClick={skip}
                className="rounded-md border border-zinc-300 px-4 py-2 text-sm text-zinc-600 hover:bg-zinc-50 dark:border-zinc-700 dark:text-zinc-400 dark:hover:bg-zinc-800"
              >
                Continuă fără actualizare
              </button>
            )}
          </>
        )}

        {state === "downloading" && (
          <div className="flex w-full flex-col items-center gap-3">
            <p className="text-lg font-medium text-zinc-700 dark:text-zinc-300">
              Se descarcă actualizarea...
            </p>
            {version && (
              <p className="text-sm text-zinc-500 dark:text-zinc-400">Versiunea {version}</p>
            )}

            {progress && (
              <div className="w-full">
                <div className="h-2 w-full overflow-hidden rounded-full bg-zinc-200 dark:bg-zinc-700">
                  <div
                    className="h-full rounded-full bg-blue-600 transition-all duration-200 dark:bg-blue-500"
                    style={{ width: percent !== null ? `${percent}%` : "100%" }}
                  />
                </div>
                <p className="mt-2 text-sm text-zinc-500 dark:text-zinc-400">
                  {progress.total
                    ? `${percent}% — ${formatMb(progress.downloaded)} din ${formatMb(progress.total)}`
                    : `${formatMb(progress.downloaded)} descărcate`}
                </p>
              </div>
            )}

            <p className="text-xs text-zinc-400 dark:text-zinc-500">
              Nu închide aplicația în timpul instalării.
            </p>
          </div>
        )}

        {state === "failed" && (
          <>
            <p className="text-lg font-medium text-zinc-700 dark:text-zinc-300">
              Actualizare nereușită
            </p>
            <p className="text-sm text-zinc-500 dark:text-zinc-400">{failure}</p>
            <button
              onClick={skip}
              className="rounded-md border border-zinc-300 px-4 py-2 text-sm text-zinc-600 hover:bg-zinc-50 dark:border-zinc-700 dark:text-zinc-400 dark:hover:bg-zinc-800"
            >
              Continuă
            </button>
          </>
        )}
      </div>
    </div>
  );
}
