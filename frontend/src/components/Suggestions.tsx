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
    <div className="rounded-xl2 bg-hl-panel/60 border border-slate-800 p-4">
      <h3 className="text-sm font-semibold text-hl-text mb-3">AI Copilot</h3>
      
      {items.length === 0 && (
        <div className="text-hl-muted text-center py-6 text-xs">
          Waiting for suggestions...
        </div>
      )}
      
      <div className="space-y-2">
        {items.slice(0, 3).map((s, i) => {
          const payload = s.payload || s;
          const features = payload.features || s.features;
          
          return (
            <div key={i} className="border border-slate-800 rounded-lg p-3 hover:bg-slate-800/20 transition-colors">
              <div className="flex justify-between items-start mb-2">
                <div className="font-semibold text-hl-text text-sm">{payload.symbol}</div>
                <div className="text-xs text-hl-muted">
                  {Math.round(payload.confidence * 100)}%
                </div>
              </div>
              
              <p className="text-xs text-hl-muted mb-2">{payload.suggestion}</p>
              
              {features && (
                <div className="text-xs text-hl-muted mb-2">
                  {Object.entries(features).map(([k, v]) => `${k}:${v}`).join(" • ")}
                </div>
              )}
              
              <div className="flex gap-1">
                <button 
                  className="px-2 py-1 text-xs bg-hl-buy/20 text-hl-buy border border-hl-buy/40 rounded hover:bg-hl-buy/30 transition-colors"
                  onClick={() => accept(s)}
                >
                  Accept
                </button>
                <button 
                  className="px-2 py-1 text-xs bg-slate-800/50 text-hl-muted border border-slate-700 rounded hover:bg-slate-700/50 transition-colors"
                  onClick={() => dismiss(s)}
                >
                  Dismiss
                </button>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
