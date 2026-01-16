"use client";

import { useState, useEffect } from "react";
import { Settings, Loader2, Printer, FileText } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { getAvailablePrinters } from "@/lib/tauri/commands";
import { toast } from "sonner";

interface PrintSettings {
  printer: string;
  copies: number;
  autoPrint: boolean;
  showPreview: boolean;
  paperWidth: string;
}

export default function SettingsPage() {
  const [printers, setPrinters] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [settings, setSettings] = useState<PrintSettings>({
    printer: "",
    copies: 1,
    autoPrint: true,
    showPreview: false,
    paperWidth: "80mm",
  });

  useEffect(() => {
    loadSettings();
    loadCachedPrinters();
    loadPrinters();
  }, []);

  const loadSettings = () => {
    const saved = localStorage.getItem("printSettings");
    if (saved) {
      try {
        const parsed = JSON.parse(saved);
        setSettings(parsed);
      } catch (e) {
        console.error("Failed to parse settings:", e);
      }
    }
  };

  const loadCachedPrinters = () => {
    const cached = localStorage.getItem("printersCache");
    if (cached) {
      try {
        const parsed = JSON.parse(cached);
        if (Array.isArray(parsed) && parsed.length > 0) {
          setPrinters(parsed);
          // Use cached list to select saved printer quickly
          const saved = localStorage.getItem("printSettings");
          if (saved) {
            const settingsParsed = JSON.parse(saved);
            if (settingsParsed.printer && parsed.includes(settingsParsed.printer)) {
              setSettings(prev => ({ ...prev, printer: settingsParsed.printer }));
            }
          }
          setLoading(false);
        }
      } catch (e) {
        console.error("Failed to parse cached printers:", e);
      }
    }
  };

  const loadPrinters = async () => {
    if (printers.length === 0) {
      setLoading(true);
    }
    try {
      const list = await getAvailablePrinters();
      setPrinters(list);
      localStorage.setItem("printersCache", JSON.stringify(list));
      
      // Get saved printer from settings
      const saved = localStorage.getItem("printSettings");
      if (saved) {
        const parsed = JSON.parse(saved);
        if (parsed.printer && list.includes(parsed.printer)) {
          setSettings(prev => ({ ...prev, printer: parsed.printer }));
        } else if (list.length > 0) {
          setSettings(prev => ({ ...prev, printer: list[0] }));
        }
      } else if (list.length > 0) {
        setSettings(prev => ({ ...prev, printer: list[0] }));
      }
    } catch (error) {
      console.error("Failed to load printers:", error);
      toast.error("Eroare la încărcarea imprimantelor");
      setPrinters(["Default"]);
    } finally {
      setLoading(false);
    }
  };

  const handleSaveSettings = () => {
    localStorage.setItem("printSettings", JSON.stringify(settings));
    // Also save to old key for compatibility
    localStorage.setItem("selectedPrinter", settings.printer);
    toast.success("Setările au fost salvate!");
  };

  const updateSetting = <K extends keyof PrintSettings>(key: K, value: PrintSettings[K]) => {
    setSettings(prev => ({ ...prev, [key]: value }));
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center gap-3">
        <Settings className="h-8 w-8" />
        <div>
          <h1 className="text-3xl font-bold">Setări</h1>
          <p className="text-muted-foreground">
            Configurează aplicația pentru a se potrivi cu preferințele tale
          </p>
        </div>
      </div>

      {/* Printer Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Printer className="h-5 w-5" />
            Setări Printare
          </CardTitle>
          <CardDescription>
            Configurează imprimanta și opțiunile de printare
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          {loading ? (
            <div className="flex items-center gap-2 text-muted-foreground">
              <Loader2 className="h-5 w-5 animate-spin" />
              Se încarcă imprimantele...
            </div>
          ) : (
            <>
              {/* Printer Selection */}
              <div className="space-y-2">
                <Label htmlFor="printer">Imprimantă</Label>
                <Select value={settings.printer} onValueChange={(v) => updateSetting("printer", v)}>
                  <SelectTrigger id="printer">
                    <SelectValue placeholder="Selectează o imprimantă" />
                  </SelectTrigger>
                  <SelectContent>
                    {printers.map((printer) => (
                      <SelectItem key={printer} value={printer}>
                        {printer}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {/* Number of Copies */}
              <div className="space-y-2">
                <Label htmlFor="copies">Număr de copii</Label>
                <Input
                  id="copies"
                  type="number"
                  min="1"
                  max="10"
                  value={settings.copies}
                  onChange={(e) => updateSetting("copies", parseInt(e.target.value) || 1)}
                />
                <p className="text-sm text-muted-foreground">
                  Câte copii ale facturii să se printeze automat
                </p>
              </div>

              {/* Paper Width */}
              <div className="space-y-2">
                <Label htmlFor="paperWidth">Lățime hârtie</Label>
                <Select value={settings.paperWidth} onValueChange={(v) => updateSetting("paperWidth", v)}>
                  <SelectTrigger id="paperWidth">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="58mm">58mm (Mic)</SelectItem>
                    <SelectItem value="80mm">80mm (Standard)</SelectItem>
                    <SelectItem value="A4">A4 (Lățime completă)</SelectItem>
                  </SelectContent>
                </Select>
                <p className="text-sm text-muted-foreground">
                  Template-ul actual este optimizat pentru 80mm
                </p>
              </div>

              {/* Auto Print Toggle */}
              <div className="flex items-center justify-between space-x-2 rounded-lg border p-4">
                <div className="space-y-0.5">
                  <Label htmlFor="autoPrint" className="text-base cursor-pointer">
                    Printare automată
                  </Label>
                  <p className="text-sm text-muted-foreground">
                    Printează automat factura după salvare
                  </p>
                </div>
                <Switch
                  id="autoPrint"
                  checked={settings.autoPrint}
                  onCheckedChange={(v) => updateSetting("autoPrint", v)}
                />
              </div>

              {/* Show Preview Toggle */}
              <div className="flex items-center justify-between space-x-2 rounded-lg border p-4">
                <div className="space-y-0.5">
                  <Label htmlFor="showPreview" className="text-base cursor-pointer">
                    Previzualizare PDF
                  </Label>
                  <p className="text-sm text-muted-foreground">
                    Deschide PDF-ul generat înainte de printare (nu recomandat)
                  </p>
                </div>
                <Switch
                  id="showPreview"
                  checked={settings.showPreview}
                  onCheckedChange={(v) => updateSetting("showPreview", v)}
                />
              </div>

              {/* Action Buttons */}
              <div className="flex gap-3">
                <Button onClick={handleSaveSettings} className="flex-1">
                  Salvează setările
                </Button>
                <Button onClick={loadPrinters} variant="outline">
                  Reîncarcă imprimante
                </Button>
              </div>

              {printers.length === 0 && (
                <p className="text-sm text-red-600">
                  Nu au fost găsite imprimante. Asigură-te că ai instalat o imprimantă.
                </p>
              )}
            </>
          )}
        </CardContent>
      </Card>

      {/* Info Section */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <FileText className="h-5 w-5" />
            Informații
          </CardTitle>
        </CardHeader>
        <CardContent>
          <ul className="space-y-2 text-sm text-muted-foreground">
            <li>• Versiunea aplicației: 0.3.0</li>
            <li>• Fișierele facturilor sunt salvate în: %APPDATA%\facturi.softconsulting.com\invoices\</li>
            <li>• Suport pentru printare PDF pe imprimantă termală 80mm</li>
            <li>• Printarea se face prin SumatraPDF (instalat automat)</li>
            <li>• Template optimizat pentru bonuri fiscale format 80mm x 297mm</li>
          </ul>
        </CardContent>
      </Card>
    </div>
  );
}
