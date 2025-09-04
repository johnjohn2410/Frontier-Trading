import React, { useState, useEffect } from "react";

type Trade = {
  id: string;
  price: number;
  size: number;
  side: 'buy' | 'sell';
  timestamp: number;
};

export default function TradesTape() {
  const [trades, setTrades] = useState<Trade[]>([]);

  useEffect(() => {
    // Generate mock trades
    const generateTrade = (): Trade => ({
      id: Math.random().toString(36).substr(2, 9),
      price: 238.50 + (Math.random() - 0.5) * 0.5,
      size: Math.floor(Math.random() * 1000) + 10,
      side: Math.random() > 0.5 ? 'buy' : 'sell',
      timestamp: Date.now(),
    });

    // Initial trades
    const initialTrades = Array.from({ length: 20 }, generateTrade);
    setTrades(initialTrades);

    // Add new trades every 1-3 seconds
    const interval = setInterval(() => {
      setTrades(prev => [generateTrade(), ...prev.slice(0, 49)]);
    }, Math.random() * 2000 + 1000);

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="h-32 bg-hl-panel/40 border-t border-slate-800 overflow-hidden neon-card">
      <div className="px-4 py-2 border-b border-slate-800">
        <h3 className="text-xs font-semibold text-hl-muted">Recent Trades</h3>
      </div>
      
      <div className="h-full overflow-y-auto">
        <div className="space-y-1 p-2">
          {trades.map((trade, index) => {
            const isBuy = trade.side === 'buy';
            const time = new Date(trade.timestamp).toLocaleTimeString();
            
            return (
              <div
                key={trade.id}
                className="grid grid-cols-4 gap-2 text-xs py-1 hover:bg-slate-800/20 transition-colors"
              >
                <span className="text-hl-muted font-mono">{time}</span>
                <span className={`font-mono ${isBuy ? 'text-cyan-300 text-neon-cyan' : 'text-rose-400 text-neon-rose'}`}>
                  ${trade.price.toFixed(2)}
                </span>
                <span className="text-right font-mono text-hl-text">
                  {trade.size.toLocaleString()}
                </span>
                <span className={`text-right font-mono ${isBuy ? 'text-cyan-300 text-neon-cyan' : 'text-rose-400 text-neon-rose'}`}>
                  {isBuy ? 'BUY' : 'SELL'}
                </span>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
