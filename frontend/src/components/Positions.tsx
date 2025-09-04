import React, { useEffect, useState } from "react";

type Position = {
  symbol: string;
  qty: string;
  avg_price: string;
  realized_pnl: string;
  unrealized_pnl: string;
};

interface PositionsProps {
  className?: string;
}

export default function Positions({ className = "" }: PositionsProps) {
  const [positions, setPositions] = useState<Position[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchPositions = async () => {
    try {
      const response = await fetch("http://localhost:8000/positions");
      if (response.ok) {
        const pos: Position[] = await response.json();
        setPositions(pos);
      }
    } catch (error) {
      console.error("Failed to fetch positions:", error);
    }
  };

  useEffect(() => {
    const loadData = async () => {
      setLoading(true);
      await fetchPositions();
      setLoading(false);
    };

    loadData();

    // Refresh positions every 10 seconds
    const interval = setInterval(fetchPositions, 10000);

    return () => clearInterval(interval);
  }, []);

  if (loading) {
    return (
      <div className={`rounded-xl2 bg-hl-panel/60 border border-slate-800 p-4 ${className}`}>
        <h3 className="text-sm font-semibold text-hl-text mb-3">Positions</h3>
        <div className="text-center py-8">
          <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-hl-accent mx-auto"></div>
          <p className="mt-2 text-hl-muted text-xs">Loading...</p>
        </div>
      </div>
    );
  }

  return (
    <div className={`rounded-xl2 bg-hl-panel/60 border border-slate-800 p-4 ${className}`}>
      <h3 className="text-sm font-semibold text-hl-text mb-3">Positions</h3>
      {positions.length === 0 ? (
        <div className="text-center py-8 text-hl-muted text-xs">
          No positions
        </div>
      ) : (
        <div className="space-y-2">
          {positions.map((pos, index) => {
            const unrealizedPnl = parseFloat(pos.unrealized_pnl);
            const realizedPnl = parseFloat(pos.realized_pnl);
            
            return (
              <div key={index} className="border border-slate-800 rounded-lg p-3 hover:bg-slate-800/20 transition-colors">
                <div className="flex justify-between items-center mb-2">
                  <div className="font-semibold text-hl-text text-sm">{pos.symbol}</div>
                  <div className="text-xs text-hl-muted">
                    {pos.qty} @ ${pos.avg_price}
                  </div>
                </div>
                <div className="grid grid-cols-2 gap-2 text-xs">
                  <div>
                    <span className="text-hl-muted">Unrealized: </span>
                    <span className={`font-mono ${unrealizedPnl >= 0 ? 'text-hl-buy' : 'text-hl-sell'}`}>
                      ${pos.unrealized_pnl}
                    </span>
                  </div>
                  <div>
                    <span className="text-hl-muted">Realized: </span>
                    <span className={`font-mono ${realizedPnl >= 0 ? 'text-hl-buy' : 'text-hl-sell'}`}>
                      ${pos.realized_pnl}
                    </span>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
