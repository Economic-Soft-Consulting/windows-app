"use client";

import { useState, useCallback } from "react";
import { BarcodeScanner as Scanner } from "react-barcode-scanner";
import "react-barcode-scanner/polyfill";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { ScanBarcode } from "lucide-react";
import { toast } from "sonner";

interface BarcodeScannerProps {
  onScan: (code: string) => void;
  className?: string;
}

export function BarcodeScanner({ onScan, className }: BarcodeScannerProps) {
  const [isOpen, setIsOpen] = useState(false);

  const handleCapture = useCallback((barcodes: Array<{ rawValue: string }>) => {
    if (barcodes.length > 0 && barcodes[0]?.rawValue) {
      const code = barcodes[0].rawValue;
      toast.success(`Cod scanat: ${code}`);
      onScan(code);
      setIsOpen(false);
    }
  }, [onScan]);

  return (
    <>
      <Button
        type="button"
        variant="outline"
        size="icon"
        className={`h-14 w-14 shrink-0 ${className ?? ""}`}
        onClick={() => setIsOpen(true)}
        aria-label="Scanează cod de bare"
      >
        <ScanBarcode className="h-5 w-5" />
      </Button>

      <Dialog open={isOpen} onOpenChange={setIsOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <ScanBarcode className="h-5 w-5" />
              Scanează Cod de Bare
            </DialogTitle>
          </DialogHeader>

          <div className="space-y-4">
            <div className="w-full h-[300px] bg-muted rounded-lg overflow-hidden">
              {isOpen && (
                <Scanner
                  onCapture={handleCapture}
                  options={{
                    formats: [
                      "ean_13",
                      "ean_8",
                      "code_128",
                      "code_39",
                      "code_93",
                      "upc_a",
                      "upc_e",
                      "itf",
                      "codabar",
                      "qr_code",
                    ],
                  }}
                />
              )}
            </div>

            <p className="text-center text-sm text-muted-foreground">
              Poziționează codul de bare în cadrul de scanare
            </p>
          </div>
        </DialogContent>
      </Dialog>
    </>
  );
}
