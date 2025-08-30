#pragma once

#include <string>
#include <chrono>
#include <variant>
#include <optional>
#include <nlohmann/json.hpp>

namespace trading {

// Asset types
enum class AssetType {
    STOCK,
    ETF,
    CRYPTO,
    FOREX,
    FUTURES,
    OPTIONS
};

// Order types
enum class OrderType {
    MARKET,
    LIMIT,
    STOP,
    STOP_LIMIT,
    TRAILING_STOP
};

// Order side
enum class OrderSide {
    BUY,
    SELL
};

// Order status
enum class OrderStatus {
    PENDING,
    PARTIAL,
    FILLED,
    CANCELLED,
    REJECTED,
    EXPIRED
};

// Time in force
enum class TimeInForce {
    DAY,
    GTC,  // Good Till Cancelled
    IOC,  // Immediate or Cancel
    FOK   // Fill or Kill
};

// Price representation with precision
struct Price {
    double value;
    int precision;
    
    Price(double v = 0.0, int p = 2) : value(v), precision(p) {}
    
    bool operator==(const Price& other) const {
        return std::abs(value - other.value) < std::pow(10, -precision);
    }
    
    bool operator<(const Price& other) const {
        return value < other.value;
    }
    
    std::string toString() const;
};

// Quantity representation
struct Quantity {
    double value;
    int precision;
    
    Quantity(double v = 0.0, int p = 8) : value(v), precision(p) {}
    
    bool operator==(const Quantity& other) const {
        return std::abs(value - other.value) < std::pow(10, -precision);
    }
    
    std::string toString() const;
};

// Asset identifier
struct Asset {
    std::string symbol;
    std::string exchange;
    AssetType type;
    std::string name;
    std::string currency;
    Price tickSize;
    Quantity lotSize;
    
    Asset(const std::string& sym = "", 
          const std::string& ex = "",
          AssetType t = AssetType::STOCK)
        : symbol(sym), exchange(ex), type(t) {}
};

// Market data tick
struct MarketTick {
    Asset asset;
    Price bid;
    Price ask;
    Price last;
    Quantity bidSize;
    Quantity askSize;
    Quantity volume;
    std::chrono::system_clock::time_point timestamp;
    
    Price spread() const { return ask - bid; }
    Price mid() const { return (bid + ask) / 2.0; }
};

// Order representation
struct Order {
    std::string id;
    Asset asset;
    OrderType type;
    OrderSide side;
    Quantity quantity;
    std::optional<Price> limitPrice;
    std::optional<Price> stopPrice;
    TimeInForce timeInForce;
    OrderStatus status;
    std::chrono::system_clock::time_point timestamp;
    std::string clientOrderId;
    
    // Filled quantities and prices
    Quantity filledQuantity;
    Price averageFillPrice;
    
    Order() : type(OrderType::MARKET), side(OrderSide::BUY), 
              timeInForce(TimeInForce::DAY), status(OrderStatus::PENDING) {}
};

// Trade execution
struct Trade {
    std::string id;
    std::string orderId;
    Asset asset;
    OrderSide side;
    Quantity quantity;
    Price price;
    std::chrono::system_clock::time_point timestamp;
    std::string exchange;
    double commission;
    
    Trade() : side(OrderSide::BUY), commission(0.0) {}
};

// Position representation
struct Position {
    Asset asset;
    Quantity quantity;
    Price averagePrice;
    Price currentPrice;
    double unrealizedPnL;
    double realizedPnL;
    std::chrono::system_clock::time_point lastUpdate;
    
    Position() : unrealizedPnL(0.0), realizedPnL(0.0) {}
    
    double marketValue() const {
        return quantity.value * currentPrice.value;
    }
    
    double totalPnL() const {
        return unrealizedPnL + realizedPnL;
    }
};

// Account information
struct Account {
    std::string id;
    std::string name;
    std::string currency;
    double cash;
    double buyingPower;
    double equity;
    double marginUsed;
    double marginAvailable;
    std::chrono::system_clock::time_point lastUpdate;
    
    Account() : cash(0.0), buyingPower(0.0), equity(0.0), 
                marginUsed(0.0), marginAvailable(0.0) {}
};

// Risk limits
struct RiskLimits {
    double maxPositionSize;
    double maxDailyLoss;
    double maxDrawdown;
    double maxLeverage;
    bool allowShortSelling;
    bool allowOptions;
    bool allowFutures;
    
    RiskLimits() : maxPositionSize(100000.0), maxDailyLoss(5000.0),
                   maxDrawdown(0.1), maxLeverage(2.0),
                   allowShortSelling(false), allowOptions(false), allowFutures(false) {}
};

// JSON serialization
NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE(Price, value, precision)
NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE(Quantity, value, precision)
NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE(Asset, symbol, exchange, type, name, currency, tickSize, lotSize)
NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE(Order, id, asset, type, side, quantity, limitPrice, stopPrice, 
                                   timeInForce, status, timestamp, clientOrderId, filledQuantity, averageFillPrice)
NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE(Trade, id, orderId, asset, side, quantity, price, timestamp, exchange, commission)
NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE(Position, asset, quantity, averagePrice, currentPrice, unrealizedPnL, realizedPnL, lastUpdate)
NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE(Account, id, name, currency, cash, buyingPower, equity, marginUsed, marginAvailable, lastUpdate)
NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE(RiskLimits, maxPositionSize, maxDailyLoss, maxDrawdown, maxLeverage, 
                                   allowShortSelling, allowOptions, allowFutures)

} // namespace trading
