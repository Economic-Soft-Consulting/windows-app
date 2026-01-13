"use client";

import { Menu } from "lucide-react";
import { Button } from "@/components/ui/button";
import { NetworkIndicator } from "./NetworkIndicator";
import { SyncButton } from "../sync/SyncButton";

interface HeaderProps {
  title?: string;
  onMenuClick?: () => void;
}

export function Header({ title, onMenuClick }: HeaderProps) {
  return (
    <header className="h-16 bg-card border-b border-border flex items-center justify-between px-4 sm:px-6">
      <div className="flex items-center gap-3">
        {/* Hamburger menu button - visible on mobile/tablet */}
        {onMenuClick && (
          <Button
            variant="ghost"
            size="icon"
            className="lg:hidden h-11 w-11"
            onClick={onMenuClick}
          >
            <Menu className="h-5 w-5" />
          </Button>
        )}
        {title && <h2 className="text-lg font-semibold">{title}</h2>}
      </div>
      <div className="flex items-center gap-2 sm:gap-4">
        <SyncButton />
        <NetworkIndicator />
      </div>
    </header>
  );
}
