"use client";

import { Info } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { cn } from "@/lib/utils";

interface SettingFieldProps {
  id: string;
  label: string;
  value: string | number | null | undefined;
  onChange: (value: string) => void;
  placeholder?: string;
  /** Shown on demand behind the (i) button, so it costs no vertical space. */
  hint?: string;
  type?: "text" | "number";
  min?: string;
  disabled?: boolean;
  /** Use for the few free-text values that need more room than a code does. */
  wide?: boolean;
  /**
   * Narrows the field to a short label plus a counter-sized input, and lets it wrap instead
   * of stretching. Use where fields form a row of their own — the numbering rows — rather
   * than sitting in the section's aligned columns.
   */
  compact?: boolean;
  className?: string;
}

/**
 * One labelled setting: bold label on the left, input on the right, on a single line.
 *
 * Stacking the label above the input cost ~50px per field and left a full-width box for a
 * four-character code like "FONG". Inline, a field is one 36px row and the input is sized to
 * its content — roughly 15 characters, which covers every value on this screen including an
 * IP address (192.168.100.100) and a delegate's ID document.
 *
 * The hint sits behind an (i) button rather than a line of text under the field. That button
 * is a Popover, not a Tooltip, because these are touch-first tablets and a hover tooltip is
 * unreachable with a finger.
 */
export function SettingField({
  id,
  label,
  value,
  onChange,
  placeholder,
  hint,
  type = "text",
  min,
  disabled,
  wide,
  compact,
  className,
}: SettingFieldProps) {
  return (
    <div
      className={cn(
        // Fixed label column so the inputs line up down the page, and the input sits right
        // next to its label. Letting the pair stretch pushed the box far from its label.
        "grid items-center",
        // Compact fields sit in a row of their own, so they keep their natural width instead
        // of being squeezed by the row: shrink-0 here, and no truncation on the label below.
        compact
          ? "shrink-0 gap-2 grid-cols-[minmax(104px,auto)_84px]"
          : cn("gap-2.5", wide ? "grid-cols-[150px_190px]" : "grid-cols-[150px_130px]"),
        className,
      )}
    >
      <div className={cn("flex items-center gap-1", compact ? "whitespace-nowrap" : "min-w-0")}>
        <Label htmlFor={id} className={cn("text-sm font-semibold", !compact && "truncate")}>
          {label}
        </Label>
        {hint && (
          <Popover>
            <PopoverTrigger
              aria-label={`Ce înseamnă ${label}`}
              className="shrink-0 rounded-full text-muted-foreground/70 hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            >
              <Info className="h-3.5 w-3.5" />
            </PopoverTrigger>
            <PopoverContent side="top" className="w-64 text-sm leading-snug">
              {hint}
            </PopoverContent>
          </Popover>
        )}
      </div>
      <Input
        id={id}
        type={type}
        min={min}
        placeholder={placeholder}
        disabled={disabled}
        className={cn("w-full", disabled && "bg-muted")}
        value={value ?? ""}
        onChange={(e) => onChange(e.target.value)}
      />
    </div>
  );
}
