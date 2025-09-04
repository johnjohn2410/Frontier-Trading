import React, { useEffect, useState } from "react";

type Quote = {
  schema_version: string;
  symbol: string;
  ts: number;
  price: string;
  source: string;
};

type Position = {
  symbol: string;
  qty: string;
  avg_price: string;
  realized_pnl: string;
  unrealized_pnl: string;
};

const SYMBOLS = ["AAPL", "MSFT", "TSLA", "SPY", "GOOGL", "AMZN"];

export default function MarketData() {
  const [quotes, setQuotes] = useState<Record<string, Quote>>({});
  const [positions, setPositions] = useState<Position[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchQuote = async (symbol: string) => {
    try {
      const response = await fetch(`http://localhost:8000/quote?symbol=${symbol}`);
      if (response.ok) {
        const quote: Quote = await response.json();
        setQuotes(prev => ({ ...prev, [symbol]: quote }));
      }
    } catch (error) {
      console.error(`Failed to fetch quote for ${symbol}:`, error);
    }
  };

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
      await Promise.all([
        ...SYMBOLS.map(symbol => fetchQuote(symbol)),
        fetchPositions()
      ]);
      setLoading(false);
    };

    loadData();

    // Refresh quotes every 5 seconds
    const interval = setInterval(() => {
      SYMBOLS.forEach(symbol => fetchQuote(symbol));
    }, 5000);

    return () => clearInterval(interval);
  }, []);

  if (loading) {
    return (
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-xl font-bold mb-4">Market Data</h2>
        <div className="text-center py-8">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900 mx-auto"></div>
          <p className="mt-2 text-gray-600">Loading market data...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Market Quotes */}
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-xl font-bold mb-4">Live Market Quotes</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {SYMBOLS.map(symbol => {
            const quote = quotes[symbol];
            return (
              <div key={symbol} className="border rounded-lg p-4 hover:shadow-md transition-shadow">
                <div className="flex justify-between items-center">
                  <div className="font-bold text-lg">{symbol}</div>
                  <div className="text-sm text-gray-500">
                    {quote ? new Date(quote.ts).toLocaleTimeString() : 'No data'}
                  </div>
                </div>
                <div className="mt-2">
                  {quote ? (
                    <div className="text-2xl font-bold text-green-600">
                      ${parseFloat(quote.price).toFixed(2)}
                    </div>
                  ) : (
                    <div className="text-2xl font-bold text-gray-400">--</div>
                  )}
                </div>
                <div className="text-xs text-gray-500 mt-1">
                  {quote ? `Source: ${quote.source}` : 'Loading...'}
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Positions */}
      <div className="bg-white rounded-lg shadow p-6">
        <h2 className="text-xl font-bold mb-4">Portfolio Positions</h2>
        {positions.length === 0 ? (
          <div className="text-center py-8 text-gray-500">
            No positions found
          </div>
        ) : (
          <div className="space-y-3">
            {positions.map((pos, index) => (
              <div key={index} className="border rounded-lg p-4">
                <div className="flex justify-between items-center">
                  <div className="font-bold text-lg">{pos.symbol}</div>
                  <div className="text-sm text-gray-500">
                    {pos.qty} shares @ ${pos.avg_price}
                  </div>
                </div>
                <div className="mt-2 flex justify-between">
                  <div>
                    <span className="text-sm text-gray-600">Unrealized P&L: </span>
                    <span className={`font-bold ${parseFloat(pos.unrealized_pnl) >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                      ${pos.unrealized_pnl}
                    </span>
                  </div>
                  <div>
                    <span className="text-sm text-gray-600">Realized P&L: </span>
                    <span className={`font-bold ${parseFloat(pos.realized_pnl) >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                      ${pos.realized_pnl}
                    </span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
