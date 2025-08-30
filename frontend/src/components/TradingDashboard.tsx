import React, { useEffect, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { 
  BarChart3, 
  TrendingUp, 
  DollarSign, 
  AlertTriangle, 
  Bot, 
  Settings,
  Maximize2,
  Minimize2,
  X
} from 'lucide-react';
import { useTradingStore, useTradingUI, useAccount, useRiskMetrics } from '../stores/tradingStore';
import { TradingChart } from './TradingChart';
import { OrderPanel } from './OrderPanel';
import { PositionsPanel } from './PositionsPanel';
import { CopilotPanel } from './CopilotPanel';
import { OrderBook } from './OrderBook';
import { MarketData } from './MarketData';
import { RiskMetrics } from './RiskMetrics';
import { Hotkeys } from './Hotkeys';

export const TradingDashboard: React.FC = () => {
  const {
    ui,
    setChartFullscreen,
    setOrderPanelOpen,
    setPositionPanelOpen,
    setCopilotPanelOpen,
  } = useTradingStore();

  const account = useAccount();
  const riskMetrics = useRiskMetrics();
  const [activeTab, setActiveTab] = useState<'chart' | 'orders' | 'positions'>('chart');

  // Hotkeys
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Ctrl/Cmd + Enter: Toggle order panel
      if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
        event.preventDefault();
        setOrderPanelOpen(!ui.orderPanelOpen);
      }
      
      // Ctrl/Cmd + P: Toggle positions panel
      if ((event.ctrlKey || event.metaKey) && event.key === 'p') {
        event.preventDefault();
        setPositionPanelOpen(!ui.positionPanelOpen);
      }
      
      // Ctrl/Cmd + C: Toggle copilot panel
      if ((event.ctrlKey || event.metaKey) && event.key === 'c') {
        event.preventDefault();
        setCopilotPanelOpen(!ui.copilotPanelOpen);
      }
      
      // F11: Toggle fullscreen chart
      if (event.key === 'F11') {
        event.preventDefault();
        setChartFullscreen(!ui.isChartFullscreen);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [ui, setOrderPanelOpen, setPositionPanelOpen, setCopilotPanelOpen, setChartFullscreen]);

  return (
    <div className="h-screen bg-gray-900 text-white overflow-hidden">
      {/* Header */}
      <header className="h-16 bg-gray-800 border-b border-gray-700 flex items-center justify-between px-6">
        <div className="flex items-center space-x-4">
          <h1 className="text-xl font-bold text-blue-400">Frontier Trading</h1>
          <div className="flex items-center space-x-2 text-sm text-gray-300">
            <span>Paper Trading</span>
            <div className="w-2 h-2 bg-green-400 rounded-full"></div>
          </div>
        </div>
        
        <div className="flex items-center space-x-4">
          {/* Account Summary */}
          <div className="flex items-center space-x-4 text-sm">
            <div className="text-right">
              <div className="text-gray-400">Portfolio Value</div>
              <div className="font-semibold">
                ${account?.equity?.toLocaleString() || '0.00'}
              </div>
            </div>
            <div className="text-right">
              <div className="text-gray-400">Daily P&L</div>
              <div className={`font-semibold ${
                (riskMetrics?.dailyPnl || 0) >= 0 ? 'text-green-400' : 'text-red-400'
              }`}>
                ${(riskMetrics?.dailyPnl || 0).toFixed(2)}
              </div>
            </div>
          </div>
          
          {/* Action Buttons */}
          <div className="flex items-center space-x-2">
            <button
              onClick={() => setOrderPanelOpen(!ui.orderPanelOpen)}
              className="p-2 rounded-lg bg-blue-600 hover:bg-blue-700 transition-colors"
              title="Order Panel (Ctrl+Enter)"
            >
              <BarChart3 size={16} />
            </button>
            <button
              onClick={() => setPositionPanelOpen(!ui.positionPanelOpen)}
              className="p-2 rounded-lg bg-green-600 hover:bg-green-700 transition-colors"
              title="Positions (Ctrl+P)"
            >
              <TrendingUp size={16} />
            </button>
            <button
              onClick={() => setCopilotPanelOpen(!ui.copilotPanelOpen)}
              className="p-2 rounded-lg bg-purple-600 hover:bg-purple-700 transition-colors"
              title="AI Copilot (Ctrl+C)"
            >
              <Bot size={16} />
            </button>
            <button
              onClick={() => setChartFullscreen(!ui.isChartFullscreen)}
              className="p-2 rounded-lg bg-gray-600 hover:bg-gray-700 transition-colors"
              title="Fullscreen Chart (F11)"
            >
              {ui.isChartFullscreen ? <Minimize2 size={16} /> : <Maximize2 size={16} />}
            </button>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <div className="flex h-[calc(100vh-4rem)]">
        {/* Left Sidebar - Market Data & Order Book */}
        <div className="w-80 bg-gray-800 border-r border-gray-700 flex flex-col">
          <div className="p-4 border-b border-gray-700">
            <h2 className="text-lg font-semibold mb-4">Market Data</h2>
            <MarketData />
          </div>
          
          <div className="flex-1 p-4">
            <h2 className="text-lg font-semibold mb-4">Order Book</h2>
            <OrderBook symbol={ui.selectedSymbol} />
          </div>
        </div>

        {/* Center - Chart */}
        <div className="flex-1 flex flex-col">
          {/* Chart Tabs */}
          <div className="h-12 bg-gray-800 border-b border-gray-700 flex items-center px-4">
            <div className="flex space-x-1">
              <button
                onClick={() => setActiveTab('chart')}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                  activeTab === 'chart'
                    ? 'bg-blue-600 text-white'
                    : 'text-gray-400 hover:text-white hover:bg-gray-700'
                }`}
              >
                Chart
              </button>
              <button
                onClick={() => setActiveTab('orders')}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                  activeTab === 'orders'
                    ? 'bg-blue-600 text-white'
                    : 'text-gray-400 hover:text-white hover:bg-gray-700'
                }`}
              >
                Orders
              </button>
              <button
                onClick={() => setActiveTab('positions')}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                  activeTab === 'positions'
                    ? 'bg-blue-600 text-white'
                    : 'text-gray-400 hover:text-white hover:bg-gray-700'
                }`}
              >
                Positions
              </button>
            </div>
          </div>

          {/* Chart Area */}
          <div className="flex-1 relative">
            <AnimatePresence mode="wait">
              {activeTab === 'chart' && (
                <motion.div
                  key="chart"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                  className="h-full"
                >
                  <TradingChart />
                </motion.div>
              )}
              
              {activeTab === 'orders' && (
                <motion.div
                  key="orders"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                  className="h-full p-4"
                >
                  <OrderPanel />
                </motion.div>
              )}
              
              {activeTab === 'positions' && (
                <motion.div
                  key="positions"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                  className="h-full p-4"
                >
                  <PositionsPanel />
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        </div>

        {/* Right Sidebar - Risk Metrics & Copilot */}
        <div className="w-80 bg-gray-800 border-l border-gray-700 flex flex-col">
          <div className="p-4 border-b border-gray-700">
            <h2 className="text-lg font-semibold mb-4">Risk Metrics</h2>
            <RiskMetrics />
          </div>
          
          <div className="flex-1 p-4">
            <h2 className="text-lg font-semibold mb-4">AI Copilot</h2>
            <CopilotPanel />
          </div>
        </div>
      </div>

      {/* Floating Panels */}
      <AnimatePresence>
        {ui.orderPanelOpen && (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 20 }}
            className="fixed inset-4 bg-gray-800 border border-gray-700 rounded-lg shadow-2xl z-50"
          >
            <div className="h-full flex flex-col">
              <div className="flex items-center justify-between p-4 border-b border-gray-700">
                <h2 className="text-lg font-semibold">Order Panel</h2>
                <button
                  onClick={() => setOrderPanelOpen(false)}
                  className="p-1 hover:bg-gray-700 rounded"
                >
                  <X size={20} />
                </button>
              </div>
              <div className="flex-1 overflow-auto">
                <OrderPanel />
              </div>
            </div>
          </motion.div>
        )}

        {ui.positionPanelOpen && (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 20 }}
            className="fixed inset-4 bg-gray-800 border border-gray-700 rounded-lg shadow-2xl z-50"
          >
            <div className="h-full flex flex-col">
              <div className="flex items-center justify-between p-4 border-b border-gray-700">
                <h2 className="text-lg font-semibold">Positions</h2>
                <button
                  onClick={() => setPositionPanelOpen(false)}
                  className="p-1 hover:bg-gray-700 rounded"
                >
                  <X size={20} />
                </button>
              </div>
              <div className="flex-1 overflow-auto">
                <PositionsPanel />
              </div>
            </div>
          </motion.div>
        )}

        {ui.copilotPanelOpen && (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 20 }}
            className="fixed inset-4 bg-gray-800 border border-gray-700 rounded-lg shadow-2xl z-50"
          >
            <div className="h-full flex flex-col">
              <div className="flex items-center justify-between p-4 border-b border-gray-700">
                <h2 className="text-lg font-semibold">AI Copilot</h2>
                <button
                  onClick={() => setCopilotPanelOpen(false)}
                  className="p-1 hover:bg-gray-700 rounded"
                >
                  <X size={20} />
                </button>
              </div>
              <div className="flex-1 overflow-auto">
                <CopilotPanel />
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Hotkeys Help */}
      <Hotkeys />
    </div>
  );
};
