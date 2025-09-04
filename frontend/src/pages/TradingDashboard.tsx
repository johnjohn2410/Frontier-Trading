import React from "react";
import { StockProvider } from "../contexts/StockContext";
import Watchlist from "../components/Watchlist";
import StockSearch from "../components/StockSearch";
import ChartArea from "../components/ChartArea";
import OrderBook from "../components/OrderBook";
import DepthChart from "../components/DepthChart";
import TradesTape from "../components/TradesTape";
import OrderTicket from "../components/OrderTicket";
import Positions from "../components/Positions";
import Suggestions from "../components/Suggestions";

export default function TradingDashboard() {
  return (
    <StockProvider>
      <div className="grid h-screen grid-cols-1 md:grid-cols-[280px_minmax(0,1fr)_360px]">
        {/* Left: Watchlist */}
        <aside className="hidden md:flex flex-col border-r border-slate-800 bg-hl-panel/60 neon-card">
          <div className="p-4 border-b border-slate-800">
            <h1 className="text-xl font-bold text-hl-text">Frontier Trading</h1>
            <p className="text-sm text-hl-muted">AI-Powered Terminal</p>
          </div>
          <StockSearch />
          <Watchlist />
        </aside>

      {/* Center: Chart + Orderbook */}
      <main className="flex flex-col">
        <ChartArea />
        <div className="grid grid-cols-1 xl:grid-cols-2 gap-2 p-2">
          <OrderBook />
          <DepthChart />
        </div>
        <TradesTape />
      </main>

        {/* Right: Ticket + Positions + Copilot */}
        <aside className="flex flex-col gap-2 p-2 bg-hl-panel/50 border-l border-slate-800 neon-card">
          <OrderTicket />
          <Positions className="flex-1" />
          <Suggestions />
        </aside>
      </div>
    </StockProvider>
  );
}
