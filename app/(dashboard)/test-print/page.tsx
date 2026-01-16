"use client";

import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";

export default function TestPrintPage() {
  const [invoices, setInvoices] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<string>("");

  const loadInvoices = async () => {
    setLoading(true);
    try {
      const result = await invoke<any[]>("get_invoices");
      setInvoices(result || []);
      setResult(`Loaded ${result?.length || 0} invoices`);
    } catch (e) {
      setResult(`Error loading invoices: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const testPrint = async (invoiceId: string) => {
    setLoading(true);
    try {
      console.log("Testing print for invoice:", invoiceId);
      const filePath = await invoke<string>("print_invoice_to_html", {
        invoiceId,
      });
      setResult(`SUCCESS! File generated at: ${filePath}`);
      console.log("File path:", filePath);
    } catch (e) {
      setResult(`Error: ${e}`);
      console.error("Print error:", e);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="p-8 max-w-2xl mx-auto">
      <h1 className="text-3xl font-bold mb-6">Test Print Functionality</h1>

      <Card className="p-6 mb-6">
        <Button onClick={loadInvoices} disabled={loading} className="mr-4">
          Load Invoices
        </Button>
        {result && <p className="text-sm text-gray-600 mt-4">{result}</p>}
      </Card>

      {invoices.length > 0 && (
        <div className="space-y-3">
          <h2 className="text-xl font-semibold">Invoices ({invoices.length})</h2>
          {invoices.map((invoice) => (
            <Card
              key={invoice.id}
              className="p-4 flex justify-between items-center"
            >
              <div>
                <p className="font-medium">{invoice.partner_name}</p>
                <p className="text-sm text-gray-500">{invoice.id}</p>
                <p className="text-sm">Total: {invoice.total_amount} RON</p>
              </div>
              <Button
                onClick={() => testPrint(invoice.id)}
                disabled={loading}
                variant="outline"
              >
                Test Print
              </Button>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
