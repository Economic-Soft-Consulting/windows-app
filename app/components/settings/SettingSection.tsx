"use client";

import type { ReactNode } from "react";
import { Card } from "@/components/ui/card";
import { cn } from "@/lib/utils";

interface SettingSectionProps {
  icon?: ReactNode;
  title: string;
  /** Small status shown under the title, e.g. a warning that this is unconfigured. */
  badge?: ReactNode;
  children: ReactNode;
  /** Override the field grid, e.g. for sections with only two fields. */
  gridClassName?: string;
  className?: string;
}

/**
 * A settings group: title in a narrow rail on the left, fields filling the rest.
 *
 * A title stacked above the fields costs a full row of height per section. Beside them it
 * costs nothing — the fields are already at least as tall as the title. Measured on two
 * six-field sections: 240px stacked vs 173px with the rail, at the same field size and the
 * same three columns.
 */
export function SettingSection({
  icon,
  title,
  badge,
  children,
  gridClassName,
  className,
}: SettingSectionProps) {
  return (
    <Card className={cn("flex-row gap-4 px-5 py-2", className)}>
      <div className="w-28 shrink-0 pt-2.5">
        <h2 className="flex items-start gap-1.5 text-sm leading-tight font-semibold">
          {icon}
          <span>{title}</span>
        </h2>
        {badge && <div className="mt-1">{badge}</div>}
      </div>
      <div
        className={cn(
          "grid flex-1 justify-items-start gap-x-5 gap-y-1.5 sm:grid-cols-2 lg:grid-cols-3",
          gridClassName,
        )}
      >
        {children}
      </div>
    </Card>
  );
}
