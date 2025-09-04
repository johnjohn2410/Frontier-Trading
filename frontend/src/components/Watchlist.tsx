import React, { useState, useEffect } from "react";

type WatchlistItem = {
  symbol: string;
  price: number;
  change: number;
  changePercent: number;
};

const SYMBOLS = ['AAPL', 'MSFT', 'TSLA', 'SPY', 'GOOGL', 'AMZN', 'NVDA', 'META'];

export default function Watchlist() {
  const [items, setItems] = useState<WatchlistItem[]>([]);
  const [selected, setSelected] = useState('AAPL');

  useEffect(() => {
    // Mock data - in production this would fetch real prices
    const mockItems: WatchlistItem[] = SYMBOLS.map(symbol => ({
      symbol,
      price: 200 + Math.random() * 100,
      change: (Math.random() - 0.5) * 10,
      changePercent: (Math.random() - 0.5) * 5,
    }));
    
    setItems(mockItems);
    
    const interval = setInterval(() => {
      setItems(prev => prev.map(item => ({
        ...item,
        price: item.price + (Math.random() - 0.5) * 0.5,
        change: (Math.random() - 0.5) * 10,
        changePercent: (Math.random() - 0.5) * 5,
      })));
    }, 2000);
    
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="flex-1 overflow-y-auto">
      <div className="p-3 border-b border-slate-800">
        <h3 className="text-sm font-semibold text-hl-text">Watchlist</h3>
      </div>
      
      <div className="space-y-1 p-2">
        {items.map((item) => {
          const isPositive = item.change >= 0;
          const isSelected = selected === item.symbol;
          
          return (
            <div
              key={item.symbol}
              onClick={() => setSelected(item.symbol)}
              className={`relative p-2 rounded-lg cursor-pointer transition-all ${
                isSelected 
                  ? 'bg-hl-accent/20 border border-hl-accent/40' 
                  : 'hover:bg-slate-800/30'
              }`}
            >
              {isSelected && (
                <span className="absolute left-0 top-1/2 -translate-y-1/2 h-8 w-[3px] rounded bg-cyan-400 shadow-neon-cyan"></span>
              )}
              <div className="flex justify-between items-center">
                <div>
                  <div className="text-sm font-semibold text-hl-text">
                    {item.symbol}
                  </div>
                  <div className="text-xs text-hl-muted">
                    ${item.price.toFixed(2)}
                  </div>
                </div>
                <div className="text-right">
                  <div className={`text-xs font-mono ${
                    isPositive ? 'text-cyan-300 text-neon-cyan' : 'text-rose-400 text-neon-rose'
                  }`}>
                    {isPositive ? '+' : ''}{item.change.toFixed(2)}
                  </div>
                  <div className={`text-xs font-mono ${
                    isPositive ? 'text-cyan-300 text-neon-cyan' : 'text-rose-400 text-neon-rose'
                  }`}>
                    {isPositive ? '+' : ''}{item.changePercent.toFixed(1)}%
                  </div>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
