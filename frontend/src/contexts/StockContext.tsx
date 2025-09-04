import React, { createContext, useContext, useState, ReactNode } from 'react';

interface StockContextType {
  selectedSymbol: string;
  setSelectedSymbol: (symbol: string) => void;
  searchSymbol: string;
  setSearchSymbol: (symbol: string) => void;
}

const StockContext = createContext<StockContextType | undefined>(undefined);

export function StockProvider({ children }: { children: ReactNode }) {
  const [selectedSymbol, setSelectedSymbol] = useState('AAPL');
  const [searchSymbol, setSearchSymbol] = useState('');

  return (
    <StockContext.Provider value={{
      selectedSymbol,
      setSelectedSymbol,
      searchSymbol,
      setSearchSymbol
    }}>
      {children}
    </StockContext.Provider>
  );
}

export function useStock() {
  const context = useContext(StockContext);
  if (context === undefined) {
    throw new Error('useStock must be used within a StockProvider');
  }
  return context;
}
