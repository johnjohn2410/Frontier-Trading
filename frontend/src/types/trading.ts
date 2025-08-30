// Trading platform type definitions

export enum AssetType {
  Stock = 'Stock',
  Etf = 'Etf',
  Crypto = 'Crypto',
  Forex = 'Forex',
  Futures = 'Futures',
  Options = 'Options',
}

export enum OrderType {
  Market = 'Market',
  Limit = 'Limit',
  Stop = 'Stop',
  StopLimit = 'StopLimit',
  TrailingStop = 'TrailingStop',
}

export enum OrderSide {
  Buy = 'Buy',
  Sell = 'Sell',
}

export enum OrderStatus {
  Pending = 'Pending',
  Partial = 'Partial',
  Filled = 'Filled',
  Cancelled = 'Cancelled',
  Rejected = 'Rejected',
  Expired = 'Expired',
}

export enum TimeInForce {
  Day = 'Day',
  Gtc = 'Gtc',
  Ioc = 'Ioc',
  Fok = 'Fok',
}

export interface Price {
  value: number;
  precision: number;
}

export interface Quantity {
  value: number;
  precision: number;
}

export interface Asset {
  symbol: string;
  exchange: string;
  assetType: AssetType;
  name: string;
  currency: string;
  tickSize: Price;
  lotSize: Quantity;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface MarketTick {
  asset: Asset;
  bid: Price;
  ask: Price;
  last: Price;
  bidSize: Quantity;
  askSize: Quantity;
  volume: Quantity;
  timestamp: string;
}

export interface Order {
  id: string;
  clientOrderId: string;
  userId: string;
  asset: Asset;
  orderType: OrderType;
  side: OrderSide;
  quantity: Quantity;
  limitPrice?: Price;
  stopPrice?: Price;
  timeInForce: TimeInForce;
  status: OrderStatus;
  filledQuantity: Quantity;
  averageFillPrice?: Price;
  commission: number;
  createdAt: string;
  updatedAt: string;
  expiresAt?: string;
}

export interface Trade {
  id: string;
  orderId: string;
  userId: string;
  asset: Asset;
  side: OrderSide;
  quantity: Quantity;
  price: Price;
  commission: number;
  exchange: string;
  timestamp: string;
}

export interface Position {
  id: string;
  userId: string;
  asset: Asset;
  quantity: Quantity;
  averagePrice: Price;
  currentPrice: Price;
  unrealizedPnl: number;
  realizedPnl: number;
  marketValue: number;
  createdAt: string;
  updatedAt: string;
}

export interface Account {
  id: string;
  userId: string;
  name: string;
  currency: string;
  cash: number;
  buyingPower: number;
  equity: number;
  marginUsed: number;
  marginAvailable: number;
  isPaperTrading: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface RiskLimits {
  id: string;
  userId: string;
  maxPositionSize: number;
  maxDailyLoss: number;
  maxDrawdown: number;
  maxLeverage: number;
  allowShortSelling: boolean;
  allowOptions: boolean;
  allowFutures: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface RiskMetrics {
  totalPnl: number;
  dailyPnl: number;
  maxDrawdown: number;
  currentDrawdown: number;
  portfolioValue: number;
  marginUsed: number;
  marginAvailable: number;
  leverage: number;
  beta: number;
  sharpeRatio: number;
  volatility: number;
  timestamp: string;
}

export interface RiskViolation {
  id: string;
  userId: string;
  violationType: ViolationType;
  message: string;
  currentValue: number;
  limitValue: number;
  timestamp: string;
}

export enum ViolationType {
  PositionSize = 'PositionSize',
  DailyLoss = 'DailyLoss',
  Drawdown = 'Drawdown',
  Leverage = 'Leverage',
  Concentration = 'Concentration',
  Margin = 'Margin',
}

export interface OrderBookEntry {
  price: Price;
  quantity: Quantity;
  orderId: string;
  timestamp: string;
}

export interface OrderBook {
  symbol: string;
  bids: OrderBookEntry[];
  asks: OrderBookEntry[];
  timestamp: string;
}

export interface User {
  id: string;
  email: string;
  username: string;
  firstName: string;
  lastName: string;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

// API Request/Response types
export interface CreateOrderRequest {
  symbol: string;
  orderType: OrderType;
  side: OrderSide;
  quantity: number;
  limitPrice?: number;
  stopPrice?: number;
  timeInForce: TimeInForce;
  clientOrderId?: string;
}

export interface CreateOrderResponse {
  order: Order;
  message: string;
}

export interface CancelOrderRequest {
  orderId: string;
}

export interface CancelOrderResponse {
  success: boolean;
  message: string;
}

export interface GetOrdersRequest {
  symbol?: string;
  status?: OrderStatus;
  limit?: number;
  offset?: number;
}

export interface GetOrdersResponse {
  orders: Order[];
  total: number;
  limit: number;
  offset: number;
}

export interface GetPositionsResponse {
  positions: Position[];
  totalValue: number;
  totalPnl: number;
}

export interface GetAccountResponse {
  account: Account;
  riskMetrics: RiskMetrics;
}

// WebSocket message types
export type WebSocketMessage = 
  | { type: 'MarketTick'; data: MarketTick }
  | { type: 'OrderUpdate'; data: Order }
  | { type: 'TradeExecution'; data: Trade }
  | { type: 'PositionUpdate'; data: Position }
  | { type: 'AccountUpdate'; data: Account }
  | { type: 'RiskViolation'; data: RiskViolation }
  | { type: 'OrderBookUpdate'; data: OrderBook }
  | { type: 'Error'; data: { message: string } }
  | { type: 'Ping' }
  | { type: 'Pong' };

// Chart data types
export interface CandlestickData {
  time: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}

export interface TechnicalIndicator {
  name: string;
  values: Array<{ time: number; value: number }>;
  color: string;
}

// AI Copilot types
export interface CopilotInsight {
  id: string;
  type: string;
  title: string;
  description: string;
  confidence: 'low' | 'medium' | 'high';
  riskLevel: 'low' | 'medium' | 'high' | 'extreme';
  sources: string[];
  timestamp: string;
  actionable: boolean;
  suggestedActions: string[];
  dataPoints: Record<string, any>;
}

// UI State types
export interface TradingState {
  selectedSymbol: string;
  selectedTimeframe: string;
  isChartFullscreen: boolean;
  orderPanelOpen: boolean;
  positionPanelOpen: boolean;
  copilotPanelOpen: boolean;
}

export interface Theme {
  mode: 'light' | 'dark';
  primary: string;
  secondary: string;
  accent: string;
  background: string;
  surface: string;
  text: string;
  textSecondary: string;
  border: string;
  success: string;
  warning: string;
  error: string;
}

// Form types
export interface OrderFormData {
  symbol: string;
  orderType: OrderType;
  side: OrderSide;
  quantity: number;
  limitPrice?: number;
  stopPrice?: number;
  timeInForce: TimeInForce;
  clientOrderId?: string;
}

export interface RiskSettingsFormData {
  maxPositionSize: number;
  maxDailyLoss: number;
  maxDrawdown: number;
  maxLeverage: number;
  allowShortSelling: boolean;
  allowOptions: boolean;
  allowFutures: boolean;
}

// Notification types
export interface Notification {
  id: string;
  type: 'success' | 'warning' | 'error' | 'info';
  title: string;
  message: string;
  timestamp: string;
  read: boolean;
  action?: {
    label: string;
    onClick: () => void;
  };
}

// Hotkey types
export interface Hotkey {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  action: string;
  description: string;
}

// Filter types
export interface OrderFilter {
  symbol?: string;
  status?: OrderStatus[];
  side?: OrderSide[];
  dateRange?: {
    start: string;
    end: string;
  };
}

export interface PositionFilter {
  symbol?: string;
  pnlRange?: {
    min: number;
    max: number;
  };
  sizeRange?: {
    min: number;
    max: number;
  };
}
