import React, { useState, useEffect } from "react";
import { useStock } from "../contexts/StockContext";

type OrderBookRow = {
  price: number;
  size: number;
  side: 'bid' | 'ask';
};

// Mock order book data - in production this would come from WebSocket
const generateMockOrderBook = (): OrderBookRow[] => {
  const basePrice = 238.50;
  const rows: OrderBookRow[] = [];
  
  // Generate bids (below base price)
  for (let i = 0; i < 10; i++) {
    const price = basePrice - (i + 1) * 0.01;
    const size = Math.random() * 1000 + 100;
    rows.push({ price, size, side: 'bid' });
  }
  
  // Generate asks (above base price)
  for (let i = 0; i < 10; i++) {
    const price = basePrice + (i + 1) * 0.01;
    const size = Math.random() * 1000 + 100;
    rows.push({ price, size, side: 'ask' });
  }
  
  return rows.sort((a, b) => b.price - a.price); // Highest first
};

export default function OrderBook() {
  const { selectedSymbol } = useStock();
  const [orderBook, setOrderBook] = useState<OrderBookRow[]>([]);
  const [midPrice, setMidPrice] = useState(238.50);

  useEffect(() => {
    // Initialize with mock data
    setOrderBook(generateMockOrderBook());
    
    // Update every 2 seconds with new data
    const interval = setInterval(() => {
      setOrderBook(generateMockOrderBook());
    }, 2000);
    
    return () => clearInterval(interval);
  }, []);

  const maxSize = Math.max(1, ...orderBook.map(r => r.size));
  const bids = orderBook.filter(r => r.side === 'bid').slice(0, 8);
  const asks = orderBook.filter(r => r.side === 'ask').slice(0, 8);

  return (
    <div className="rounded-xl2 bg-hl-panel/60 border border-slate-800 overflow-hidden neon-card">
      <div className="grid grid-cols-3 text-xs px-3 py-2 text-hl-muted border-b border-slate-800">
        <span>Price</span>
        <span className="text-right">Size</span>
        <span className="text-right">Cum</span>
      </div>
      
      <div className="divide-y divide-slate-900/60">
        {/* Asks (highest prices first) */}
        {asks.reverse().map((row, i) => {
          const intensity = Math.min(1, row.size / maxSize);
          const heat = `rgba(244,63,94,${0.08 + 0.22 * intensity})`;
          const isMid = Math.abs(row.price - midPrice) < 0.01;
          
          return (
            <div 
              key={`ask-${i}`} 
              className="ob-row grid grid-cols-3 px-3 py-1.5 text-sm relative hover:bg-slate-800/30 transition-colors"
              style={{ ['--heat' as any]: heat }}
            >
              <span className="text-rose-400 font-mono">
                {row.price.toFixed(2)}
              </span>
              <span className="text-right font-mono">
                {row.size.toLocaleString()}
              </span>
              <span className="text-right font-mono opacity-70">
                {asks.slice(0, i + 1).reduce((sum, r) => sum + r.size, 0).toLocaleString()}
              </span>
              {isMid && (
                <div className="absolute inset-x-0 top-0 h-px bg-cyan-400/70 shadow-neon-cyan"></div>
              )}
            </div>
          );
        })}
        
        {/* Mid price indicator */}
        <div className="grid grid-cols-3 px-3 py-2 text-sm bg-slate-800/40 border-y border-slate-700">
          <span className="text-hl-accent font-bold font-mono">
            {midPrice.toFixed(2)}
          </span>
          <span className="text-center text-hl-muted text-xs">MID</span>
          <span className="text-right text-hl-muted text-xs">SPREAD</span>
        </div>
        
        {/* Bids (highest prices first) */}
        {bids.map((row, i) => {
          const intensity = Math.min(1, row.size / maxSize);
          const heat = `rgba(34,211,238,${0.08 + 0.22 * intensity})`;
          const isMid = Math.abs(row.price - midPrice) < 0.01;
          
          return (
            <div 
              key={`bid-${i}`} 
              className="ob-row grid grid-cols-3 px-3 py-1.5 text-sm relative hover:bg-slate-800/30 transition-colors"
              style={{ ['--heat' as any]: heat }}
            >
              <span className="text-cyan-300 font-mono">
                {row.price.toFixed(2)}
              </span>
              <span className="text-right font-mono">
                {row.size.toLocaleString()}
              </span>
              <span className="text-right font-mono opacity-70">
                {bids.slice(i).reduce((sum, r) => sum + r.size, 0).toLocaleString()}
              </span>
              {isMid && (
                <div className="absolute inset-x-0 top-0 h-px bg-cyan-400/70 shadow-neon-cyan"></div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
