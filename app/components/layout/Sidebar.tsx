"use client";

import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import Image from "next/image";
import { FileText, Database, Home, X, Settings, LogOut } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { useAuth } from "@/app/contexts/AuthContext";

const navItems = [
  {
    href: "/",
    label: "Acasă",
    icon: Home,
    requiresAdmin: false,
  },
  {
    href: "/invoices",
    label: "Facturi",
    icon: FileText,
    requiresAdmin: false,
  },
  {
    href: "/data",
    label: "Date",
    icon: Database,
    requiresAdmin: false,
  },
  {
    href: "/settings",
    label: "Setări",
    icon: Settings,
    requiresAdmin: true, // Only admin can access
  },
];

interface SidebarProps {
  isOpen?: boolean;
  onClose?: () => void;
}

export function Sidebar({ isOpen = true, onClose }: SidebarProps) {
  const pathname = usePathname();
  const router = useRouter();
  const { isAdmin, isAgent, userRole, logout } = useAuth();

  const handleNavClick = () => {
    // Close sidebar on mobile when navigating
    if (onClose) {
      onClose();
    }
  };

  const handleLogout = () => {
    logout();
    router.push("/login");
  };

  // Filter nav items based on role
  const visibleNavItems = navItems.filter(item => {
    if (item.requiresAdmin) {
      return isAdmin; // Only show to admin
    }
    return true; // Show to everyone
  });

  return (
    <>
      {/* Overlay for mobile */}
      {isOpen && onClose && (
        <div
          className="fixed inset-0 bg-black/50 z-40 md:hidden"
          onClick={onClose}
        />
      )}

      {/* Sidebar - Reduced width from w-72 to w-60 for compact design */}
      <aside
        className={cn(
          "bg-card border-r border-border flex flex-col z-50",
          // Desktop/Tablet: static sidebar
          "md:relative md:w-64 md:translate-x-0 h-full",
          // Mobile: overlay sidebar
          "fixed inset-y-0 left-0 w-64 transition-transform duration-300 ease-in-out md:transition-none",
          isOpen ? "translate-x-0" : "-translate-x-full md:translate-x-0"
        )}
      >
        {/* Logo - Reduced padding from p-6 to p-4 */}
        <div className="p-4 border-b border-border flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="relative h-18 w-18 flex-shrink-0">
              <Image
                src="/logo-simbol-transparent.png"
                alt="eSoft Logo"
                fill
                className="object-contain"
                priority
              />
            </div>
            <div>
              <h1 className="text-xl font-bold text-primary">eSoft Facturi</h1>
              <p className="text-sm text-muted-foreground">Gestiune facturi</p>
            </div>
          </div>
          {/* Close button for mobile */}
          {onClose && (
            <Button
              variant="ghost"
              size="icon"
              className="md:hidden h-9 w-9"
              onClick={onClose}
            >
              <X className="h-4 w-4" />
            </Button>
          )}
        </div>

        {/* User Role Badge */}
        <div className="px-6 py-2 bg-muted/30">
          <div className="text-sm font-medium text-muted-foreground">
            {isAdmin ? "👤 Administrator" : "👤 Agent"}
          </div>
        </div>

        {/* Navigation - Reduced padding from p-4 to p-3 */}
        <nav className="flex-1 p-3">
          <ul className="space-y-1">
            {visibleNavItems.map((item) => {
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
                      "flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium transition-colors",
                      "hover:bg-accent hover:text-accent-foreground",
                      "active:scale-[0.98]",
                      "min-h-[40px]", // Reduced from 44px
                      isActive
                        ? "bg-primary text-primary-foreground"
                        : "text-muted-foreground"
                    )}
                  >
                    <item.icon className="h-6 w-7" />
                    {item.label}
                  </Link>
                </li>
              );
            })}
          </ul>
        </nav>

        {/* Logout Button */}
        <div className="p-3 border-t border-border">
          <Button
            onClick={handleLogout}
            variant="outline"
            className="w-full justify-start gap-2 h-10"
          >
            <LogOut className="h-6 w-7" />
            Schimbă utilizator
          </Button>
        </div>

        {/* Footer - Reduced padding */}
        <div className="p-3 border-t border-border">
          <p className="text-sm text-muted-foreground text-center">
            v0.7.4 • © 2026 eSoft
          </p>
        </div>
      </aside>
    </>
  );
}
