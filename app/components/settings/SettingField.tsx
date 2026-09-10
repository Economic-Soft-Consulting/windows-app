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
  className?: string;
}

/**
 * One labelled setting.
 *
 * The settings page used to repeat a Label/Input/description triple fifteen times — 284 lines
 * of the same block, and the always-visible description doubled the height of every row. The
 * hint now lives behind an (i) button.
 *
 * That button is a Popover rather than a Tooltip on purpose: these tablets are touch-first,
 * and a hover tooltip is unreachable with a finger.
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
  className,
}: SettingFieldProps) {
  return (
    <div className={cn("space-y-1", className)}>
      <div className="flex items-center gap-1">
        <Label htmlFor={id} className="text-sm">
          {label}
        </Label>
        {hint && (
          <Popover>
            <PopoverTrigger
              aria-label={`Ce înseamnă ${label}`}
              className="text-muted-foreground/70 hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring rounded-full"
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
        className={cn(disabled && "bg-muted")}
        value={value ?? ""}
        onChange={(e) => onChange(e.target.value)}
      />
    </div>
  );
}
