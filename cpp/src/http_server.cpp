#include "frontier/rpc.hpp"
#include <httplib.h>
#include <spdlog/spdlog.h>
#include <nlohmann/json.hpp>
#include <memory>
#include <string>

namespace frontier {

class HttpServer {
private:
    std::unique_ptr<httplib::Server> server_;
    std::shared_ptr<RpcServer> rpc_server_;
    std::shared_ptr<spdlog::logger> logger_;

public:
    HttpServer(std::shared_ptr<TradingEngine> engine) {
        server_ = std::make_unique<httplib::Server>();
        rpc_server_ = std::make_shared<RpcServer>(engine);
        logger_ = spdlog::stdout_color_mt("http_server");
        logger_->set_level(spdlog::level::info);
        
        setup_routes();
    }

    void setup_routes() {
        // Health check endpoint
        server_->Get("/health", [this](const httplib::Request&, httplib::Response& res) {
            nlohmann::json health = {
                {"status", "healthy"},
                {"service", "cpp-trading-engine"},
                {"timestamp", std::chrono::duration_cast<std::chrono::milliseconds>(
                    std::chrono::system_clock::now().time_since_epoch()).count()}
            };
            res.set_content(health.dump(), "application/json");
        });

        // JSON-RPC endpoint
        server_->Post("/jsonrpc", [this](const httplib::Request& req, httplib::Response& res) {
            try {
                logger_->debug("Received JSON-RPC request: {}", req.body);
                
                // Handle the JSON-RPC request
                std::string response = rpc_server_->handle_request(req.body);
                
                res.set_content(response, "application/json");
                logger_->debug("JSON-RPC response: {}", response);
                
            } catch (const std::exception& e) {
                logger_->error("Error handling JSON-RPC request: {}", e.what());
                
                nlohmann::json error_response = {
                    {"jsonrpc", "2.0"},
                    {"error", {
                        {"code", -32603},
                        {"message", "Internal error: " + std::string(e.what())}
                    }},
                    {"id", nullptr}
                };
                
                res.status = 500;
                res.set_content(error_response.dump(), "application/json");
            }
        });

        // Ping endpoint for health checks
        server_->Get("/ping", [](const httplib::Request&, httplib::Response& res) {
            res.set_content("pong", "text/plain");
        });

        // Get account info
        server_->Get("/account", [this](const httplib::Request&, httplib::Response& res) {
            try {
                nlohmann::json params = {};
                auto result = rpc_server_->get_account(params);
                res.set_content(result.dump(), "application/json");
            } catch (const std::exception& e) {
                logger_->error("Error getting account: {}", e.what());
                res.status = 500;
                res.set_content("{\"error\": \"Internal server error\"}", "application/json");
            }
        });

        // Get positions
        server_->Get("/positions", [this](const httplib::Request&, httplib::Response& res) {
            try {
                nlohmann::json params = {};
                auto result = rpc_server_->get_positions(params);
                res.set_content(result.dump(), "application/json");
            } catch (const std::exception& e) {
                logger_->error("Error getting positions: {}", e.what());
                res.status = 500;
                res.set_content("{\"error\": \"Internal server error\"}", "application/json");
            }
        });

        // Error handler
        server_->set_exception_handler([](const auto& req, auto& res, std::exception_ptr ep) {
            try {
                std::rethrow_exception(ep);
            } catch (const std::exception& e) {
                nlohmann::json error = {
                    {"error", "Internal server error"},
                    {"message", e.what()}
                };
                res.status = 500;
                res.set_content(error.dump(), "application/json");
            }
        });
    }

    bool start(int port = 8003) {
        logger_->info("Starting HTTP server on port {}", port);
        
        if (!server_->listen("0.0.0.0", port)) {
            logger_->error("Failed to start HTTP server on port {}", port);
            return false;
        }
        
        logger_->info("HTTP server started successfully on port {}", port);
        return true;
    }

    void stop() {
        logger_->info("Stopping HTTP server");
        server_->stop();
    }
};

} // namespace frontier
