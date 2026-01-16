"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { FileText, Database, Home, X, Settings, Wifi } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";

const navItems = [
  {
    href: "/",
    label: "Acasă",
    icon: Home,
  },
  {
    href: "/invoices",
    label: "Facturi",
    icon: FileText,
  },
  {
    href: "/data",
    label: "Date",
    icon: Database,
  },
  {
    href: "/settings",
    label: "Setări",
    icon: Settings,
  },
  {
    href: "/api-settings",
    label: "API",
    icon: Wifi,
  },
];

interface SidebarProps {
  isOpen?: boolean;
  onClose?: () => void;
}

export function Sidebar({ isOpen = true, onClose }: SidebarProps) {
  const pathname = usePathname();

  const handleNavClick = () => {
    // Close sidebar on mobile when navigating
    if (onClose) {
      onClose();
    }
  };

  return (
    <>
      {/* Overlay for mobile */}
      {isOpen && onClose && (
        <div
          className="fixed inset-0 bg-black/50 z-40 lg:hidden"
          onClick={onClose}
        />
      )}

      {/* Sidebar */}
      <aside
        className={cn(
          "bg-card border-r border-border flex flex-col z-50",
          // Desktop: static sidebar
          "lg:relative lg:w-64 lg:translate-x-0",
          // Mobile: overlay sidebar
          "fixed inset-y-0 left-0 w-72 transition-transform duration-300 ease-in-out lg:transition-none",
          isOpen ? "translate-x-0" : "-translate-x-full lg:translate-x-0"
        )}
      >
        {/* Logo */}
        <div className="p-6 border-b border-border flex items-center justify-between">
          <div>
            <h1 className="text-xl font-bold text-primary">eSoft Facturi</h1>
            <p className="text-sm text-muted-foreground mt-1">Gestiune facturi</p>
          </div>
          {/* Close button for mobile */}
          {onClose && (
            <Button
              variant="ghost"
              size="icon"
              className="lg:hidden h-11 w-11"
              onClick={onClose}
            >
              <X className="h-5 w-5" />
            </Button>
          )}
        </div>

        {/* Navigation */}
        <nav className="flex-1 p-4">
          <ul className="space-y-2">
            {navItems.map((item) => {
              const isActive =
                item.href === "/"
                  ? pathname === "/"
                  : pathname.startsWith(item.href);

              return (
                <li key={item.href}>
                  <Link
                    href={item.href}
                    onClick={handleNavClick}
                    className={cn(
                      "flex items-center gap-3 px-4 py-3 rounded-lg text-base font-medium transition-colors",
                      "hover:bg-accent hover:text-accent-foreground",
                      "active:scale-[0.98]",
                      "min-h-[44px]", // Ensure touch target
                      isActive
                        ? "bg-primary text-primary-foreground"
                        : "text-muted-foreground"
                    )}
                  >
                    <item.icon className="h-5 w-5" />
                    {item.label}
                  </Link>
                </li>
              );
            })}
          </ul>
        </nav>

        {/* Footer */}
        <div className="p-4 border-t border-border">
          <p className="text-xs text-muted-foreground text-center">
            v0.1.0 • © 2026 eSoft
          </p>
        </div>
      </aside>
    </>
  );
}
