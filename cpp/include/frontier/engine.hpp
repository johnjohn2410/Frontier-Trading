#pragma once

#include "types.hpp"
#include <memory>
#include <spdlog/spdlog.h>

namespace frontier {

class TradingEngine {
public:
    TradingEngine();
    ~TradingEngine() = default;
    
    // Core trading functions
    bool place_market_order(const std::string& symbol, Side side, double quantity, double price);
    void mark_to_market(const std::map<std::string, double>& prices);
    
    // Account management
    const Account& get_account() const { return account_; }
    const Position* get_position(const std::string& symbol) const;
    
    // Market data
    void update_market_data(const MarketData& data);
    const MarketData* get_market_data(const std::string& symbol) const;
    
    // Risk management
    bool check_risk_limits(const std::string& symbol, Side side, double quantity, double price) const;
    
    // Reporting
    void print_account_summary() const;
    void print_positions() const;
    
private:
    Account account_;
    std::map<std::string, MarketData> market_data_;
    std::shared_ptr<spdlog::logger> logger_;
    
    // Internal helper functions
    void update_position(const std::string& symbol, Side side, double quantity, double price);
    void calculate_unrealized_pnl();
};

} // namespace frontier
