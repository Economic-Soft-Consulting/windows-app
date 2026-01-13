"use client";

import { NetworkIndicator } from "./NetworkIndicator";
import { SyncButton } from "../sync/SyncButton";

interface HeaderProps {
  title?: string;
}

export function Header({ title }: HeaderProps) {
  return (
    <header className="h-16 bg-card border-b border-border flex items-center justify-between px-6">
      <div>
        {title && <h2 className="text-lg font-semibold">{title}</h2>}
      </div>
      <div className="flex items-center gap-4">
        <SyncButton />
        <NetworkIndicator />
      </div>
    </header>
  );
}
