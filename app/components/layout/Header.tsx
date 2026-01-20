"use client";

import { Menu, LogOut, User } from "lucide-react";
import { Button } from "@/components/ui/button";
import { NetworkIndicator } from "./NetworkIndicator";
import { SyncButton } from "../sync/SyncButton";
import { useAuth } from "@/app/contexts/AuthContext";
import { useRouter } from "next/navigation";

interface HeaderProps {
  title?: string;
  onMenuClick?: () => void;
}

export function Header({ title, onMenuClick }: HeaderProps) {
  const { isAdmin, logout } = useAuth();
  const router = useRouter();

  const handleLogout = () => {
    logout();
    router.push("/login");
  };

  return (
    <header className="h-14 bg-card border-b border-border flex items-center justify-between px-4 sm:px-6">
      <div className="flex items-center gap-3">
        {/* Hamburger menu button - visible on mobile/tablet */}
        {onMenuClick && (
          <Button
            variant="ghost"
            size="icon"
            className="lg:hidden h-9 w-9"
            onClick={onMenuClick}
          >
            <Menu className="h-4 w-4" />
          </Button>
        )}
        {title && <h2 className="text-base font-semibold">{title}</h2>}
      </div>
      <div className="flex items-center gap-2 sm:gap-3">
        {/* User Role Badge */}
        <div className="hidden sm:flex items-center gap-2 px-3 py-1.5 bg-muted rounded-md">
          <User className="h-3.5 w-3.5 text-muted-foreground" />
          <span className="text-xs font-medium text-muted-foreground">
            {isAdmin ? "Admin" : "Agent"}
          </span>
        </div>

        <SyncButton />
        <NetworkIndicator />

        {/* Switch User Button */}
        <Button
          onClick={handleLogout}
          variant="outline"
          size="sm"
          className="gap-2 h-9"
        >
          <LogOut className="h-3.5 w-3.5" />
          <span className="hidden sm:inline">Schimbă</span>
        </Button>
      </div>
    </header>
  );
}
