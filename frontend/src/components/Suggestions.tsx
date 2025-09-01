import React from "react";
import { useStream } from "../hooks/useStream";

type Sugg = {
  symbol: string;
  suggestion: string;
  confidence: number;
  what_if?: { qty: string; limit?: string };
  features?: Record<string, string | number>;
  correlation_id?: string;
  payload?: {
    symbol: string;
    suggestion: string;
    confidence: number;
    what_if: {
      qty: string;
      limit: string;
      delta_bp: string;
      est_fee: string;
    };
    features: {
      cmin: string;
      cmax: string;
      vol_z: string;
    };
    guardrails: {
      max_pos_ok: boolean;
      daily_loss_ok: boolean;
      mkt_hours_ok: boolean;
    };
  };
};

export default function Suggestions() {
  const items = useStream<Sugg>("ws://localhost:8000/ws/stream?s=suggestions.stream");
  
  const accept = async (s: Sugg) => {
    try {
      const payload = s.payload || s;
      const body = {
        symbol: payload.symbol,
        side: payload.suggestion.toLowerCase().includes('buy') ? "buy" : "sell",
        type: payload.what_if?.limit ? "limit" : "market",
        qty: payload.what_if?.qty ?? "1",
        limit_price: payload.what_if?.limit,
        correlation_id: s.correlation_id
      };
      
      console.log('Placing order:', body);
      
      const res = await fetch("http://localhost:8000/orders", { 
        method: "POST", 
        headers: { "Content-Type": "application/json" }, 
        body: JSON.stringify(body) 
      });
      
      const json = await res.json();
      alert(`Order result: ${JSON.stringify(json)}`);
    } catch (error) {
      console.error('Failed to place order:', error);
      alert('Failed to place order. Check console for details.');
    }
  };

  const dismiss = (s: Sugg) => {
    // TODO: Implement dismiss functionality
    console.log('Dismissing suggestion:', s);
  };

  return (
    <div className="space-y-3">
      <h2 className="text-xl font-bold mb-4">Copilot Suggestions</h2>
      
      {items.length === 0 && (
        <div className="text-gray-500 text-center py-8">
          Waiting for suggestions... Make sure the Copilot service is running.
        </div>
      )}
      
      {items.map((s, i) => {
        const payload = s.payload || s;
        const features = payload.features || s.features;
        
        return (
          <div key={i} className="rounded-xl border p-3 bg-white shadow-sm">
            <div className="flex justify-between items-start">
              <div className="font-semibold text-lg">{payload.symbol}</div>
              <div className="text-sm opacity-70">
                conf {Math.round(payload.confidence * 100)}%
              </div>
            </div>
            
            <p className="mt-2 text-gray-700">{payload.suggestion}</p>
            
            {features && (
              <div className="mt-2 text-xs text-gray-600">
                Why: {Object.entries(features).map(([k, v]) => `${k}:${v}`).join(" • ")}
              </div>
            )}
            
            {payload.what_if && (
              <div className="mt-2 text-xs text-gray-600">
                What if: {payload.what_if.qty} shares @ ${payload.what_if.limit} 
                (${payload.what_if.delta_bp} impact)
              </div>
            )}
            
            <div className="mt-3 flex gap-2">
              <button 
                className="px-3 py-2 rounded bg-black text-white hover:bg-gray-800 transition-colors"
                onClick={() => accept(s)}
              >
                Accept
              </button>
              <button 
                className="px-3 py-2 rounded border hover:bg-gray-50 transition-colors"
                onClick={() => dismiss(s)}
              >
                Dismiss
              </button>
            </div>
          </div>
        );
      })}
    </div>
  );
}
