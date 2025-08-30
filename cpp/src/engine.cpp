#include "frontier/engine.hpp"
#include <spdlog/sinks/stdout_color_sinks.h>
#include <iostream>
#include <iomanip>

namespace frontier {

TradingEngine::TradingEngine() {
    // Initialize logger with unique name
    static int instance_count = 0;
    std::string logger_name = "trading_engine_" + std::to_string(instance_count++);
    logger_ = spdlog::stdout_color_mt(logger_name);
    logger_->set_level(spdlog::level::info);
    
    logger_->info("Frontier Trading Engine initialized (paper mode)");
}

bool TradingEngine::place_market_order(const std::string& symbol, Side side, double quantity, double price) {
    if (!check_risk_limits(symbol, side, quantity, price)) {
        logger_->warn("Risk limit check failed for {} order", symbol);
        return false;
    }
    
    update_position(symbol, side, quantity, price);
    calculate_unrealized_pnl();
    account_.update_equity();
    
    logger_->info("Market order executed: {} {} {} shares at ${:.2f}", 
                  side == Side::Buy ? "BUY" : "SELL", quantity, symbol, price);
    
    return true;
}

void TradingEngine::mark_to_market(const std::map<std::string, double>& prices) {
    for (const auto& [symbol, price] : prices) {
        auto it = account_.positions.find(symbol);
        if (it != account_.positions.end()) {
            // Update the market price for P&L calculation
            it->second.market_price = price;
        }
    }
    
    calculate_unrealized_pnl();
    account_.update_equity();
    
    logger_->info("Mark to market completed for {} symbols", prices.size());
}

const Position* TradingEngine::get_position(const std::string& symbol) const {
    auto it = account_.positions.find(symbol);
    return it != account_.positions.end() ? &it->second : nullptr;
}

void TradingEngine::update_market_data(const MarketData& data) {
    market_data_[data.symbol] = data;
}

const MarketData* TradingEngine::get_market_data(const std::string& symbol) const {
    auto it = market_data_.find(symbol);
    return it != market_data_.end() ? &it->second : nullptr;
}

bool TradingEngine::check_risk_limits(const std::string& symbol, Side side, double quantity, double price) const {
    (void)symbol; // Suppress unused parameter warning
    double order_value = quantity * price;
    
    // Check if we have enough cash for buy orders
    if (side == Side::Buy && order_value > account_.cash) {
        logger_->warn("Insufficient cash for buy order: need ${:.2f}, have ${:.2f}", 
                      order_value, account_.cash);
        return false;
    }
    
    // Check position size limits (max 20% of account in single position)
    double max_position_value = account_.equity * 0.2;
    if (order_value > max_position_value) {
        logger_->warn("Order value ${:.2f} exceeds position limit ${:.2f}", 
                      order_value, max_position_value);
        return false;
    }
    
    return true;
}

void TradingEngine::print_account_summary() const {
    std::cout << "\n=== Account Summary ===" << std::endl;
    std::cout << std::fixed << std::setprecision(2);
    std::cout << "Cash: $" << account_.cash << std::endl;
    std::cout << "Equity: $" << account_.equity << std::endl;
    std::cout << "Total P&L: $" << (account_.equity - 100000.0) << std::endl;
    std::cout << "=====================" << std::endl;
}

void TradingEngine::print_positions() const {
    if (account_.positions.empty()) {
        std::cout << "No open positions" << std::endl;
        return;
    }
    
    std::cout << "\n=== Open Positions ===" << std::endl;
    std::cout << std::fixed << std::setprecision(2);
    std::cout << std::setw(8) << "Symbol" 
              << std::setw(12) << "Quantity" 
              << std::setw(12) << "Avg Price" 
              << std::setw(12) << "Market Value" 
              << std::setw(12) << "Realized P&L" 
              << std::setw(12) << "Unrealized P&L" << std::endl;
    std::cout << std::string(80, '-') << std::endl;
    
    for (const auto& [symbol, pos] : account_.positions) {
        std::cout << std::setw(8) << symbol
                  << std::setw(12) << pos.quantity
                  << std::setw(12) << pos.average_price
                  << std::setw(12) << pos.market_value()
                  << std::setw(12) << pos.realized_pnl
                  << std::setw(12) << pos.unrealized_pnl << std::endl;
    }
    std::cout << "=====================" << std::endl;
}

void TradingEngine::update_position(const std::string& symbol, Side side, double quantity, double price) {
    auto& position = account_.positions[symbol];
    position.symbol = symbol;
    
    if (side == Side::Buy) {
        // Calculate new average price using weighted average
        double total_cost = position.quantity * position.average_price + quantity * price;
        double total_quantity = position.quantity + quantity;
        
        if (total_quantity > 0) {
            position.average_price = total_cost / total_quantity;
        }
        position.quantity += quantity;
        account_.cash -= quantity * price;
    } else {  // Sell
        if (position.quantity < quantity) {
            logger_->error("Insufficient shares to sell: have {}, trying to sell {}", 
                          position.quantity, quantity);
            return;
        }
        
        // Calculate realized P&L
        double realized_pnl = (price - position.average_price) * quantity;
        position.realized_pnl += realized_pnl;
        
        position.quantity -= quantity;
        account_.cash += quantity * price;
        
        // Remove position if quantity becomes zero
        if (position.quantity == 0) {
            account_.positions.erase(symbol);
        }
    }
}

void TradingEngine::calculate_unrealized_pnl() {
    for (auto& [symbol, position] : account_.positions) {
        if (position.market_price > 0) {
            double cost_basis = position.quantity * position.average_price;
            double market_value = position.quantity * position.market_price;
            position.unrealized_pnl = market_value - cost_basis;
        }
    }
}

} // namespace frontier
