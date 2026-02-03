"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { debugDbCounts } from "@/lib/tauri/commands";
import { invoke } from "@tauri-apps/api/core";

export default function DebugPage() {
  const [result, setResult] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const [partnerId, setPartnerId] = useState("");
  const [partnerResult, setPartnerResult] = useState<string>("");
  const [partnerLoading, setPartnerLoading] = useState(false);
  const [updateLoading, setUpdateLoading] = useState(false);
  const [updateResult, setUpdateResult] = useState<string>("");

  const handleDebug = async () => {
    setLoading(true);
    try {
      const data = await debugDbCounts();
      setResult(data);
    } catch (error) {
      setResult(`Error: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  const handleCheckPartner = async () => {
    if (!partnerId.trim()) {
      setPartnerResult("Please enter a partner ID");
      return;
    }

    setPartnerLoading(true);
    try {
      const data = await invoke<string>("debug_partner_payment_terms", { partnerId: partnerId.trim() });
      setPartnerResult(data);
    } catch (error) {
      setPartnerResult(`Error: ${error}`);
    } finally {
      setPartnerLoading(false);
    }
  };

  const handleUpdateAllPartners = async () => {
    if (!confirm("Sigur vrei să actualizezi TOȚI partenerii la 30 zile scadență?")) {
      return;
    }

    setUpdateLoading(true);
    try {
      const data = await invoke<string>("update_all_partners_payment_terms", { newDays: "30" });
      setUpdateResult(data);
    } catch (error) {
      setUpdateResult(`Error: ${error}`);
    } finally {
      setUpdateLoading(false);
    }
  };

  return (
    <div className="container mx-auto p-6 space-y-6">
      <Card>
        <CardHeader>
          <CardTitle>Database Debug Info</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <Button onClick={handleDebug} disabled={loading}>
            {loading ? "Loading..." : "Check Database"}
          </Button>
          {result && (
            <pre className="bg-gray-100 dark:bg-gray-800 p-4 rounded-md overflow-auto text-sm whitespace-pre-wrap">
              {result}
            </pre>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Partner Payment Terms Checker</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex gap-2">
            <Input
              placeholder="Enter partner ID"
              value={partnerId}
              onChange={(e) => setPartnerId(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleCheckPartner()}
            />
            <Button onClick={handleCheckPartner} disabled={partnerLoading}>
              {partnerLoading ? "Loading..." : "Check"}
            </Button>
          </div>
          {partnerResult && (
            <pre className="bg-gray-100 dark:bg-gray-800 p-4 rounded-md overflow-auto text-sm whitespace-pre-wrap">
              {partnerResult}
            </pre>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Update Payment Terms</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <p className="text-sm text-muted-foreground">
              Actualizează toți partenerii din baza de date la scadență de 30 zile.
            </p>
            <Button 
              onClick={handleUpdateAllPartners} 
              disabled={updateLoading}
              variant="destructive"
            >
              {updateLoading ? "Se actualizează..." : "Setează toți partenerii la 30 zile"}
            </Button>
          </div>
          {updateResult && (
            <pre className="bg-gray-100 dark:bg-gray-800 p-4 rounded-md overflow-auto text-sm whitespace-pre-wrap">
              {updateResult}
            </pre>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
