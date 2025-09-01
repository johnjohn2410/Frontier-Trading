import React from "react";
import { useStream } from "../hooks/useStream";

type Pos = { 
  symbol: string; 
  qty: string; 
  avg_price: string; 
  unrealized: string; 
  realized: string;
  payload?: {
    symbol: string;
    quantity: string;
    average_price: string;
    market_price: string;
    realized_pnl: string;
    unrealized_pnl: string;
    market_value: string;
  };
};

export default function Positions() {
  const items = useStream<Pos>("ws://localhost:8000/ws/stream?s=positions.stream");
  
  return (
    <div>
      <h2 className="text-xl font-bold mb-4">Positions</h2>
      
      {items.length === 0 && (
        <div className="text-gray-500 text-center py-8">
          No positions yet. Place an order to see positions here.
        </div>
      )}
      
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b">
              <th className="text-left py-2">Symbol</th>
              <th className="text-right py-2">Qty</th>
              <th className="text-right py-2">Avg Price</th>
              <th className="text-right py-2">Unrealized</th>
              <th className="text-right py-2">Realized</th>
            </tr>
          </thead>
          <tbody>
            {items.map((p, i) => {
              const payload = p.payload || p;
              const unrealized = parseFloat(payload.unrealized_pnl || p.unrealized);
              const realized = parseFloat(payload.realized_pnl || p.realized);
              
              return (
                <tr key={i} className="border-b hover:bg-gray-50">
                  <td className="py-2 font-medium">{payload.symbol}</td>
                  <td className="py-2 text-right">{payload.quantity || p.qty}</td>
                  <td className="py-2 text-right">${parseFloat(payload.average_price || p.avg_price).toFixed(2)}</td>
                  <td className={`py-2 text-right ${unrealized >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                    ${unrealized.toFixed(2)}
                  </td>
                  <td className={`py-2 text-right ${realized >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                    ${realized.toFixed(2)}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
      
      {items.length > 0 && (
        <div className="mt-4 text-xs text-gray-500">
          Last updated: {new Date().toLocaleTimeString()}
        </div>
      )}
    </div>
  );
}
