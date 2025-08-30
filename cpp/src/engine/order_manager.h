#pragma once

#include "types.h"
#include <memory>
#include <unordered_map>
#include <queue>
#include <mutex>
#include <condition_variable>
#include <functional>
#include <atomic>
#include <spdlog/spdlog.h>

namespace trading {

// Order book entry
struct OrderBookEntry {
    Price price;
    Quantity quantity;
    std::string orderId;
    std::chrono::system_clock::time_point timestamp;
    
    OrderBookEntry(const Price& p, const Quantity& q, const std::string& id)
        : price(p), quantity(q), orderId(id) {}
};

// Order book side (bids or asks)
class OrderBookSide {
private:
    std::map<Price, std::vector<OrderBookEntry>, std::greater<Price>> entries; // Bids: highest first
    std::mutex mutex_;
    
public:
    void addOrder(const Order& order);
    void removeOrder(const std::string& orderId);
    void updateOrder(const Order& order);
    std::vector<OrderBookEntry> getTopLevels(size_t levels = 5) const;
    bool isEmpty() const;
    void clear();
};

// Complete order book
class OrderBook {
private:
    OrderBookSide bids;
    OrderBookSide asks;
    std::string symbol;
    mutable std::mutex mutex_;
    
public:
    explicit OrderBook(const std::string& sym) : symbol(sym) {}
    
    void addOrder(const Order& order);
    void removeOrder(const std::string& orderId);
    void updateOrder(const Order& order);
    
    std::pair<std::vector<OrderBookEntry>, std::vector<OrderBookEntry>> 
    getTopLevels(size_t levels = 5) const;
    
    Price getBestBid() const;
    Price getBestAsk() const;
    Price getSpread() const;
    
    std::string getSymbol() const { return symbol; }
};

// Order execution result
struct ExecutionResult {
    bool success;
    std::string message;
    std::vector<Trade> trades;
    Order updatedOrder;
    
    ExecutionResult() : success(false) {}
};

// Order manager callbacks
using OrderCallback = std::function<void(const Order&)>;
using TradeCallback = std::function<void(const Trade&)>;
using ExecutionCallback = std::function<void(const ExecutionResult&)>;

// Main order manager class
class OrderManager {
private:
    std::unordered_map<std::string, OrderBook> orderBooks;
    std::unordered_map<std::string, Order> activeOrders;
    std::unordered_map<std::string, std::vector<Trade>> orderTrades;
    
    // Callbacks
    OrderCallback orderUpdateCallback_;
    TradeCallback tradeCallback_;
    ExecutionCallback executionCallback_;
    
    // Thread safety
    mutable std::mutex orderMutex_;
    mutable std::mutex bookMutex_;
    
    // Order ID generation
    std::atomic<uint64_t> orderIdCounter_{0};
    
    // Logging
    std::shared_ptr<spdlog::logger> logger_;
    
    // Internal methods
    std::string generateOrderId();
    ExecutionResult executeMarketOrder(const Order& order, const MarketTick& tick);
    ExecutionResult executeLimitOrder(const Order& order, const MarketTick& tick);
    void processOrderBookUpdate(const std::string& symbol);
    bool checkOrderValidity(const Order& order) const;
    
public:
    OrderManager();
    ~OrderManager() = default;
    
    // Order management
    std::string submitOrder(const Order& order);
    bool cancelOrder(const std::string& orderId);
    bool modifyOrder(const std::string& orderId, const Order& newOrder);
    
    // Order queries
    Order getOrder(const std::string& orderId) const;
    std::vector<Order> getActiveOrders() const;
    std::vector<Order> getOrdersBySymbol(const std::string& symbol) const;
    std::vector<Trade> getOrderTrades(const std::string& orderId) const;
    
    // Order book queries
    OrderBook getOrderBook(const std::string& symbol) const;
    std::vector<std::string> getSymbols() const;
    
    // Market data processing
    void processMarketTick(const MarketTick& tick);
    
    // Callback registration
    void setOrderUpdateCallback(OrderCallback callback) { orderUpdateCallback_ = callback; }
    void setTradeCallback(TradeCallback callback) { tradeCallback_ = callback; }
    void setExecutionCallback(ExecutionCallback callback) { executionCallback_ = callback; }
    
    // Statistics
    size_t getActiveOrderCount() const;
    size_t getOrderBookCount() const;
    
    // Risk management
    bool checkRiskLimits(const Order& order, const RiskLimits& limits) const;
    
    // Cleanup
    void cancelAllOrders();
    void clearOrderBooks();
};

} // namespace trading
