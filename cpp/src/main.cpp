#include "frontier/engine.hpp"
#include "frontier/rpc.hpp"
#include "http_server.cpp"
#include <iostream>
#include <memory>
#include <signal.h>
#include <spdlog/spdlog.h>

// Forward declaration
namespace frontier {
    class HttpServer;
}

// Global server pointer for signal handling
std::unique_ptr<frontier::HttpServer> g_server;

void signal_handler(int signal) {
    if (g_server) {
        spdlog::info("Received signal {}, shutting down server...", signal);
        g_server->stop();
    }
    exit(0);
}

int main() {
    // Set up signal handling
    signal(SIGINT, signal_handler);
    signal(SIGTERM, signal_handler);
    
    spdlog::info("Starting Frontier Trading Platform - C++ Engine");
    spdlog::info("=============================================");
    
    try {
        // Initialize trading engine with starting cash
        auto engine = std::make_shared<frontier::TradingEngine>(100000.0);
        
        // Set up some initial market data
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
        
        engine->update_market_data(aapl_data);
        engine->update_market_data(googl_data);
        
        // Show initial account state
        spdlog::info("Initial account state:");
        engine->print_account_summary();
        
        // Create and start HTTP server
        g_server = std::make_unique<frontier::HttpServer>(engine);
        
        int port = 8003;
        spdlog::info("Starting HTTP server on port {}", port);
        
        if (!g_server->start(port)) {
            spdlog::error("Failed to start HTTP server");
            return 1;
        }
        
        spdlog::info("C++ Trading Engine HTTP server is running on port {}", port);
        spdlog::info("Health check: http://localhost:{}/health", port);
        spdlog::info("JSON-RPC endpoint: http://localhost:{}/jsonrpc", port);
        spdlog::info("Press Ctrl+C to stop the server");
        
        // Keep the server running
        while (true) {
            std::this_thread::sleep_for(std::chrono::seconds(1));
        }
        
    } catch (const std::exception& e) {
        spdlog::error("Fatal error: {}", e.what());
        return 1;
    }
    
    return 0;
}
