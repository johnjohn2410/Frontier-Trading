#include <gtest/gtest.h>
#include "frontier/engine.hpp"

class TradingEngineTest : public ::testing::Test {
protected:
    void SetUp() override {
        engine_ = std::make_unique<frontier::TradingEngine>();
    }
    
    void TearDown() override {
        engine_.reset();
    }
    
    std::unique_ptr<frontier::TradingEngine> engine_;
};

TEST_F(TradingEngineTest, InitialAccountState) {
    const auto& account = engine_->get_account();
    EXPECT_EQ(account.cash, 100000.0);
    EXPECT_EQ(account.equity, 100000.0);
    EXPECT_TRUE(account.positions.empty());
}

TEST_F(TradingEngineTest, PlaceMarketBuyOrder) {
    // Set up market data
    frontier::MarketData data;
    data.symbol = "AAPL";
    data.last = 150.00;
    engine_->update_market_data(data);
    
    // Place buy order
    bool success = engine_->place_market_order("AAPL", frontier::Side::Buy, 100, 150.00);
    EXPECT_TRUE(success);
    
    const auto& account = engine_->get_account();
    EXPECT_EQ(account.cash, 85000.0);  // 100000 - (100 * 150)
    
    const auto* position = engine_->get_position("AAPL");
    ASSERT_NE(position, nullptr);
    EXPECT_EQ(position->quantity, 100.0);
    EXPECT_EQ(position->average_price, 150.00);
}

TEST_F(TradingEngineTest, WeightedAveragePrice) {
    // Set up market data
    frontier::MarketData data;
    data.symbol = "AAPL";
    data.last = 150.00;
    engine_->update_market_data(data);
    
    // Buy 100 shares at $150
    engine_->place_market_order("AAPL", frontier::Side::Buy, 100, 150.00);
    
    // Buy 50 more shares at $160
    engine_->place_market_order("AAPL", frontier::Side::Buy, 50, 160.00);
    
    const auto* position = engine_->get_position("AAPL");
    ASSERT_NE(position, nullptr);
    EXPECT_EQ(position->quantity, 150.0);
    
    // Weighted average: (100*150 + 50*160) / 150 = 153.33
    EXPECT_NEAR(position->average_price, 153.33, 0.01);
}

TEST_F(TradingEngineTest, RealizedPnL) {
    // Set up market data
    frontier::MarketData data;
    data.symbol = "AAPL";
    data.last = 150.00;
    engine_->update_market_data(data);
    
    // Buy 100 shares at $150
    engine_->place_market_order("AAPL", frontier::Side::Buy, 100, 150.00);
    
    // Sell 50 shares at $160 (profit of $10 per share)
    engine_->place_market_order("AAPL", frontier::Side::Sell, 50, 160.00);
    
    const auto* position = engine_->get_position("AAPL");
    ASSERT_NE(position, nullptr);
    EXPECT_EQ(position->quantity, 50.0);
    EXPECT_EQ(position->realized_pnl, 500.0);  // 50 shares * $10 profit
}

TEST_F(TradingEngineTest, UnrealizedPnL) {
    // Set up market data
    frontier::MarketData data;
    data.symbol = "AAPL";
    data.last = 150.00;
    engine_->update_market_data(data);
    
    // Buy 100 shares at $150
    engine_->place_market_order("AAPL", frontier::Side::Buy, 100, 150.00);
    
    // Update market data to $160
    data.last = 160.00;
    engine_->update_market_data(data);
    
    // Mark to market
    std::map<std::string, double> prices = {{"AAPL", 160.00}};
    engine_->mark_to_market(prices);
    
    const auto* position = engine_->get_position("AAPL");
    ASSERT_NE(position, nullptr);
    EXPECT_EQ(position->unrealized_pnl, 1000.0);  // 100 shares * $10 gain
}

TEST_F(TradingEngineTest, RiskLimits) {
    // Try to buy more than we have cash for
    bool success = engine_->place_market_order("AAPL", frontier::Side::Buy, 1000, 150.00);
    EXPECT_FALSE(success);  // Should fail due to insufficient cash
    
    const auto& account = engine_->get_account();
    EXPECT_EQ(account.cash, 100000.0);  // Cash should be unchanged
}

TEST_F(TradingEngineTest, InsufficientShares) {
    // Set up market data
    frontier::MarketData data;
    data.symbol = "AAPL";
    data.last = 150.00;
    engine_->update_market_data(data);
    
    // Buy 100 shares
    engine_->place_market_order("AAPL", frontier::Side::Buy, 100, 150.00);
    
    // Try to sell more than we have
    bool success = engine_->place_market_order("AAPL", frontier::Side::Sell, 150, 160.00);
    EXPECT_FALSE(success);  // Should fail due to insufficient shares
    
    const auto* position = engine_->get_position("AAPL");
    ASSERT_NE(position, nullptr);
    EXPECT_EQ(position->quantity, 100.0);  // Quantity should be unchanged
}

int main(int argc, char **argv) {
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
