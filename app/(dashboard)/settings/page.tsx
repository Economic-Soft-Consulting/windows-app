"use client";

import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import { Settings, Loader2, Printer, FileText, User, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { getAvailablePrinters, getAgentSettings, saveAgentSettings } from "@/lib/tauri/commands";
import type { AgentSettings } from "@/lib/tauri/types";
import { toast } from "sonner";
import { useSyncStatus } from "@/hooks/useSyncStatus";
import { useOnlineStatus } from "@/hooks/useOnlineStatus";
import { useAuth } from "@/app/contexts/AuthContext";

interface PrintSettings {
  printer: string;
  copies: number;
  autoPrint: boolean;
  showPreview: boolean;
  paperWidth: string;
}

export default function SettingsPage() {
  const { isAdmin } = useAuth();
  const router = useRouter();
  const [printers, setPrinters] = useState<string[]>([]);
  const [loadingPrinters, setLoadingPrinters] = useState(true);
  const [settings, setSettings] = useState<PrintSettings>({
    printer: "",
    copies: 1,
    autoPrint: true,
    showPreview: false,
    paperWidth: "80mm",
  });
  const [agentSettings, setAgentSettings] = useState<AgentSettings>({
    agent_name: null,
    carnet_series: null,
    simbol_carnet_livr: null,
    simbol_gestiune_livrare: null,
    cod_carnet: null,
    cod_carnet_livr: null,
    delegate_name: null,
    delegate_act: null,
    invoice_number_start: null,
    invoice_number_end: null,
    invoice_number_current: null,
  });
  const [savingAgent, setSavingAgent] = useState(false);
  const [loadingAgentSettings, setLoadingAgentSettings] = useState(true);

  const { status, isSyncing, triggerSync } = useSyncStatus();
  const { isOnline } = useOnlineStatus();

  // Redirect non-admin users
  useEffect(() => {
    if (!isAdmin) {
      toast.error("Acces interzis - doar pentru Administrator");
      router.push("/");
    }
  }, [isAdmin, router]);

  const handleSyncNow = async () => {
    if (!isOnline) {
      toast.error("Nu există conexiune la internet");
      return;
    }

    try {
      await triggerSync();
      toast.success("Datele au fost sincronizate cu succes!");
    } catch (e) {
      console.error("Sync error:", e);
      toast.error(`Eroare la sincronizare: ${e}`);
    }
  };

  const formatLastSync = (dateStr: string | null) => {
    if (!dateStr) return "Niciodată";
    const date = new Date(dateStr);
    return date.toLocaleString("ro-RO", {
      day: "2-digit",
      month: "2-digit",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  useEffect(() => {
    // Load settings and cached data immediately
    loadSettings();
    loadCachedPrinters();
    loadAgentSettings();

    // Load printers - this is the slow operation
    loadPrinters();
  }, []);

  const loadAgentSettings = async () => {
    setLoadingAgentSettings(true);
    try {
      const settings = await getAgentSettings();
      setAgentSettings(settings);
    } catch (error) {
      console.error("Failed to load agent settings:", error);
    } finally {
      setLoadingAgentSettings(false);
    }
  };

  const handleSaveAgentSettings = async () => {
    setSavingAgent(true);
    try {
      await saveAgentSettings(
        agentSettings.agent_name || null,
        agentSettings.carnet_series || null,
        agentSettings.simbol_carnet_livr || null,
        agentSettings.simbol_gestiune_livrare || null,
        agentSettings.cod_carnet || null,
        agentSettings.cod_carnet_livr || null,
        agentSettings.delegate_name || null,
        agentSettings.delegate_act || null,
        agentSettings.invoice_number_start,
        agentSettings.invoice_number_end,
        agentSettings.invoice_number_current
      );
      // Reload settings to update UI with latest values from database
      await loadAgentSettings();
      toast.success("Setările agentului au fost salvate!");
    } catch (error) {
      console.error("Failed to save agent settings:", error);
      toast.error("Eroare la salvarea setărilor agentului");
    } finally {
      setSavingAgent(false);
    }
  };

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
          // Note: Don't set loading to false here - wait for actual printer fetch
        }
      } catch (e) {
        console.error("Failed to parse cached printers:", e);
      }
    }
  };

  const loadPrinters = async () => {
    setLoadingPrinters(true);
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
        } else if (list.length > 0 && !parsed.printer) {
          setSettings(prev => ({ ...prev, printer: list[0] }));
        }
      } else if (list.length > 0) {
        setSettings(prev => ({ ...prev, printer: list[0] }));
      }
    } catch (error) {
      console.error("Failed to load printers:", error);
      // Only show error if we don't have cached printers
      if (printers.length === 0) {
        toast.error("Eroare la încărcarea imprimantelor");
        setPrinters(["Default"]);
      }
    } finally {
      setLoadingPrinters(false);
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

  // Show full page loading while all settings are loading initially
  if (loadingPrinters || loadingAgentSettings) {
    return (
      <div className="h-full flex flex-col items-center justify-center gap-4">
        <Loader2 className="h-12 w-12 animate-spin text-primary" />
        <div className="text-center">
          <h2 className="text-lg font-semibold">Se încarcă setările...</h2>
          <p className="text-muted-foreground text-sm">
            {loadingPrinters
              ? "Se verifică imprimantele disponibile"
              : "Se încarcă configurările agentului"
            }
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col space-y-6">
      {/* Header */}
      <div className="flex items-center gap-3 shrink-0">
        <Settings className="h-8 w-8" />
        <div>
          <h1 className="text-3xl font-bold">Setări</h1>
          <p className="text-muted-foreground">
            Configurează aplicația pentru a se potrivi cu preferințele tale
          </p>
        </div>
      </div>

      {/* Scrollable content */}
      <div className="flex-1 overflow-y-auto min-h-0 space-y-6">
        {/* Agent Settings */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <User className="h-5 w-5" />
              Date Agent
            </CardTitle>
            <CardDescription>
              Configurează informațiile agentului care va fi folosit la trimiterea facturilor
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">          <div className="space-y-2">
            <Label htmlFor="agentName">Marca Agent</Label>
            <Input
              id="agentName"
              type="text"
              placeholder="Ex: Ion Popescu"
              value={agentSettings.agent_name || ""}
              onChange={(e) =>
                setAgentSettings((prev) => ({
                  ...prev,
                  agent_name: e.target.value,
                }))
              }
            />
            <p className="text-sm text-muted-foreground">
              Numele agentului care va apărea pe documente
            </p>
          </div>

            <div className="space-y-2">
              <Label htmlFor="carnetSeries">Serie Carnet (SimbolCarnet)</Label>
              <Input
                id="carnetSeries"
                type="text"
                placeholder="Ex: RS, FAC"
                value={agentSettings.carnet_series || ""}
                onChange={(e) =>
                  setAgentSettings((prev) => ({
                    ...prev,
                    carnet_series: e.target.value,
                  }))
                }
              />
              <p className="text-sm text-muted-foreground">
                Seria carnetului pentru facturi (ex: RS, FAC, etc.)
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="simbolCarnetLivr">Serie Carnet Livrări (SimbolCarnetLivr)</Label>
              <Input
                id="simbolCarnetLivr"
                type="text"
                placeholder="Ex: BL, LIV"
                value={agentSettings.simbol_carnet_livr || ""}
                onChange={(e) =>
                  setAgentSettings((prev) => ({
                    ...prev,
                    simbol_carnet_livr: e.target.value,
                  }))
                }
              />
              <p className="text-sm text-muted-foreground">
                Seria carnetului pentru livrări (ex: BL, LIV, etc.)
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="simbolGestiuneLivrare">Simbol Gestiune Livrare</Label>
              <Input
                id="simbolGestiuneLivrare"
                type="text"
                placeholder="Ex: MAGAZIN, DEPOZIT"
                value={agentSettings.simbol_gestiune_livrare || ""}
                onChange={(e) =>
                  setAgentSettings((prev) => ({
                    ...prev,
                    simbol_gestiune_livrare: e.target.value,
                  }))
                }
              />
              <p className="text-sm text-muted-foreground">
                Simbolul gestiunii de livrare din WME
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="codCarnet">Cod Carnet Facturi (CodCarnet)</Label>
              <Input
                id="codCarnet"
                type="text"
                placeholder="Ex: 1"
                value={agentSettings.cod_carnet || ""}
                onChange={(e) =>
                  setAgentSettings((prev) => ({
                    ...prev,
                    cod_carnet: e.target.value || null,
                  }))
                }
              />
              <p className="text-sm text-muted-foreground">
                Codul numeric al carnetului de facturi din WME pentru numerotare automată
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="codCarnetLivr">Cod Carnet Livrări (CodCarnetLivr)</Label>
              <Input
                id="codCarnetLivr"
                type="text"
                placeholder="Ex: 2"
                value={agentSettings.cod_carnet_livr || ""}
                onChange={(e) =>
                  setAgentSettings((prev) => ({
                    ...prev,
                    cod_carnet_livr: e.target.value || null,
                  }))
                }
              />
              <p className="text-sm text-muted-foreground">
                Codul numeric al carnetului de livrări din WME pentru numerotare automată
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="delegateName">Numele Delegatului</Label>
              <Input
                id="delegateName"
                type="text"
                placeholder="Ex: Ion Popescu"
                value={agentSettings.delegate_name || ""}
                onChange={(e) =>
                  setAgentSettings((prev) => ({
                    ...prev,
                    delegate_name: e.target.value || null,
                  }))
                }
              />
              <p className="text-sm text-muted-foreground">
                Numele delegatului care apare pe factură
              </p>
            </div>

            <div className="space-y-2">
              <Label htmlFor="delegateAct">Act Delegat</Label>
              <Input
                id="delegateAct"
                type="text"
                placeholder="Ex: CI nr. AA123456"
                value={agentSettings.delegate_act || ""}
                onChange={(e) =>
                  setAgentSettings((prev) => ({
                    ...prev,
                    delegate_act: e.target.value || null,
                  }))
                }
              />
              <p className="text-sm text-muted-foreground">
                Actul de identitate al delegatului (ex: CI, BI, Pașaport)
              </p>
            </div>

            <div className="space-y-4 pt-4 border-t">
              <h3 className="font-semibold">Numerotare Facturi</h3>

              <div className="grid grid-cols-3 gap-4">
                <div className="space-y-2">
                  <Label htmlFor="invoiceStart">Număr Start</Label>
                  <Input
                    id="invoiceStart"
                    type="number"
                    min="1"
                    placeholder="1"
                    value={agentSettings.invoice_number_start || ''}
                    onChange={(e) =>
                      setAgentSettings((prev) => ({
                        ...prev,
                        invoice_number_start: parseInt(e.target.value) || null,
                      }))
                    }
                  />
                </div>

                <div className="space-y-2">
                  <Label htmlFor="invoiceEnd">Număr Final</Label>
                  <Input
                    id="invoiceEnd"
                    type="number"
                    min="1"
                    placeholder="99999"
                    value={agentSettings.invoice_number_end || ''}
                    onChange={(e) =>
                      setAgentSettings((prev) => ({
                        ...prev,
                        invoice_number_end: parseInt(e.target.value) || null,
                      }))
                    }
                  />
                </div>

                <div className="space-y-2">
                  <Label htmlFor="invoiceCurrent">Număr Curent</Label>
                  <Input
                    id="invoiceCurrent"
                    type="number"
                    value={agentSettings.invoice_number_current || 1}
                    disabled
                    className="bg-muted"
                  />
                </div>
              </div>

              <p className="text-sm text-muted-foreground">
                Setează intervalul de numerotare pentru facturi. Numărul curent se actualizează automat la fiecare factură creată.
              </p>
            </div>

            <Button
              onClick={handleSaveAgentSettings}
              disabled={savingAgent}
              className="w-full"
            >
              {savingAgent ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  Se salvează...
                </>
              ) : (
                "Salvează date agent"
              )}
            </Button>
          </CardContent>
        </Card>

        {/* Sync Settings */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <RefreshCw className="h-5 w-5" />
              Sincronizare Date
            </CardTitle>
            <CardDescription>
              Sincronizează datele cu serverul WME pentru a obține ultimele informații despre parteneri și produse
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            <div className="grid gap-4 md:grid-cols-2">
              <div className="space-y-1 p-4 rounded-lg border bg-muted/30">
                <p className="text-sm font-medium">Ultima sincronizare parteneri</p>
                <p className="text-lg font-semibold text-primary">
                  {formatLastSync(status?.partners_synced_at ?? null)}
                </p>
              </div>
              <div className="space-y-1 p-4 rounded-lg border bg-muted/30">
                <p className="text-sm font-medium">Ultima sincronizare produse</p>
                <p className="text-lg font-semibold text-primary">
                  {formatLastSync(status?.products_synced_at ?? null)}
                </p>
              </div>
            </div>

            <div className="space-y-3">
              <p className="text-sm text-muted-foreground">
                Apasă butonul de mai jos pentru a sincroniza datele acum. Sincronizarea este necesară înainte de a crea prima factură.
              </p>
              <Button
                onClick={handleSyncNow}
                disabled={isSyncing || !isOnline}
                size="lg"
                className="w-full h-14 text-base gap-3"
              >
                {isSyncing ? (
                  <>
                    <Loader2 className="h-5 w-5 animate-spin" />
                    Se sincronizează datele...
                  </>
                ) : !isOnline ? (
                  <>
                    <RefreshCw className="h-5 w-5" />
                    Offline - Nu se poate sincroniza
                  </>
                ) : (
                  <>
                    <RefreshCw className="h-5 w-5" />
                    Sincronizează Acum
                  </>
                )}
              </Button>
            </div>
          </CardContent>
        </Card>

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
            {loadingPrinters ? (
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
              <li>• Versiunea aplicației: 0.6.9</li>
              <li>• Fișierele facturilor sunt salvate în: %APPDATA%\facturi.softconsulting.com\invoices\</li>
              <li>• Suport pentru printare PDF pe imprimantă termală 80mm</li>
              <li>• Printarea se face prin SumatraPDF (instalat automat)</li>
              <li>• Template optimizat pentru bonuri fiscale format 80mm x 297mm</li>
            </ul>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
