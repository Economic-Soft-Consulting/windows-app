"use client";

import { Card, CardContent } from "@/components/ui/card";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { Label } from "@/components/ui/label";
import { MapPin, Check } from "lucide-react";
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
  console.log("LocationStep - partner locations:", partner.locations);
  console.log("LocationStep - selectedLocation:", selectedLocation);
  
  return (
    <div className="space-y-6">
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
        <RadioGroup
          value={selectedLocation?.id ?? ""}
          onValueChange={(value) => {
            console.log("RadioGroup onValueChange:", value);
            const location = partner.locations.find((l) => l.id === value);
            console.log("Found location:", location);
            if (location) onSelect(location);
          }}
          className="grid gap-3 sm:grid-cols-2 md:grid-cols-2"
        >
          {partner.locations.map((location) => {
            const isSelected = selectedLocation?.id === location.id;
            return (
              <Label
                key={location.id}
                htmlFor={location.id}
                className="cursor-pointer"
              >
              <Card
                className={cn(
                  "transition-all hover:border-primary/50",
                  isSelected && "border-primary bg-primary/5 ring-2 ring-primary"
                )}
              >
                <CardContent className="p-4">
                  <div className="flex items-start gap-3">
                    <RadioGroupItem
                      value={location.id}
                      id={location.id}
                      className="mt-1"
                    />
                    <div className="flex-1 min-w-0">
                      <div className="flex items-start justify-between">
                        <div className="flex items-center gap-2">
                          <MapPin className="h-4 w-4 text-muted-foreground flex-shrink-0" />
                          <span className="font-medium">{location.name}</span>
                        </div>
                        {isSelected && (
                          <div className="h-6 w-6 rounded-full bg-primary flex items-center justify-center flex-shrink-0">
                            <Check className="h-4 w-4 text-primary-foreground" />
                          </div>
                        )}
                      </div>
                      {location.address && (
                        <p className="text-sm text-muted-foreground mt-1 ml-6">
                          {location.address}
                        </p>
                      )}
                    </div>
                  </div>
                </CardContent>
              </Card>
            </Label>
          );
          })}
        </RadioGroup>
      )}
    </div>
  );
}
