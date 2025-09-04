import React, { useState } from "react";

type OrderType = 'market' | 'limit' | 'stop';
type OrderSide = 'buy' | 'sell';

export default function OrderTicket() {
  const [side, setSide] = useState<OrderSide>('buy');
  const [orderType, setOrderType] = useState<OrderType>('market');
  const [quantity, setQuantity] = useState('1');
  const [price, setPrice] = useState('238.50');
  const [stopPrice, setStopPrice] = useState('');

  const handleSubmit = async () => {
    try {
      const orderData = {
        symbol: 'AAPL',
        side,
        qty: quantity,
        order_type: orderType,
        ...(orderType === 'limit' && { limit_price: price }),
        ...(orderType === 'stop' && { stop_price: stopPrice }),
      };

      const response = await fetch('http://localhost:8000/orders', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(orderData),
      });

      const result = await response.json();
      console.log('Order placed:', result);
      
      // Reset form
      setQuantity('1');
      setPrice('238.50');
      setStopPrice('');
    } catch (error) {
      console.error('Failed to place order:', error);
    }
  };

  return (
    <div className="rounded-xl2 bg-hl-panel/60 border border-slate-800 p-4">
      <div className="text-sm font-semibold text-hl-text mb-4">Order Ticket</div>
      
      {/* Buy/Sell buttons */}
      <div className="grid grid-cols-2 gap-2 mb-4">
        <button
          onClick={() => setSide('buy')}
          className={`neon-btn neon-btn--buy font-semibold ${
            side === 'buy' ? 'shadow-neon-cyan' : 'opacity-50'
          }`}
        >
          BUY
        </button>
        <button
          onClick={() => setSide('sell')}
          className={`neon-btn neon-btn--sell font-semibold ${
            side === 'sell' ? 'shadow-neon-rose' : 'opacity-50'
          }`}
        >
          SELL
        </button>
      </div>

      {/* Order type */}
      <div className="mb-4">
        <label className="block text-xs text-hl-muted mb-2">Order Type</label>
        <select
          value={orderType}
          onChange={(e) => setOrderType(e.target.value as OrderType)}
          className="w-full px-3 py-2 bg-slate-800/50 border border-slate-700 rounded-lg text-hl-text text-sm focus:border-hl-accent focus:outline-none"
        >
          <option value="market">Market</option>
          <option value="limit">Limit</option>
          <option value="stop">Stop</option>
        </select>
      </div>

      {/* Quantity */}
      <div className="mb-4">
        <label className="block text-xs text-hl-muted mb-2">Quantity</label>
        <input
          type="number"
          value={quantity}
          onChange={(e) => setQuantity(e.target.value)}
          className="w-full px-3 py-2 bg-slate-800/50 border border-slate-700 rounded-lg text-hl-text text-sm font-mono focus:border-hl-accent focus:outline-none"
          placeholder="1"
        />
      </div>

      {/* Price (for limit orders) */}
      {orderType === 'limit' && (
        <div className="mb-4">
          <label className="block text-xs text-hl-muted mb-2">Limit Price</label>
          <input
            type="number"
            step="0.01"
            value={price}
            onChange={(e) => setPrice(e.target.value)}
            className="w-full px-3 py-2 bg-slate-800/50 border border-slate-700 rounded-lg text-hl-text text-sm font-mono focus:border-hl-accent focus:outline-none"
            placeholder="238.50"
          />
        </div>
      )}

      {/* Stop price (for stop orders) */}
      {orderType === 'stop' && (
        <div className="mb-4">
          <label className="block text-xs text-hl-muted mb-2">Stop Price</label>
          <input
            type="number"
            step="0.01"
            value={stopPrice}
            onChange={(e) => setStopPrice(e.target.value)}
            className="w-full px-3 py-2 bg-slate-800/50 border border-slate-700 rounded-lg text-hl-text text-sm font-mono focus:border-hl-accent focus:outline-none"
            placeholder="235.00"
          />
        </div>
      )}

      {/* Submit button */}
      <button
        onClick={handleSubmit}
        className={`w-full py-3 rounded-lg font-semibold transition-all ${
          side === 'buy'
            ? 'neon-btn neon-btn--buy shadow-neon-cyan'
            : 'neon-btn neon-btn--sell shadow-neon-rose'
        }`}
      >
        {side.toUpperCase()} {quantity} AAPL
      </button>

      {/* Quick quantity buttons */}
      <div className="grid grid-cols-4 gap-1 mt-3">
        {['1', '5', '10', '25'].map((qty) => (
          <button
            key={qty}
            onClick={() => setQuantity(qty)}
            className="px-2 py-1 text-xs bg-slate-800/30 border border-slate-700 rounded text-hl-muted hover:bg-slate-700/50 hover:text-hl-text transition-colors"
          >
            {qty}
          </button>
        ))}
      </div>
    </div>
  );
}
