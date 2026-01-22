"use client";

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { toast } from "sonner";

export default function ApiSettingsPage() {
  const [apiIp, setApiIp] = useState("10.200.1.94");
  const [apiPort, setApiPort] = useState("8089");
  const [loading, setLoading] = useState(false);
  const [testResult, setTestResult] = useState<string>("");

  useEffect(() => {
    // Load saved settings
    const savedIp = localStorage.getItem("apiIp");
    const savedPort = localStorage.getItem("apiPort");
    
    if (savedIp) setApiIp(savedIp);
    if (savedPort) setApiPort(savedPort);
  }, []);

  const handleSave = () => {
    setLoading(true);
    
    try {
      localStorage.setItem("apiIp", apiIp);
      localStorage.setItem("apiPort", apiPort);
      
      toast.success("Setări API salvate cu succes!");
    } catch (error) {
      toast.error("Eroare la salvarea setărilor");
      console.error(error);
    } finally {
      setLoading(false);
    }
  };

  const handleTestPartners = async () => {
    setLoading(true);
    setTestResult("");
    toast.info("Testare GET parteneri...");
    
    try {
      const result = await invoke<string>("test_api_partners");
      setTestResult(result);
      toast.success("Test parteneri reușit!");
    } catch (error: any) {
      setTestResult(error);
      toast.error("Test parteneri eșuat");
      console.error(error);
    } finally {
      setLoading(false);
    }
  };

  const handleTestArticles = async () => {
    setLoading(true);
    setTestResult("");
    toast.info("Testare GET articole...");
    
    try {
      const result = await invoke<string>("test_api_articles");
      setTestResult(result);
      toast.success("Test articole reușit!");
    } catch (error: any) {
      setTestResult(error);
      toast.error("Test articole eșuat");
      console.error(error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="p-6 max-w-2xl mx-auto">
      <h1 className="text-3xl font-bold mb-6">Configurare API</h1>

      <Card>
        <CardHeader>
          <CardTitle>Setări Server REST</CardTitle>
          <CardDescription>
            Configurează adresa IP și portul serverului WME REST
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="apiIp">Adresă IP Server</Label>
            <Input
              id="apiIp"
              type="text"
              value={apiIp}
              onChange={(e) => setApiIp(e.target.value)}
              placeholder="10.200.1.94"
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="apiPort">Port</Label>
            <Input
              id="apiPort"
              type="text"
              value={apiPort}
              onChange={(e) => setApiPort(e.target.value)}
              placeholder="8089"
            />
          </div>

          <div className="text-sm text-muted-foreground bg-muted p-3 rounded">
            <p className="font-semibold mb-1">URL complet:</p>
            <code>http://{apiIp}:{apiPort}/datasnap/rest/TServerMethods</code>
          </div>

          <div className="flex gap-2">
            <Button onClick={handleSave} disabled={loading}>
              Salvează
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card className="mt-6">
        <CardHeader>
          <CardTitle>Testare API</CardTitle>
          <CardDescription>
            Verifică conexiunea și funcționalitatea API-ului
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex gap-2">
            <Button onClick={handleTestPartners} variant="outline" disabled={loading}>
              Test GET Parteneri
            </Button>
            <Button onClick={handleTestArticles} variant="outline" disabled={loading}>
              Test GET Articole
            </Button>
          </div>

          {testResult && (
            <div className="mt-4 p-4 bg-muted rounded-lg">
              <pre className="text-sm whitespace-pre-wrap">{testResult}</pre>
            </div>
          )}
        </CardContent>
      </Card>

      <Card className="mt-6">
        <CardHeader>
          <CardTitle>Informații</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2 text-sm">
          <p>
            <strong>IP implicit:</strong> 10.200.1.94
          </p>
          <p>
            <strong>Port implicit:</strong> 8089
          </p>
          <p className="text-muted-foreground mt-4">
            Aplicația folosește date mock pentru testare. Butoanele de test verifică dacă
            API-ul real răspunde corect la cereri GET.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
