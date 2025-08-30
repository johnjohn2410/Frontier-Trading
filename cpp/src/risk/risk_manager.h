#pragma once

#include "../engine/types.h"
#include <memory>
#include <unordered_map>
#include <vector>
#include <mutex>
#include <atomic>
#include <chrono>
#include <spdlog/spdlog.h>

namespace trading {

// Risk metrics
struct RiskMetrics {
    double totalPnL;
    double dailyPnL;
    double maxDrawdown;
    double currentDrawdown;
    double portfolioValue;
    double marginUsed;
    double marginAvailable;
    double leverage;
    double beta;
    double sharpeRatio;
    double volatility;
    
    RiskMetrics() : totalPnL(0.0), dailyPnL(0.0), maxDrawdown(0.0), 
                    currentDrawdown(0.0), portfolioValue(0.0), marginUsed(0.0),
                    marginAvailable(0.0), leverage(1.0), beta(1.0), 
                    sharpeRatio(0.0), volatility(0.0) {}
};

// Position risk
struct PositionRisk {
    std::string symbol;
    double marketValue;
    double unrealizedPnL;
    double realizedPnL;
    double exposure;
    double concentration;
    double var95;  // Value at Risk (95% confidence)
    double maxLoss;
    
    PositionRisk() : marketValue(0.0), unrealizedPnL(0.0), realizedPnL(0.0),
                     exposure(0.0), concentration(0.0), var95(0.0), maxLoss(0.0) {}
};

// Risk violation
struct RiskViolation {
    enum class Type {
        POSITION_SIZE,
        DAILY_LOSS,
        DRAWDOWN,
        LEVERAGE,
        CONCENTRATION,
        MARGIN
    };
    
    Type type;
    std::string message;
    double currentValue;
    double limitValue;
    std::chrono::system_clock::time_point timestamp;
    
    RiskViolation(Type t, const std::string& msg, double current, double limit)
        : type(t), message(msg), currentValue(current), limitValue(limit) {}
};

// Risk manager callbacks
using RiskViolationCallback = std::function<void(const RiskViolation&)>;
using RiskMetricsCallback = std::function<void(const RiskMetrics&)>;

// Main risk manager class
class RiskManager {
private:
    // Risk limits
    RiskLimits limits_;
    
    // Current state
    std::unordered_map<std::string, Position> positions_;
    std::vector<Trade> dailyTrades_;
    std::vector<RiskViolation> violations_;
    
    // Historical data for calculations
    std::vector<double> dailyReturns_;
    std::vector<double> portfolioValues_;
    double peakPortfolioValue_;
    
    // Callbacks
    RiskViolationCallback violationCallback_;
    RiskMetricsCallback metricsCallback_;
    
    // Thread safety
    mutable std::mutex mutex_;
    
    // Logging
    std::shared_ptr<spdlog::logger> logger_;
    
    // Internal methods
    void calculatePositionRisk(PositionRisk& risk, const Position& position);
    void calculatePortfolioRisk(RiskMetrics& metrics);
    void checkPositionLimits(const Order& order, std::vector<RiskViolation>& violations);
    void checkPortfolioLimits(std::vector<RiskViolation>& violations);
    void addViolation(const RiskViolation& violation);
    double calculateVaR(const std::vector<double>& returns, double confidence = 0.95);
    double calculateSharpeRatio(const std::vector<double>& returns);
    double calculateBeta(const std::vector<double>& portfolioReturns, 
                        const std::vector<double>& marketReturns);
    
public:
    explicit RiskManager(const RiskLimits& limits = RiskLimits());
    ~RiskManager() = default;
    
    // Risk limit management
    void setRiskLimits(const RiskLimits& limits);
    RiskLimits getRiskLimits() const;
    
    // Position management
    void updatePosition(const Position& position);
    void removePosition(const std::string& symbol);
    std::vector<Position> getPositions() const;
    Position getPosition(const std::string& symbol) const;
    
    // Trade processing
    void processTrade(const Trade& trade);
    void clearDailyTrades();
    
    // Risk checking
    bool checkOrderRisk(const Order& order);
    std::vector<RiskViolation> getRiskViolations() const;
    void clearViolations();
    
    // Risk metrics
    RiskMetrics getRiskMetrics() const;
    std::vector<PositionRisk> getPositionRisks() const;
    
    // Portfolio analysis
    double getPortfolioValue() const;
    double getTotalPnL() const;
    double getDailyPnL() const;
    double getMaxDrawdown() const;
    double getCurrentDrawdown() const;
    double getLeverage() const;
    
    // Risk calculations
    double calculatePositionSize(const std::string& symbol, double price) const;
    double calculateMaxLoss(const std::string& symbol) const;
    double calculateConcentration(const std::string& symbol) const;
    
    // Callback registration
    void setViolationCallback(RiskViolationCallback callback) { violationCallback_ = callback; }
    void setMetricsCallback(RiskMetricsCallback callback) { metricsCallback_ = callback; }
    
    // Reset and cleanup
    void resetDailyMetrics();
    void resetAllMetrics();
    
    // Reporting
    std::string generateRiskReport() const;
    nlohmann::json exportRiskData() const;
};

// Risk calculator utilities
class RiskCalculator {
public:
    // Value at Risk calculations
    static double calculateHistoricalVaR(const std::vector<double>& returns, double confidence = 0.95);
    static double calculateParametricVaR(double mean, double stdDev, double confidence = 0.95);
    static double calculateMonteCarloVaR(const std::vector<double>& returns, int simulations = 10000);
    
    // Volatility calculations
    static double calculateVolatility(const std::vector<double>& returns);
    static double calculateExponentialVolatility(const std::vector<double>& returns, double lambda = 0.94);
    
    // Correlation and beta
    static double calculateCorrelation(const std::vector<double>& x, const std::vector<double>& y);
    static double calculateBeta(const std::vector<double>& assetReturns, const std::vector<double>& marketReturns);
    
    // Drawdown calculations
    static double calculateMaxDrawdown(const std::vector<double>& values);
    static double calculateCurrentDrawdown(const std::vector<double>& values);
    
    // Position sizing
    static double calculateKellyCriterion(double winRate, double avgWin, double avgLoss);
    static double calculateOptimalPositionSize(double accountSize, double riskPerTrade, double stopLoss);
};

} // namespace trading
