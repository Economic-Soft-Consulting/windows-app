"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { PartnerStep } from "./PartnerStep";
import { LocationStep } from "./LocationStep";
import { ProductsStep } from "./ProductsStep";
import { ReviewStep } from "./ReviewStep";
import { createInvoice, sendInvoice } from "@/lib/tauri/commands";
import { toast } from "sonner";
import {
  ArrowLeft,
  ArrowRight,
  Check,
  Building2,
  MapPin,
  Package,
  FileText,
  Loader2,
  Send,
} from "lucide-react";
import { cn } from "@/lib/utils";
import type {
  PartnerWithLocations,
  Location,
  CartItem,
  CreateInvoiceRequest,
} from "@/lib/tauri/types";

type Step = 1 | 2 | 3 | 4;

const steps = [
  { number: 1 as Step, title: "Partener", icon: Building2 },
  { number: 2 as Step, title: "Locație", icon: MapPin },
  { number: 3 as Step, title: "Produse", icon: Package },
  { number: 4 as Step, title: "Revizuire", icon: FileText },
];

export function InvoiceWizard() {
  const router = useRouter();
  const [currentStep, setCurrentStep] = useState<Step>(1);
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Form state
  const [selectedPartner, setSelectedPartner] = useState<PartnerWithLocations | null>(null);
  const [selectedLocation, setSelectedLocation] = useState<Location | null>(null);
  const [cartItems, setCartItems] = useState<CartItem[]>([]);
  const [notes, setNotes] = useState("");

  const canGoNext = () => {
    switch (currentStep) {
      case 1:
        return selectedPartner !== null;
      case 2:
        return selectedLocation !== null;
      case 3:
        return cartItems.length > 0;
      case 4:
        return true;
      default:
        return false;
    }
  };

  const handleNext = () => {
    if (currentStep < 4) {
      setCurrentStep((currentStep + 1) as Step);
    }
  };

  const handleBack = () => {
    if (currentStep > 1) {
      setCurrentStep((currentStep - 1) as Step);
    }
  };

  const handlePartnerSelect = (partner: PartnerWithLocations) => {
    setSelectedPartner(partner);
    // Reset location if partner changed
    if (selectedPartner?.id !== partner.id) {
      setSelectedLocation(null);
    }
  };

  const handleSubmit = async () => {
    if (!selectedPartner || !selectedLocation || cartItems.length === 0) {
      return;
    }

    setIsSubmitting(true);

    try {
      const request: CreateInvoiceRequest = {
        partner_id: selectedPartner.id,
        location_id: selectedLocation.id,
        notes: notes || undefined,
        items: cartItems.map((item) => ({
          product_id: item.product.id,
          quantity: item.quantity,
        })),
      };

      // Create the invoice
      const invoice = await createInvoice(request);
      toast.success("Factura a fost creată cu succes!");

      // Immediately try to send
      try {
        const sentInvoice = await sendInvoice(invoice.id);
        if (sentInvoice.status === "sent") {
          toast.success("Factura a fost trimisă cu succes!");
        } else if (sentInvoice.status === "failed") {
          toast.error(`Eroare la trimitere: ${sentInvoice.error_message}`);
        }
      } catch (e) {
        toast.error("Eroare la trimiterea facturii");
      }

      // Navigate to invoices list
      router.push("/invoices");
    } catch (e) {
      toast.error(`Eroare la crearea facturii: ${e}`);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="space-y-6">
      {/* Step Indicator */}
      <div className="flex items-center justify-center">
        <div className="flex items-center gap-2">
          {steps.map((step, index) => {
            const isActive = currentStep === step.number;
            const isCompleted = currentStep > step.number;
            const Icon = step.icon;

            return (
              <div key={step.number} className="flex items-center">
                <div
                  className={cn(
                    "flex items-center gap-2 px-4 py-2 rounded-full transition-colors",
                    isActive && "bg-primary text-primary-foreground",
                    isCompleted && "bg-primary/20 text-primary",
                    !isActive && !isCompleted && "bg-muted text-muted-foreground"
                  )}
                >
                  {isCompleted ? (
                    <Check className="h-4 w-4" />
                  ) : (
                    <Icon className="h-4 w-4" />
                  )}
                  <span className="text-sm font-medium hidden sm:inline">
                    {step.title}
                  </span>
                </div>
                {index < steps.length - 1 && (
                  <div
                    className={cn(
                      "w-8 h-0.5 mx-1",
                      currentStep > step.number ? "bg-primary" : "bg-muted"
                    )}
                  />
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Step Content */}
      <Card>
        <CardContent className="p-6">
          {currentStep === 1 && (
            <PartnerStep
              selectedPartner={selectedPartner}
              onSelect={handlePartnerSelect}
            />
          )}
          {currentStep === 2 && selectedPartner && (
            <LocationStep
              partner={selectedPartner}
              selectedLocation={selectedLocation}
              onSelect={setSelectedLocation}
            />
          )}
          {currentStep === 3 && (
            <ProductsStep cartItems={cartItems} onUpdateCart={setCartItems} />
          )}
          {currentStep === 4 && selectedPartner && selectedLocation && (
            <ReviewStep
              partner={selectedPartner}
              location={selectedLocation}
              cartItems={cartItems}
              notes={notes}
              onNotesChange={setNotes}
            />
          )}
        </CardContent>
      </Card>

      {/* Navigation */}
      <div className="flex items-center justify-between">
        <Button
          variant="outline"
          size="lg"
          onClick={handleBack}
          disabled={currentStep === 1}
          className="gap-2 h-12"
        >
          <ArrowLeft className="h-4 w-4" />
          Înapoi
        </Button>

        {currentStep < 4 ? (
          <Button
            size="lg"
            onClick={handleNext}
            disabled={!canGoNext()}
            className="gap-2 h-12"
          >
            Continuă
            <ArrowRight className="h-4 w-4" />
          </Button>
        ) : (
          <Button
            size="lg"
            onClick={handleSubmit}
            disabled={isSubmitting || !canGoNext()}
            className="gap-2 h-12"
          >
            {isSubmitting ? (
              <>
                <Loader2 className="h-4 w-4 animate-spin" />
                Se procesează...
              </>
            ) : (
              <>
                <Send className="h-4 w-4" />
                Salvează și Trimite
              </>
            )}
          </Button>
        )}
      </div>
    </div>
  );
}
