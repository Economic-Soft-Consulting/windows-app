"use client";

import { useMemo, useState } from "react";
import { Card, CardContent } from "@/components/ui/card";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { MapPin, Check, Search } from "lucide-react";
import { cn } from "@/lib/utils";
import type { PartnerWithLocations, Location } from "@/lib/tauri/types";

interface LocationStepProps {
  partner: PartnerWithLocations;
  selectedLocation: Location | null;
  onSelect: (location: Location) => void;
}

export function LocationStep({
  partner,
  selectedLocation,
  onSelect,
}: LocationStepProps) {
  const [searchQuery, setSearchQuery] = useState("");

  const filteredLocations = useMemo(() => {
    const normalizedQuery = searchQuery.trim().toLowerCase();
    if (!normalizedQuery) {
      return partner.locations;
    }

    return partner.locations.filter((location) => {
      const searchableText = [
        location.name,
        location.address,
        location.localitate,
        location.judet,
      ]
        .filter(Boolean)
        .join(" ")
        .toLowerCase();

      return searchableText.includes(normalizedQuery);
    });
  }, [partner.locations, searchQuery]);

  return (
    <div className="space-y-4">
      <div>
        <h2 className="text-xl font-semibold">Selectează locația</h2>
        <p className="text-muted-foreground mt-1">
          Alege locația pentru <span className="font-medium">{partner.name}</span>
        </p>
      </div>

      {partner.locations.length === 0 ? (
        <Card>
          <CardContent className="p-6 text-center text-muted-foreground">
            <MapPin className="h-12 w-12 mx-auto mb-2 opacity-50" />
            <p>Acest partener nu are locații configurate.</p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-3">
          <div className="relative">
            <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground" />
            <Input
              placeholder="Caută locație după nume, adresă, localitate..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-9 h-9 text-sm"
            />
          </div>

          {filteredLocations.length === 0 ? (
            <Card>
              <CardContent className="p-6 text-center text-muted-foreground">
                <MapPin className="h-10 w-10 mx-auto mb-2 opacity-50" />
                <p>Nu s-au găsit locații pentru căutarea introdusă.</p>
              </CardContent>
            </Card>
          ) : (
            <ScrollArea className="h-[420px] pr-3">
              <RadioGroup
                value={selectedLocation?.id ?? ""}
                onValueChange={(value) => {
                  const location = partner.locations.find((l) => l.id === value);
                  if (location) onSelect(location);
                }}
                className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3 auto-rows-fr"
              >
                {filteredLocations.map((location) => {
                  const isSelected = selectedLocation?.id === location.id;
                  const subtitle = [location.address, location.localitate, location.judet]
                    .filter((value) => value && value.trim().length > 0)
                    .join(" • ");

                  return (
                    <Label key={location.id} htmlFor={location.id} className="cursor-pointer block h-full">
                      <Card
                        className={cn(
                          "transition-all hover:border-primary/50 h-full min-h-[96px]",
                          isSelected && "border-primary bg-primary/5 ring-2 ring-primary"
                        )}
                      >
                        <CardContent className="px-3 py-2.5 h-full">
                          <div className="flex items-start gap-2">
                            <RadioGroupItem value={location.id} id={location.id} className="mt-1" />

                            <MapPin className="h-4.5 w-4.5 text-muted-foreground mt-0.5 flex-shrink-0" />

                            <div className="flex-1 min-w-0">
                              <div className="flex items-start justify-between gap-3">
                                <p className="text-sm font-medium leading-5 break-words line-clamp-2">{location.name}</p>
                                {isSelected && (
                                  <div className="h-5 w-5 rounded-full bg-primary flex items-center justify-center flex-shrink-0">
                                    <Check className="h-3 w-3 text-primary-foreground" />
                                  </div>
                                )}
                              </div>

                              {subtitle && (
                                <p className="text-xs text-muted-foreground mt-0.5 break-words line-clamp-2">{subtitle}</p>
                              )}
                            </div>
                          </div>
                        </CardContent>
                      </Card>
                    </Label>
                  );
                })}
              </RadioGroup>
            </ScrollArea>
          )}
        </div>
      )}
    </div>
  );
}
