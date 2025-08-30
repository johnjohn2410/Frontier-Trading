#pragma once
#include <string>
#include <memory>
#include <nlohmann/json.hpp>
#include "frontier/engine.hpp"

namespace frontier {

class RpcServer {
public:
    explicit RpcServer(std::shared_ptr<TradingEngine> engine);
    
    // Main RPC handler
    std::string handle_request(const std::string& request);
    
    // Individual RPC methods
    nlohmann::json place_market_order(const nlohmann::json& params);
    nlohmann::json place_limit_order(const nlohmann::json& params);
    nlohmann::json cancel_order(const nlohmann::json& params);
    nlohmann::json mark_to_market(const nlohmann::json& params);
    nlohmann::json get_positions(const nlohmann::json& params);
    nlohmann::json get_account(const nlohmann::json& params);
    nlohmann::json simulate_order(const nlohmann::json& params);
    nlohmann::json check_risk_limits(const nlohmann::json& params);

private:
    std::shared_ptr<TradingEngine> engine_;
    std::shared_ptr<spdlog::logger> logger_;
    
    // Helper methods
    nlohmann::json create_error_response(int code, const std::string& message, const std::string& id = "");
    nlohmann::json create_success_response(const nlohmann::json& result, const std::string& id = "");
    std::string serialize_response(const nlohmann::json& response);
    
    // Validation helpers
    bool validate_order_params(const nlohmann::json& params);
    bool validate_market_data(const nlohmann::json& params);
};

// RPC Request/Response structures
struct RpcRequest {
    std::string jsonrpc = "2.0";
    std::string method;
    nlohmann::json params;
    std::string id;
};

struct RpcResponse {
    std::string jsonrpc = "2.0";
    nlohmann::json result;
    nlohmann::json error;
    std::string id;
};

// Error codes
enum class RpcErrorCode {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    
    // Custom error codes
    OrderRejected = -32001,
    RiskLimitExceeded = -32002,
    InsufficientFunds = -32003,
    InvalidSymbol = -32004,
    MarketClosed = -32005,
    PositionNotFound = -32006,
};

} // namespace frontier
