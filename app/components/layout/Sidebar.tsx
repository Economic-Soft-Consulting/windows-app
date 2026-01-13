"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { FileText, Database, Home } from "lucide-react";
import { cn } from "@/lib/utils";

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
];

export function Sidebar() {
  const pathname = usePathname();

  return (
    <aside className="w-64 bg-card border-r border-border flex flex-col">
      {/* Logo */}
      <div className="p-6 border-b border-border">
        <h1 className="text-xl font-bold text-primary">eSoft Facturi</h1>
        <p className="text-sm text-muted-foreground mt-1">Gestiune facturi</p>
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
                  className={cn(
                    "flex items-center gap-3 px-4 py-3 rounded-lg text-base font-medium transition-colors",
                    "hover:bg-accent hover:text-accent-foreground",
                    "active:scale-[0.98]",
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
  );
}
