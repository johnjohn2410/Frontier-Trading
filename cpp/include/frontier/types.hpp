#pragma once

#include <string>
#include <map>
#include <vector>
#include <memory>
// spdlog included only in implementation files
// nlohmann/json included only in implementation files

namespace frontier {

enum class Side {
    Buy,
    Sell
};

enum class OrderType {
    Market,
    Limit,
    Stop
};

struct Position {
    std::string symbol;
    double quantity = 0.0;
    double average_price = 0.0;
    double market_price = 0.0;  // Current market price for P&L calculation
    double realized_pnl = 0.0;
    double unrealized_pnl = 0.0;
    
    double market_value() const {
        return quantity * market_price;
    }
    
    double total_pnl() const {
        return realized_pnl + unrealized_pnl;
    }
};

struct Account {
    double cash = 100000.0;  // Starting cash
    double equity = 100000.0;
    std::map<std::string, Position> positions;
    
    void update_equity() {
        equity = cash;
        for (const auto& [symbol, pos] : positions) {
            equity += pos.market_value();
        }
    }
};

struct MarketData {
    std::string symbol;
    double bid = 0.0;
    double ask = 0.0;
    double last = 0.0;
    double volume = 0.0;
    
    double mid_price() const {
        return (bid + ask) / 2.0;
    }
    
    double spread() const {
        return ask - bid;
    }
};

// JSON serialization - defined in implementation files

} // namespace frontier
