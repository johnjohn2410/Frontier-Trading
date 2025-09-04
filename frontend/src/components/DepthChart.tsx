import React, { useState, useEffect } from "react";
import { useStock } from "../contexts/StockContext";

type DepthPoint = {
  price: number;
  bidCumulative: number;
  askCumulative: number;
};

// Mock depth data
const generateMockDepth = (): DepthPoint[] => {
  const basePrice = 238.50;
  const points: DepthPoint[] = [];
  
  // Generate depth points
  for (let i = -20; i <= 20; i++) {
    const price = basePrice + (i * 0.01);
    const distance = Math.abs(i);
    
    // Bid depth (below mid price)
    const bidDepth = i <= 0 ? Math.max(0, 1000 - distance * 50) : 0;
    
    // Ask depth (above mid price)  
    const askDepth = i >= 0 ? Math.max(0, 1000 - distance * 50) : 0;
    
    points.push({
      price,
      bidCumulative: bidDepth,
      askCumulative: askDepth
    });
  }
  
  return points;
};

export default function DepthChart() {
  const { selectedSymbol } = useStock();
  const [depthData, setDepthData] = useState<DepthPoint[]>([]);

  useEffect(() => {
    setDepthData(generateMockDepth());
    
    const interval = setInterval(() => {
      setDepthData(generateMockDepth());
    }, 3000);
    
    return () => clearInterval(interval);
  }, []);

  const maxDepth = Math.max(
    ...depthData.map(d => Math.max(d.bidCumulative, d.askCumulative))
  );

  return (
    <div className="rounded-xl2 bg-hl-panel/60 border border-slate-800 overflow-hidden neon-card">
      <div className="px-3 py-2 text-xs text-hl-muted border-b border-slate-800">
        {selectedSymbol} Market Depth
      </div>
      
      <div className="p-3 h-48 relative">
        <svg 
          width="100%" 
          height="100%" 
          viewBox="0 0 400 150" 
          className="absolute inset-0"
        >
          {/* Grid lines */}
          <defs>
            <pattern id="grid" width="40" height="20" patternUnits="userSpaceOnUse">
              <path d="M 40 0 L 0 0 0 20" fill="none" stroke="#1e293b" strokeWidth="0.5"/>
            </pattern>
          </defs>
          <rect width="100%" height="100%" fill="url(#grid)" />
          
          {/* Bid depth area (left side) */}
          <path
            d={`M 0,150 ${depthData
              .filter(d => d.bidCumulative > 0)
              .map((d, i) => {
                const x = (i / depthData.length) * 200;
                const y = 150 - (d.bidCumulative / maxDepth) * 150;
                return `L ${x},${y}`;
              })
              .join(' ')} L 200,150 Z`}
            fill="rgba(34,211,238,0.1)"
            stroke="rgba(34,211,238,0.6)"
            strokeWidth="1"
          />
          
          {/* Ask depth area (right side) */}
          <path
            d={`M 200,150 ${depthData
              .filter(d => d.askCumulative > 0)
              .map((d, i) => {
                const x = 200 + ((i / depthData.length) * 200);
                const y = 150 - (d.askCumulative / maxDepth) * 150;
                return `L ${x},${y}`;
              })
              .join(' ')} L 400,150 Z`}
            fill="rgba(244,63,94,0.1)"
            stroke="rgba(244,63,94,0.6)"
            strokeWidth="1"
          />
          
          {/* Mid price line */}
          <line
            x1="200"
            y1="0"
            x2="200"
            y2="150"
            stroke="#06b6d4"
            strokeWidth="1"
            strokeDasharray="2,2"
          />
          
          {/* Price labels */}
          <text x="10" y="15" fill="#94a3b8" fontSize="10" fontFamily="monospace">
            {depthData[0]?.price.toFixed(2)}
          </text>
          <text x="350" y="15" fill="#94a3b8" fontSize="10" fontFamily="monospace">
            {depthData[depthData.length - 1]?.price.toFixed(2)}
          </text>
          <text x="190" y="15" fill="#06b6d4" fontSize="10" fontFamily="monospace">
            238.50
          </text>
        </svg>
        
        {/* Depth info overlay */}
        <div className="absolute bottom-1 left-2 right-2 flex justify-between text-xs text-hl-muted">
          <span>Bid: {depthData.find(d => d.bidCumulative > 0)?.bidCumulative.toLocaleString()}</span>
          <span>Ask: {depthData.find(d => d.askCumulative > 0)?.askCumulative.toLocaleString()}</span>
        </div>
      </div>
    </div>
  );
}
