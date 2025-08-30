#include "frontier/engine.hpp"
#include <iostream>
#include <thread>
#include <chrono>

int main() {
    std::cout << "🚀 Frontier Trading Platform - C++ Engine Demo" << std::endl;
    std::cout << "=============================================" << std::endl;
    
    // Initialize trading engine
    frontier::TradingEngine engine;
    
    // Set up some market data
    frontier::MarketData aapl_data;
    aapl_data.symbol = "AAPL";
    aapl_data.bid = 150.00;
    aapl_data.ask = 150.10;
    aapl_data.last = 150.05;
    aapl_data.volume = 1000000;
    
    frontier::MarketData googl_data;
    googl_data.symbol = "GOOGL";
    googl_data.bid = 2800.00;
    googl_data.ask = 2800.50;
    googl_data.last = 2800.25;
    googl_data.volume = 500000;
    
    engine.update_market_data(aapl_data);
    engine.update_market_data(googl_data);
    
    // Show initial account state
    engine.print_account_summary();
    
    // Place some demo trades
    std::cout << "\n📈 Placing demo trades..." << std::endl;
    
    // Buy 100 shares of AAPL at $150.05
    engine.place_market_order("AAPL", frontier::Side::Buy, 100, 150.05);
    
    // Buy 10 shares of GOOGL at $2800.25
    engine.place_market_order("GOOGL", frontier::Side::Buy, 10, 2800.25);
    
    // Show positions after trades
    engine.print_positions();
    engine.print_account_summary();
    
    // Simulate price movement and mark to market
    std::cout << "\n📊 Simulating price movement..." << std::endl;
    
    // AAPL goes up 2%
    double new_aapl_price = 150.05 * 1.02;  // $153.05
    std::cout << "AAPL price moved from $150.05 to $" << new_aapl_price << std::endl;
    
    // GOOGL goes down 1%
    double new_googl_price = 2800.25 * 0.99;  // $2772.25
    std::cout << "GOOGL price moved from $2800.25 to $" << new_googl_price << std::endl;
    
    // Mark to market
    std::map<std::string, double> new_prices = {
        {"AAPL", new_aapl_price},
        {"GOOGL", new_googl_price}
    };
    
    engine.mark_to_market(new_prices);
    
    // Show updated positions
    engine.print_positions();
    engine.print_account_summary();
    
    // Sell some AAPL to realize some gains
    std::cout << "\n💰 Selling some AAPL to realize gains..." << std::endl;
    engine.place_market_order("AAPL", frontier::Side::Sell, 50, new_aapl_price);
    
    // Final account summary
    engine.print_positions();
    engine.print_account_summary();
    
    std::cout << "\n✅ Demo completed successfully!" << std::endl;
    std::cout << "The C++ trading engine is working correctly." << std::endl;
    
    return 0;
}
