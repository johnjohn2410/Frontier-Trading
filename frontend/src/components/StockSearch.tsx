import React, { useState } from 'react';
import { useStock } from '../contexts/StockContext';

export default function StockSearch() {
  const { selectedSymbol, setSelectedSymbol, searchSymbol, setSearchSymbol } = useStock();
  const [isSearching, setIsSearching] = useState(false);

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!searchSymbol.trim()) return;
    
    setIsSearching(true);
    
    // Simulate API call delay
    setTimeout(() => {
      setSelectedSymbol(searchSymbol.toUpperCase());
      setSearchSymbol('');
      setIsSearching(false);
    }, 500);
  };

  const handleSymbolClick = (symbol: string) => {
    setSelectedSymbol(symbol);
  };

  const popularSymbols = ['AAPL', 'MSFT', 'TSLA', 'SPY', 'QQQ', 'NVDA', 'GOOGL', 'AMZN'];

  return (
    <div className="p-3 border-b border-slate-800">
      <form onSubmit={handleSearch} className="mb-3">
        <div className="flex gap-2">
          <input
            type="text"
            value={searchSymbol}
            onChange={(e) => setSearchSymbol(e.target.value)}
            placeholder="Search symbol..."
            className="flex-1 px-3 py-2 bg-slate-900/50 border border-slate-700 rounded-lg text-hl-text placeholder-hl-muted focus:border-hl-accent focus:outline-none focus:ring-1 focus:ring-hl-accent/50"
            disabled={isSearching}
          />
          <button
            type="submit"
            disabled={isSearching || !searchSymbol.trim()}
            className="px-4 py-2 bg-hl-accent text-white rounded-lg hover:bg-hl-accent/80 disabled:opacity-50 disabled:cursor-not-allowed neon-btn"
          >
            {isSearching ? '...' : 'Search'}
          </button>
        </div>
      </form>
      
      <div className="mb-3">
        <div className="text-xs text-hl-muted mb-2">Current: {selectedSymbol}</div>
        <div className="text-xs text-hl-muted mb-2">Popular:</div>
        <div className="flex flex-wrap gap-1">
          {popularSymbols.map((symbol) => (
            <button
              key={symbol}
              onClick={() => handleSymbolClick(symbol)}
              className={`px-2 py-1 text-xs rounded border transition-colors ${
                selectedSymbol === symbol
                  ? 'bg-hl-accent/20 border-hl-accent text-hl-accent neon-btn'
                  : 'bg-slate-800/50 border-slate-700 text-hl-text hover:border-slate-600'
              }`}
            >
              {symbol}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
