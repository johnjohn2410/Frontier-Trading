import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';
import { 
  Order, 
  Position, 
  Account, 
  MarketTick, 
  OrderBook, 
  RiskMetrics, 
  RiskViolation,
  CopilotInsight,
  TradingState,
  WebSocketMessage,
  OrderStatus,
  OrderSide
} from '../types/trading';

interface TradingStore {
  // Real-time market data
  marketTicks: Record<string, MarketTick>;
  orderBooks: Record<string, OrderBook>;
  
  // Trading state
  orders: Order[];
  positions: Position[];
  account: Account | null;
  riskMetrics: RiskMetrics | null;
  riskViolations: RiskViolation[];
  
  // AI Copilot
  copilotInsights: CopilotInsight[];
  
  // UI state
  ui: TradingState;
  
  // Loading states
  isLoading: {
    orders: boolean;
    positions: boolean;
    account: boolean;
    marketData: boolean;
  };
  
  // Error states
  errors: {
    orders: string | null;
    positions: string | null;
    account: string | null;
    marketData: string | null;
  };
  
  // Actions
  // Market data actions
  updateMarketTick: (tick: MarketTick) => void;
  updateOrderBook: (orderBook: OrderBook) => void;
  
  // Order actions
  addOrder: (order: Order) => void;
  updateOrder: (orderId: string, updates: Partial<Order>) => void;
  removeOrder: (orderId: string) => void;
  setOrders: (orders: Order[]) => void;
  
  // Position actions
  addPosition: (position: Position) => void;
  updatePosition: (positionId: string, updates: Partial<Position>) => void;
  removePosition: (positionId: string) => void;
  setPositions: (positions: Position[]) => void;
  
  // Account actions
  setAccount: (account: Account) => void;
  updateAccount: (updates: Partial<Account>) => void;
  
  // Risk actions
  setRiskMetrics: (metrics: RiskMetrics) => void;
  addRiskViolation: (violation: RiskViolation) => void;
  clearRiskViolations: () => void;
  
  // Copilot actions
  addCopilotInsight: (insight: CopilotInsight) => void;
  setCopilotInsights: (insights: CopilotInsight[]) => void;
  removeCopilotInsight: (insightId: string) => void;
  
  // UI actions
  setSelectedSymbol: (symbol: string) => void;
  setSelectedTimeframe: (timeframe: string) => void;
  setChartFullscreen: (fullscreen: boolean) => void;
  setOrderPanelOpen: (open: boolean) => void;
  setPositionPanelOpen: (open: boolean) => void;
  setCopilotPanelOpen: (open: boolean) => void;
  
  // Loading actions
  setLoading: (key: keyof TradingStore['isLoading'], loading: boolean) => void;
  
  // Error actions
  setError: (key: keyof TradingStore['errors'], error: string | null) => void;
  
  // WebSocket message handler
  handleWebSocketMessage: (message: WebSocketMessage) => void;
  
  // Computed values
  getActiveOrders: () => Order[];
  getOrdersBySymbol: (symbol: string) => Order[];
  getPositionBySymbol: (symbol: string) => Position | null;
  getTotalPnL: () => number;
  getDailyPnL: () => number;
  getPortfolioValue: () => number;
}

export const useTradingStore = create<TradingStore>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    marketTicks: {},
    orderBooks: {},
    orders: [],
    positions: [],
    account: null,
    riskMetrics: null,
    riskViolations: [],
    copilotInsights: [],
    ui: {
      selectedSymbol: 'AAPL',
      selectedTimeframe: '1D',
      isChartFullscreen: false,
      orderPanelOpen: false,
      positionPanelOpen: false,
      copilotPanelOpen: false,
    },
    isLoading: {
      orders: false,
      positions: false,
      account: false,
      marketData: false,
    },
    errors: {
      orders: null,
      positions: null,
      account: null,
      marketData: null,
    },
    
    // Market data actions
    updateMarketTick: (tick: MarketTick) => {
      set((state) => ({
        marketTicks: {
          ...state.marketTicks,
          [tick.asset.symbol]: tick,
        },
      }));
    },
    
    updateOrderBook: (orderBook: OrderBook) => {
      set((state) => ({
        orderBooks: {
          ...state.orderBooks,
          [orderBook.symbol]: orderBook,
        },
      }));
    },
    
    // Order actions
    addOrder: (order: Order) => {
      set((state) => ({
        orders: [...state.orders, order],
      }));
    },
    
    updateOrder: (orderId: string, updates: Partial<Order>) => {
      set((state) => ({
        orders: state.orders.map((order) =>
          order.id === orderId ? { ...order, ...updates } : order
        ),
      }));
    },
    
    removeOrder: (orderId: string) => {
      set((state) => ({
        orders: state.orders.filter((order) => order.id !== orderId),
      }));
    },
    
    setOrders: (orders: Order[]) => {
      set({ orders });
    },
    
    // Position actions
    addPosition: (position: Position) => {
      set((state) => ({
        positions: [...state.positions, position],
      }));
    },
    
    updatePosition: (positionId: string, updates: Partial<Position>) => {
      set((state) => ({
        positions: state.positions.map((position) =>
          position.id === positionId ? { ...position, ...updates } : position
        ),
      }));
    },
    
    removePosition: (positionId: string) => {
      set((state) => ({
        positions: state.positions.filter((position) => position.id !== positionId),
      }));
    },
    
    setPositions: (positions: Position[]) => {
      set({ positions });
    },
    
    // Account actions
    setAccount: (account: Account) => {
      set({ account });
    },
    
    updateAccount: (updates: Partial<Account>) => {
      set((state) => ({
        account: state.account ? { ...state.account, ...updates } : null,
      }));
    },
    
    // Risk actions
    setRiskMetrics: (metrics: RiskMetrics) => {
      set({ riskMetrics: metrics });
    },
    
    addRiskViolation: (violation: RiskViolation) => {
      set((state) => ({
        riskViolations: [...state.riskViolations, violation],
      }));
    },
    
    clearRiskViolations: () => {
      set({ riskViolations: [] });
    },
    
    // Copilot actions
    addCopilotInsight: (insight: CopilotInsight) => {
      set((state) => ({
        copilotInsights: [...state.copilotInsights, insight],
      }));
    },
    
    setCopilotInsights: (insights: CopilotInsight[]) => {
      set({ copilotInsights: insights });
    },
    
    removeCopilotInsight: (insightId: string) => {
      set((state) => ({
        copilotInsights: state.copilotInsights.filter(
          (insight) => insight.id !== insightId
        ),
      }));
    },
    
    // UI actions
    setSelectedSymbol: (symbol: string) => {
      set((state) => ({
        ui: { ...state.ui, selectedSymbol: symbol },
      }));
    },
    
    setSelectedTimeframe: (timeframe: string) => {
      set((state) => ({
        ui: { ...state.ui, selectedTimeframe: timeframe },
      }));
    },
    
    setChartFullscreen: (fullscreen: boolean) => {
      set((state) => ({
        ui: { ...state.ui, isChartFullscreen: fullscreen },
      }));
    },
    
    setOrderPanelOpen: (open: boolean) => {
      set((state) => ({
        ui: { ...state.ui, orderPanelOpen: open },
      }));
    },
    
    setPositionPanelOpen: (open: boolean) => {
      set((state) => ({
        ui: { ...state.ui, positionPanelOpen: open },
      }));
    },
    
    setCopilotPanelOpen: (open: boolean) => {
      set((state) => ({
        ui: { ...state.ui, copilotPanelOpen: open },
      }));
    },
    
    // Loading actions
    setLoading: (key: keyof TradingStore['isLoading'], loading: boolean) => {
      set((state) => ({
        isLoading: { ...state.isLoading, [key]: loading },
      }));
    },
    
    // Error actions
    setError: (key: keyof TradingStore['errors'], error: string | null) => {
      set((state) => ({
        errors: { ...state.errors, [key]: error },
      }));
    },
    
    // WebSocket message handler
    handleWebSocketMessage: (message: WebSocketMessage) => {
      const { updateMarketTick, updateOrderBook, addOrder, updateOrder, addPosition, updatePosition, setAccount, setRiskMetrics, addRiskViolation, addCopilotInsight } = get();
      
      switch (message.type) {
        case 'MarketTick':
          updateMarketTick(message.data);
          break;
        case 'OrderBookUpdate':
          updateOrderBook(message.data);
          break;
        case 'OrderUpdate':
          updateOrder(message.data.id, message.data);
          break;
        case 'TradeExecution':
          // Handle trade execution - might need to update orders and positions
          break;
        case 'PositionUpdate':
          updatePosition(message.data.id, message.data);
          break;
        case 'AccountUpdate':
          setAccount(message.data);
          break;
        case 'RiskViolation':
          addRiskViolation(message.data);
          break;
        case 'Error':
          console.error('WebSocket error:', message.data.message);
          break;
        default:
          console.warn('Unknown WebSocket message type:', message.type);
      }
    },
    
    // Computed values
    getActiveOrders: () => {
      const { orders } = get();
      return orders.filter(
        (order) => 
          order.status === OrderStatus.Pending || 
          order.status === OrderStatus.Partial
      );
    },
    
    getOrdersBySymbol: (symbol: string) => {
      const { orders } = get();
      return orders.filter((order) => order.asset.symbol === symbol);
    },
    
    getPositionBySymbol: (symbol: string) => {
      const { positions } = get();
      return positions.find((position) => position.asset.symbol === symbol) || null;
    },
    
    getTotalPnL: () => {
      const { positions } = get();
      return positions.reduce((total, position) => total + position.unrealizedPnl + position.realizedPnl, 0);
    },
    
    getDailyPnL: () => {
      const { riskMetrics } = get();
      return riskMetrics?.dailyPnl || 0;
    },
    
    getPortfolioValue: () => {
      const { account, positions } = get();
      const cash = account?.cash || 0;
      const positionsValue = positions.reduce((total, position) => total + position.marketValue, 0);
      return cash + positionsValue;
    },
  }))
);

// Selectors for performance optimization
export const useMarketTick = (symbol: string) =>
  useTradingStore((state) => state.marketTicks[symbol]);

export const useOrderBook = (symbol: string) =>
  useTradingStore((state) => state.orderBooks[symbol]);

export const useActiveOrders = () =>
  useTradingStore((state) => state.getActiveOrders());

export const usePositions = () =>
  useTradingStore((state) => state.positions);

export const useAccount = () =>
  useTradingStore((state) => state.account);

export const useRiskMetrics = () =>
  useTradingStore((state) => state.riskMetrics);

export const useCopilotInsights = () =>
  useTradingStore((state) => state.copilotInsights);

export const useTradingUI = () =>
  useTradingStore((state) => state.ui);

// Persist important state to localStorage
useTradingStore.subscribe(
  (state) => ({
    selectedSymbol: state.ui.selectedSymbol,
    selectedTimeframe: state.ui.selectedTimeframe,
  }),
  (state) => {
    localStorage.setItem('trading-ui-state', JSON.stringify(state));
  }
);

// Load persisted state on initialization
const loadPersistedState = () => {
  try {
    const persisted = localStorage.getItem('trading-ui-state');
    if (persisted) {
      const state = JSON.parse(persisted);
      useTradingStore.getState().setSelectedSymbol(state.selectedSymbol || 'AAPL');
      useTradingStore.getState().setSelectedTimeframe(state.selectedTimeframe || '1D');
    }
  } catch (error) {
    console.warn('Failed to load persisted state:', error);
  }
};

// Initialize on module load
loadPersistedState();
