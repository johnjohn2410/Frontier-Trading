#include "frontier/rpc.hpp"
#include <spdlog/spdlog.h>
#include <sstream>

namespace frontier {

RpcServer::RpcServer(std::shared_ptr<TradingEngine> engine) 
    : engine_(engine) {
    logger_ = spdlog::stdout_color_mt("rpc_server");
    logger_->set_level(spdlog::level::info);
    logger_->info("RPC Server initialized");
}

std::string RpcServer::handle_request(const std::string& request) {
    try {
        auto json = nlohmann::json::parse(request);
        
        // Validate JSON-RPC 2.0 structure
        if (!json.contains("jsonrpc") || json["jsonrpc"] != "2.0") {
            return serialize_response(create_error_response(
                static_cast<int>(RpcErrorCode::InvalidRequest),
                "Invalid JSON-RPC 2.0 request"
            ));
        }
        
        if (!json.contains("method")) {
            return serialize_response(create_error_response(
                static_cast<int>(RpcErrorCode::InvalidRequest),
                "Missing 'method' field"
            ));
        }
        
        std::string method = json["method"];
        nlohmann::json params = json.value("params", nlohmann::json::object());
        std::string id = json.value("id", "");
        
        logger_->debug("RPC call: {} with params: {}", method, params.dump());
        
        // Route to appropriate method
        nlohmann::json result;
        if (method == "place_market_order") {
            result = place_market_order(params);
        } else if (method == "place_limit_order") {
            result = place_limit_order(params);
        } else if (method == "cancel_order") {
            result = cancel_order(params);
        } else if (method == "mark_to_market") {
            result = mark_to_market(params);
        } else if (method == "get_positions") {
            result = get_positions(params);
        } else if (method == "get_account") {
            result = get_account(params);
        } else if (method == "simulate_order") {
            result = simulate_order(params);
        } else if (method == "check_risk_limits") {
            result = check_risk_limits(params);
        } else {
            return serialize_response(create_error_response(
                static_cast<int>(RpcErrorCode::MethodNotFound),
                "Method not found: " + method,
                id
            ));
        }
        
        return serialize_response(create_success_response(result, id));
        
    } catch (const nlohmann::json::parse_error& e) {
        logger_->error("JSON parse error: {}", e.what());
        return serialize_response(create_error_response(
            static_cast<int>(RpcErrorCode::ParseError),
            "Parse error: " + std::string(e.what())
        ));
    } catch (const std::exception& e) {
        logger_->error("RPC error: {}", e.what());
        return serialize_response(create_error_response(
            static_cast<int>(RpcErrorCode::InternalError),
            "Internal error: " + std::string(e.what())
        ));
    }
}

nlohmann::json RpcServer::place_market_order(const nlohmann::json& params) {
    if (!validate_order_params(params)) {
        throw std::runtime_error("Invalid order parameters");
    }
    
    std::string symbol = params["symbol"];
    std::string side_str = params["side"];
    std::string qty_str = params["qty"];
    std::string price_str = params.value("price", "0");
    
    // Convert strings to doubles for engine (engine uses doubles internally)
    double quantity = std::stod(qty_str);
    double price = std::stod(price_str);
    
    Side side = (side_str == "buy") ? Side::Buy : Side::Sell;
    
    std::string error;
    bool success = engine_->place_market(symbol, side, quantity, price, &error);
    
    if (!success) {
        // Map engine errors to appropriate HTTP status codes
        if (error.find("insufficient") != std::string::npos || 
            error.find("buying power") != std::string::npos) {
            throw std::runtime_error("INSUFFICIENT_BUYING_POWER: " + error);
        } else if (error.find("risk") != std::string::npos || 
                   error.find("limit") != std::string::npos) {
            throw std::runtime_error("RISK_LIMIT: " + error);
        } else if (error.find("market closed") != std::string::npos) {
            throw std::runtime_error("MARKET_CLOSED: " + error);
        } else if (error.find("invalid symbol") != std::string::npos) {
            throw std::runtime_error("INVALID_SYMBOL: " + error);
        } else {
            throw std::runtime_error("ORDER_REJECTED: " + error);
        }
    }
    
    // Return fixed-point strings for precision
    return nlohmann::json{
        {"status", "filled"},
        {"symbol", symbol},
        {"side", side_str},
        {"filled_qty", qty_str},
        {"filled_price", price_str},
        {"order_id", params.value("id", "")},
        {"correlation_id", params.value("correlation_id", "")}
    };
}

nlohmann::json RpcServer::place_limit_order(const nlohmann::json& params) {
    // TODO: Implement limit order logic
    return nlohmann::json{
        {"success", false},
        {"message", "Limit orders not yet implemented"}
    };
}

nlohmann::json RpcServer::cancel_order(const nlohmann::json& params) {
    // TODO: Implement order cancellation
    return nlohmann::json{
        {"success", false},
        {"message", "Order cancellation not yet implemented"}
    };
}

nlohmann::json RpcServer::mark_to_market(const nlohmann::json& params) {
    if (!validate_market_data(params)) {
        throw std::runtime_error("Invalid market data");
    }
    
    std::map<std::string, double> prices;
    for (const auto& [symbol, price] : params["prices"].items()) {
        prices[symbol] = price;
    }
    
    engine_->mark_to_market(prices);
    
    return nlohmann::json{
        {"success", true},
        {"message", "Mark to market completed"},
        {"symbols_updated", prices.size()}
    };
}

nlohmann::json RpcServer::get_positions(const nlohmann::json& params) {
    const auto& positions = engine_->get_positions();
    nlohmann::json result = nlohmann::json::array();
    
    for (const auto& [symbol, position] : positions) {
        result.push_back(nlohmann::json{
            {"symbol", symbol},
            {"quantity", position.quantity},
            {"average_price", position.average_price},
            {"market_price", position.market_price},
            {"realized_pnl", position.realized_pnl},
            {"unrealized_pnl", position.unrealized_pnl},
            {"market_value", position.market_value()}
        });
    }
    
    return result;
}

nlohmann::json RpcServer::get_account(const nlohmann::json& params) {
    const auto& account = engine_->get_account();
    
    return nlohmann::json{
        {"cash", account.cash},
        {"equity", account.equity},
        {"buying_power", account.buying_power},
        {"positions_count", account.positions.size()}
    };
}

nlohmann::json RpcServer::simulate_order(const nlohmann::json& params) {
    if (!validate_order_params(params)) {
        throw std::runtime_error("Invalid order parameters");
    }
    
    std::string symbol = params["symbol"];
    std::string side_str = params["side"];
    double quantity = params["quantity"];
    double price = params["price"];
    
    Side side = (side_str == "buy") ? Side::Buy : Side::Sell;
    
    // Create a temporary engine for simulation
    auto sim_engine = std::make_shared<TradingEngine>(engine_->get_account().cash);
    
    // Copy current positions
    for (const auto& [sym, pos] : engine_->get_positions()) {
        // TODO: Implement position copying
    }
    
    std::string error;
    bool success = sim_engine->place_market(symbol, side, quantity, price, &error);
    
    if (!success) {
        return nlohmann::json{
            {"success", false},
            {"error", error}
        };
    }
    
    const auto& sim_account = sim_engine->get_account();
    
    return nlohmann::json{
        {"success", true},
        {"estimated_cost", quantity * price},
        {"new_cash", sim_account.cash},
        {"new_equity", sim_account.equity},
        {"cash_impact", sim_account.cash - engine_->get_account().cash}
    };
}

nlohmann::json RpcServer::check_risk_limits(const nlohmann::json& params) {
    std::string symbol = params.value("symbol", "");
    
    // TODO: Implement comprehensive risk checks
    bool within_limits = true;
    std::vector<std::string> violations;
    
    const auto& account = engine_->get_account();
    
    // Check buying power
    if (account.cash < 0) {
        within_limits = false;
        violations.push_back("Insufficient cash");
    }
    
    // Check position limits
    if (!symbol.empty()) {
        const auto& positions = engine_->get_positions();
        auto it = positions.find(symbol);
        if (it != positions.end() && it->second.quantity > 1000) {
            within_limits = false;
            violations.push_back("Position size exceeds limit");
        }
    }
    
    return nlohmann::json{
        {"within_limits", within_limits},
        {"violations", violations},
        {"cash", account.cash},
        {"equity", account.equity}
    };
}

nlohmann::json RpcServer::create_error_response(int code, const std::string& message, const std::string& id) {
    nlohmann::json response;
    response["jsonrpc"] = "2.0";
    response["error"] = {
        {"code", code},
        {"message", message}
    };
    if (!id.empty()) {
        response["id"] = id;
    }
    return response;
}

nlohmann::json RpcServer::create_success_response(const nlohmann::json& result, const std::string& id) {
    nlohmann::json response;
    response["jsonrpc"] = "2.0";
    response["result"] = result;
    if (!id.empty()) {
        response["id"] = id;
    }
    return response;
}

std::string RpcServer::serialize_response(const nlohmann::json& response) {
    return response.dump();
}

bool RpcServer::validate_order_params(const nlohmann::json& params) {
    if (!params.contains("symbol") || !params.contains("side") || !params.contains("qty")) {
        return false;
    }
    
    // Validate symbol
    if (!params["symbol"].is_string() || params["symbol"].get<std::string>().empty()) {
        return false;
    }
    
    // Validate side
    std::string side = params["side"].get<std::string>();
    if (side != "buy" && side != "sell") {
        return false;
    }
    
    // Validate quantity (must be string for fixed-point precision)
    if (!params["qty"].is_string()) {
        return false;
    }
    
    try {
        double qty = std::stod(params["qty"].get<std::string>());
        if (qty <= 0) {
            return false;
        }
    } catch (const std::exception&) {
        return false;
    }
    
    // Price is optional for market orders
    if (params.contains("price")) {
        if (!params["price"].is_string()) {
            return false;
        }
        try {
            double price = std::stod(params["price"].get<std::string>());
            if (price < 0) {
                return false;
            }
        } catch (const std::exception&) {
            return false;
        }
    }
    
    return true;
}

bool RpcServer::validate_market_data(const nlohmann::json& params) {
    return params.contains("prices") && 
           params["prices"].is_object();
}

} // namespace frontier
