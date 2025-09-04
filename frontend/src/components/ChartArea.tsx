import React, { useState, useEffect } from "react";
import useFlashOnChange from "../hooks/useFlashOnChange";
import { useStock } from "../contexts/StockContext";

export default function ChartArea() {
  const { selectedSymbol } = useStock();
  const [lastPrice, setLastPrice] = useState(238.50);
  const [change, setChange] = useState(2.45);
  const [changePercent, setChangePercent] = useState(1.04);
  
  const flash = useFlashOnChange(lastPrice);

  useEffect(() => {
    // Simulate price updates
    const interval = setInterval(() => {
      const newPrice = lastPrice + (Math.random() - 0.5) * 0.5;
      const newChange = newPrice - 238.50;
      const newChangePercent = (newChange / 238.50) * 100;
      
      setLastPrice(newPrice);
      setChange(newChange);
      setChangePercent(newChangePercent);
    }, 3000);

    return () => clearInterval(interval);
  }, [lastPrice]);

  const isPositive = change >= 0;
  const deltaStr = `${isPositive ? '+' : ''}${change.toFixed(2)}`;
  const pctStr = `${isPositive ? '+' : ''}${changePercent.toFixed(2)}%`;

  return (
    <div className="flex-1 bg-hl-panel/40 border-b border-slate-800 p-4">
      <div className="h-full flex items-center justify-center">
        <div className="text-center">
          <div className="text-2xl font-bold text-hl-text mb-2">{selectedSymbol}</div>
          <h1 className={`text-5xl font-semibold text-cyan-300 drop-shadow-text-cyan mb-2 ${flash ? 'flash-glow' : ''}`}>
            ${lastPrice.toFixed(2)}
          </h1>
          <p className={`text-sm ${isPositive ? "text-cyan-300/80 text-neon-cyan" : "text-rose-400/80 text-neon-rose"}`}>
            {deltaStr} ({pctStr})
          </p>
          <div className="text-xs text-hl-muted mt-1">
            Volume: 45.2M
          </div>
          
          {/* Placeholder for chart */}
          <div className="mt-8 w-full h-64 bg-slate-900/50 rounded-lg border border-slate-800 flex items-center justify-center neon-card">
            <div className="text-hl-muted">
              📈 {selectedSymbol} Chart (TradingView/Recharts integration)
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
