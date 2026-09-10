"use client";

import { useState, useEffect, useRef } from "react";
import { useRouter } from "next/navigation";
import { Settings, Loader2, Printer, FileText, User, RefreshCw, Server, AlertTriangle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { getAvailablePrinters, getAgentSettings, saveAgentSettings, deletePartnersAndLocations } from "@/lib/tauri/commands";
import type { AgentSettings } from "@/lib/tauri/types";
import { toast } from "sonner";
import { useSyncStatus } from "@/hooks/useSyncStatus";
import { SettingField } from "@/app/components/settings/SettingField";
import { useOnlineStatus } from "@/hooks/useOnlineStatus";
import { useAuth } from "@/app/contexts/AuthContext";
import { getVersion } from "@tauri-apps/api/app";

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
  const [appVersion, setAppVersion] = useState<string | null>(null);

  useEffect(() => {
    getVersion().then(setAppVersion).catch(() => setAppVersion(null));
  }, []);
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
    tip_contabil: null,
    cert_comanda_serie: null,
    cert_comanda_id_client: null,
    cod_carnet: null,
    cod_carnet_livr: null,
    cod_delegat: null,
    delegate_name: null,
    delegate_act: null,
    car_number: null,
    invoice_number_start: null,
    invoice_number_end: null,
    invoice_number_current: null,
    marca_agent: null,
    nume_casa: null,
    auto_sync_collections_enabled: null,
    auto_sync_collections_time: null,
    receipt_series: null,
    receipt_number_start: null,
    receipt_number_end: null,
    receipt_number_current: null,
    wme_host: null,
    wme_port: null,
  });
  const [savingAgent, setSavingAgent] = useState(false);
  const [loadingAgentSettings, setLoadingAgentSettings] = useState(true);
  const savedMarcaAgentRef = useRef<string>("");

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
      savedMarcaAgentRef.current = (settings.marca_agent || "").trim();
    } catch (error) {
      console.error("Failed to load agent settings:", error);
    } finally {
      setLoadingAgentSettings(false);
    }
  };

  const handleSaveAgentSettings = async () => {
    setSavingAgent(true);
    try {
      const oldMarcaAgent = savedMarcaAgentRef.current;
      const newMarcaAgent = (agentSettings.marca_agent || "").trim();
      const normalizedMarcaAgent = newMarcaAgent.length > 0 ? newMarcaAgent : null;

      await saveAgentSettings({
        ...agentSettings,
        marca_agent: normalizedMarcaAgent,
        wme_host: agentSettings.wme_host?.trim() || null,
      });

      const marcaChanged = oldMarcaAgent !== newMarcaAgent;

      if (marcaChanged) {
        if (newMarcaAgent) {
          toast.info("Marca Agent a fost schimbată. Se reîncarcă partenerii pentru noul agent...");
        } else {
          toast.info("Marca Agent a fost ștearsă. Se reîncarcă partenerii fără filtrare pe marcă...");
        }

        await deletePartnersAndLocations();

        if (isOnline) {
          await triggerSync();
          toast.success("Partenerii au fost resincronizați după modificarea mărcii.");
        } else {
          toast.warning("Marca Agent a fost salvată, dar ești offline. Fă sincronizarea când revii online.");
        }
      }

      // Reload settings to update UI with latest values from database
      await loadAgentSettings();
      toast.success("Setările agentului au fost salvate!");
    } catch (error) {
      // This block covers four different operations (save, delete partners, resync,
      // reload), so show what actually failed instead of always blaming the save.
      console.error("Failed to save agent settings:", error);
      const detail =
        error instanceof Error ? error.message : String(error ?? "eroare necunoscută");
      toast.error(`Eroare la salvarea setărilor agentului: ${detail}`);
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
          // The list stays on screen while loadPrinters() refreshes it in the background;
          // see hasNoPrintersYet below. Blocking on the OS query instead kept the whole
          // Settings screen behind a spinner even though the list was already in hand.
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

  // Enumerating printers goes out to the OS and is refreshed on every visit, so only block
  // the screen when there is genuinely nothing to show. A list cached from a previous visit
  // is enough to render Settings immediately; the refresh then lands silently underneath.
  const hasNoPrintersYet = loadingPrinters && printers.length === 0;

  // Show full page loading while all settings are loading initially
  if (hasNoPrintersYet || loadingAgentSettings) {
    return (
      <div className="h-full flex flex-col items-center justify-center gap-4">
        <Loader2 className="h-12 w-12 animate-spin text-primary" />
        <div className="text-center">
          <h2 className="text-lg font-semibold">Se încarcă setările...</h2>
          <p className="text-muted-foreground text-sm">
            {hasNoPrintersYet
              ? "Se verifică imprimantele disponibile"
              : "Se încarcă configurările agentului"
            }
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col space-y-4">
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
      <div className="flex-1 overflow-y-auto min-h-0 space-y-4">
        {/* WME Server Config */}
        <Card className={!agentSettings.wme_host?.trim() ? "border-amber-400 dark:border-amber-600" : ""}>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-base">
              <Server className="h-4 w-4" />
              Server WME
              {!agentSettings.wme_host?.trim() && (
                <span className="flex items-center gap-1 ml-auto text-xs font-medium text-amber-600 dark:text-amber-400 bg-amber-100 dark:bg-amber-900/30 px-2 py-1 rounded-full">
                  <AlertTriangle className="h-3.5 w-3.5" />
                  Neconfigurat
                </span>
              )}
            </CardTitle>
          </CardHeader>
          <CardContent className="grid gap-x-4 gap-y-3 sm:grid-cols-3">
            <SettingField
              id="wmeHost"
              label="Adresă IP"
              placeholder="Ex: 192.168.1.100"
              value={agentSettings.wme_host}
              onChange={(v) => setAgentSettings((prev) => ({ ...prev, wme_host: v }))}
              hint="Adresa serverului pe care rulează WME. Fără ea, aplicația nu poate sincroniza nimic."
              className="sm:col-span-2"
            />
            <SettingField
              id="wmePort"
              label="Port"
              type="number"
              placeholder="8089"
              value={agentSettings.wme_port}
              onChange={(v) =>
                setAgentSettings((prev) => ({ ...prev, wme_port: v ? parseInt(v, 10) : null }))
              }
              hint="Implicit 8089."
            />
          </CardContent>
        </Card>

        {/* Agent identity */}
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-base">
              <User className="h-4 w-4" />
              Identitate agent
            </CardTitle>
          </CardHeader>
          <CardContent className="grid gap-x-4 gap-y-3 sm:grid-cols-2 lg:grid-cols-3">
            <SettingField
              id="agentName"
              label="Nume agent"
              placeholder="Ex: Ion Popescu"
              value={agentSettings.agent_name}
              onChange={(v) => setAgentSettings((prev) => ({ ...prev, agent_name: v }))}
              hint="Numele care apare pe documentele tipărite."
            />
            <SettingField
              id="marcaAgent"
              label="Marca agent"
              placeholder="Ex: 123"
              value={agentSettings.marca_agent}
              onChange={(v) => setAgentSettings((prev) => ({ ...prev, marca_agent: v }))}
              hint="Codul numeric din WME. Filtrează partenerii și soldurile la cei ai acestui agent. Schimbarea lui reîncarcă lista de parteneri."
            />
            <SettingField
              id="numeCasa"
              label="Nume casă"
              placeholder="Ex: CASA LEI"
              value={agentSettings.nume_casa}
              onChange={(v) => setAgentSettings((prev) => ({ ...prev, nume_casa: v }))}
              hint="Casa din WME în care intră încasările."
            />
          </CardContent>
        </Card>

        {/* Series and booklets */}
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-base">
              <FileText className="h-4 w-4" />
              Serii și carnete
            </CardTitle>
          </CardHeader>
          <CardContent className="grid gap-x-4 gap-y-3 sm:grid-cols-2 lg:grid-cols-3">
            <SettingField
              id="carnetSeries"
              label="Serie carnet"
              placeholder="Ex: FONG"
              value={agentSettings.carnet_series}
              onChange={(v) => setAgentSettings((prev) => ({ ...prev, carnet_series: v }))}
              hint="SimbolCarnet în WME. Seria facturilor emise de acest agent."
            />
            <SettingField
              id="simbolCarnetLivr"
              label="Serie carnet livrări"
              placeholder="Ex: FONGL"
              value={agentSettings.simbol_carnet_livr}
              onChange={(v) => setAgentSettings((prev) => ({ ...prev, simbol_carnet_livr: v }))}
              hint="SimbolCarnetLivr în WME."
            />
            <SettingField
              id="simbolGestiuneLivrare"
              label="Gestiune livrare"
              placeholder="Ex: MARFA"
              value={agentSettings.simbol_gestiune_livrare}
              onChange={(v) => setAgentSettings((prev) => ({ ...prev, simbol_gestiune_livrare: v }))}
              hint="Gestiunea din care se descarcă marfa."
            />
            <SettingField
              id="codCarnet"
              label="Cod carnet facturi"
              placeholder="Ex: 1"
              value={agentSettings.cod_carnet}
              onChange={(v) => setAgentSettings((prev) => ({ ...prev, cod_carnet: v }))}
              hint="CodCarnet în WME."
            />
            <SettingField
              id="codCarnetLivr"
              label="Cod carnet livrări"
              placeholder="Ex: 2"
              value={agentSettings.cod_carnet_livr}
              onChange={(v) => setAgentSettings((prev) => ({ ...prev, cod_carnet_livr: v }))}
              hint="CodCarnetLivr în WME."
            />
            <SettingField
              id="tipContabil"
              label="Tip contabil"
              placeholder="valoare"
              value={agentSettings.tip_contabil}
              onChange={(v) => setAgentSettings((prev) => ({ ...prev, tip_contabil: v }))}
              hint="Tipul contabil al liniilor de factură. Implicit „valoare”."
            />
          </CardContent>
        </Card>

        {/* Delegate */}
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-base">
              <User className="h-4 w-4" />
              Delegat
            </CardTitle>
          </CardHeader>
          <CardContent className="grid gap-x-4 gap-y-3 sm:grid-cols-2 lg:grid-cols-4">
            <SettingField
              id="codDelegat"
              label="Cod delegat"
              placeholder="Ex: 5"
              value={agentSettings.cod_delegat}
              onChange={(v) => setAgentSettings((prev) => ({ ...prev, cod_delegat: v }))}
              hint="CodDelegat în WME."
            />
            <SettingField
              id="delegateName"
              label="Nume delegat"
              placeholder="Ex: Ion Popescu"
              value={agentSettings.delegate_name}
              onChange={(v) => setAgentSettings((prev) => ({ ...prev, delegate_name: v }))}
            />
            <SettingField
              id="delegateAct"
              label="Act delegat"
              placeholder="Ex: CI SB 123456"
              value={agentSettings.delegate_act}
              onChange={(v) => setAgentSettings((prev) => ({ ...prev, delegate_act: v }))}
              hint="Seria și numărul actului de identitate, tipărite pe factură."
            />
            <SettingField
              id="carNumber"
              label="Număr auto"
              placeholder="Ex: SB 01 ABC"
              value={agentSettings.car_number}
              onChange={(v) => setAgentSettings((prev) => ({ ...prev, car_number: v }))}
              hint="Numărul mașinii de transport, tipărit pe factură."
            />
          </CardContent>
        </Card>

        {/* Quality certificate */}
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-base">
              <FileText className="h-4 w-4" />
              Certificat de calitate
            </CardTitle>
          </CardHeader>
          <CardContent className="grid gap-x-4 gap-y-3 sm:grid-cols-2">
            <SettingField
              id="certComandaSerie"
              label="Serie comandă"
              placeholder="CCAL"
              value={agentSettings.cert_comanda_serie}
              onChange={(v) => setAgentSettings((prev) => ({ ...prev, cert_comanda_serie: v }))}
              hint="Seria comenzii din care se citesc lotul și datele de expirare. Implicit CCAL."
            />
            <SettingField
              id="certComandaIdClient"
              label="ID client"
              placeholder="1602"
              value={agentSettings.cert_comanda_id_client}
              onChange={(v) => setAgentSettings((prev) => ({ ...prev, cert_comanda_id_client: v }))}
              hint="Clientul pe care sunt înregistrate comenzile de producție. Implicit 1602."
            />
          </CardContent>
        </Card>

        {/* Numbering */}
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-base">
              <FileText className="h-4 w-4" />
              Numerotare
            </CardTitle>
            <CardDescription>
              Numărul curent avansează singur la fiecare document emis.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid gap-x-4 gap-y-3 sm:grid-cols-2 lg:grid-cols-4">
              <SettingField
                id="invoiceSeriesRead"
                label="Serie facturi"
                value={agentSettings.carnet_series}
                onChange={() => {}}
                disabled
                hint="Preluată din „Serie carnet”, în secțiunea Serii și carnete."
              />
              <SettingField
                id="invoiceStart"
                label="Număr start"
                type="number"
                min="1"
                placeholder="1"
                value={agentSettings.invoice_number_start}
                onChange={(v) =>
                  setAgentSettings((prev) => ({ ...prev, invoice_number_start: parseInt(v) || null }))
                }
              />
              <SettingField
                id="invoiceEnd"
                label="Număr final"
                type="number"
                min="1"
                placeholder="99999"
                value={agentSettings.invoice_number_end}
                onChange={(v) =>
                  setAgentSettings((prev) => ({ ...prev, invoice_number_end: parseInt(v) || null }))
                }
              />
              <SettingField
                id="invoiceCurrent"
                label="Număr curent"
                type="number"
                disabled
                value={agentSettings.invoice_number_current ?? 1}
                onChange={() => {}}
              />
            </div>

            <div className="grid gap-x-4 gap-y-3 sm:grid-cols-2 lg:grid-cols-4 pt-4 border-t">
              <SettingField
                id="receiptSeries"
                label="Serie chitanțe"
                placeholder="Ex: CH"
                value={agentSettings.receipt_series}
                onChange={(v) => setAgentSettings((prev) => ({ ...prev, receipt_series: v }))}
              />
              <SettingField
                id="receiptStart"
                label="Număr start"
                type="number"
                min="1"
                placeholder="1"
                value={agentSettings.receipt_number_start}
                onChange={(v) =>
                  setAgentSettings((prev) => ({ ...prev, receipt_number_start: parseInt(v) || null }))
                }
              />
              <SettingField
                id="receiptEnd"
                label="Număr final"
                type="number"
                min="1"
                placeholder="99999"
                value={agentSettings.receipt_number_end}
                onChange={(v) =>
                  setAgentSettings((prev) => ({ ...prev, receipt_number_end: parseInt(v) || null }))
                }
              />
              <SettingField
                id="receiptCurrent"
                label="Număr curent"
                type="number"
                disabled
                value={agentSettings.receipt_number_current ?? 1}
                onChange={() => {}}
              />
            </div>
          </CardContent>
        </Card>

        {/* Automatic collection sync */}
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-base">
              <RefreshCw className="h-4 w-4" />
              Sincronizare automată încasări
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between gap-4">
              <p className="text-sm text-muted-foreground">
                Trimite încasările zilei la ora stabilită, fără intervenție.
              </p>
              <Switch
                checked={agentSettings.auto_sync_collections_enabled || false}
                onCheckedChange={(checked) =>
                  setAgentSettings((prev) => ({ ...prev, auto_sync_collections_enabled: checked }))
                }
              />
            </div>
            {agentSettings.auto_sync_collections_enabled && (
              <div className="grid gap-x-4 gap-y-3 sm:grid-cols-3">
                <div className="space-y-1.5">
                  <Label htmlFor="autoSyncTime" className="text-sm">Oră sincronizare</Label>
                  <Input
                    id="autoSyncTime"
                    type="time"
                    value={agentSettings.auto_sync_collections_time || "23:00"}
                    onChange={(e) =>
                      setAgentSettings((prev) => ({
                        ...prev,
                        auto_sync_collections_time: e.target.value,
                      }))
                    }
                  />
                </div>
              </div>
            )}
          </CardContent>
        </Card>

        <Button onClick={handleSaveAgentSettings} disabled={savingAgent} className="w-full h-11">
          {savingAgent ? (
            <>
              <Loader2 className="h-4 w-4 mr-2 animate-spin" />
              Se salvează...
            </>
          ) : (
            "Salvează setările agentului"
          )}
        </Button>


        {/* Sync Settings */}
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-base">
              <RefreshCw className="h-4 w-4" />
              Sincronizare Date
            </CardTitle>
            <CardDescription>
              Sincronizează datele cu serverul WME pentru a obține ultimele informații despre parteneri și produse
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
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
                    <RefreshCw className="h-4 w-4" />
                    Offline - Nu se poate sincroniza
                  </>
                ) : (
                  <>
                    <RefreshCw className="h-4 w-4" />
                    Sincronizează Acum
                  </>
                )}
              </Button>
            </div>
          </CardContent>
        </Card>

        {/* Printer Settings */}
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-base">
              <Printer className="h-4 w-4" />
              Setări Printare
            </CardTitle>
            <CardDescription>
              Configurează imprimanta și opțiunile de printare
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {hasNoPrintersYet ? (
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
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-base">
              <FileText className="h-4 w-4" />
              Informații
            </CardTitle>
          </CardHeader>
          <CardContent>
            <ul className="space-y-2 text-sm text-muted-foreground">
              <li>• Versiunea aplicației: {appVersion ?? "…"}</li>
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
