#!/bin/bash

# Test Alpaca API Integration
# This script tests the Alpaca API integration with REST and WebSocket endpoints

set -e

echo "🔌 Testing Alpaca API Integration"
echo "================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if Alpaca API keys are configured
check_alpaca_config() {
    echo -e "\n${BLUE}Checking Alpaca Configuration${NC}"
    echo "----------------------------"
    
    if [ -z "$APCA_API_KEY_ID" ] || [ -z "$APCA_API_SECRET_KEY" ]; then
        echo -e "${RED}❌ Alpaca API keys not configured${NC}"
        echo "Please set the following environment variables:"
        echo "  export APCA_API_KEY_ID=your-api-key"
        echo "  export APCA_API_SECRET_KEY=your-secret-key"
        echo ""
        echo "Or create a .env file with:"
        echo "  APCA_API_KEY_ID=your-api-key"
        echo "  APCA_API_SECRET_KEY=your-secret-key"
        return 1
    else
        echo -e "${GREEN}✅ Alpaca API keys configured${NC}"
        echo "API Key: ${APCA_API_KEY_ID:0:8}..."
        echo "Secret: ${APCA_API_SECRET_KEY:0:8}..."
        return 0
    fi
}

# Test Alpaca REST API
test_alpaca_rest() {
    echo -e "\n${BLUE}Testing Alpaca REST API${NC}"
    echo "----------------------"
    
    echo "📊 Testing latest bars endpoint..."
    
    response=$(curl -s -w "\n%{http_code}" \
        "https://data.alpaca.markets/v2/stocks/bars/latest?symbols=AAPL" \
        -H "APCA-API-KEY-ID: $APCA_API_KEY_ID" \
        -H "APCA-API-SECRET-KEY: $APCA_API_SECRET_KEY")
    
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "200" ]; then
        echo -e "${GREEN}✅ REST API call successful${NC}"
        echo "Response: $body" | jq '.' 2>/dev/null || echo "Response: $body"
    else
        echo -e "${RED}❌ REST API call failed${NC}"
        echo "HTTP Code: $http_code"
        echo "Response: $body"
        return 1
    fi
}

# Test Alpaca WebSocket connection
test_alpaca_websocket() {
    echo -e "\n${BLUE}Testing Alpaca WebSocket${NC}"
    echo "------------------------"
    
    echo "🔌 Testing WebSocket connection..."
    
    # Create a simple WebSocket test script
    cat > /tmp/alpaca_ws_test.js << 'EOF'
const WebSocket = require('ws');

const ws = new WebSocket('wss://stream.data.alpaca.markets/v2/test');

ws.on('open', function open() {
    console.log('✅ WebSocket connected');
    
    // Send authentication
    const auth = {
        action: 'auth',
        key: process.env.APCA_API_KEY_ID,
        secret: process.env.APCA_API_SECRET_KEY
    };
    ws.send(JSON.stringify(auth));
});

ws.on('message', function message(data) {
    const msg = JSON.parse(data);
    console.log('📨 Received:', msg);
    
    if (msg.T === 'success' && msg.msg === 'authenticated') {
        console.log('✅ Authentication successful');
        
        // Subscribe to test symbol
        const subscribe = {
            action: 'subscribe',
            trades: ['FAKEPACA'],
            quotes: ['FAKEPACA'],
            bars: ['FAKEPACA']
        };
        ws.send(JSON.stringify(subscribe));
        console.log('📡 Subscribed to FAKEPACA');
        
        // Close after 5 seconds
        setTimeout(() => {
            console.log('🔌 Closing WebSocket connection');
            ws.close();
        }, 5000);
    }
});

ws.on('error', function error(err) {
    console.error('❌ WebSocket error:', err.message);
    process.exit(1);
});

ws.on('close', function close() {
    console.log('🔌 WebSocket connection closed');
    process.exit(0);
});
EOF

    if command -v node >/dev/null 2>&1; then
        echo "Running WebSocket test with Node.js..."
        if node /tmp/alpaca_ws_test.js; then
            echo -e "${GREEN}✅ WebSocket test completed${NC}"
        else
            echo -e "${RED}❌ WebSocket test failed${NC}"
            return 1
        fi
    else
        echo -e "${YELLOW}⚠️  Node.js not available, skipping WebSocket test${NC}"
        echo "Install Node.js to test WebSocket connectivity"
    fi
    
    rm -f /tmp/alpaca_ws_test.js
}

# Test market data service integration
test_market_data_integration() {
    echo -e "\n${BLUE}Testing Market Data Service Integration${NC}"
    echo "----------------------------------------"
    
    # Check if market data service is running
    if curl -s "http://localhost:8001/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Market Data service is running${NC}"
        
        # Test market data endpoint
        echo "📊 Testing market data endpoint..."
        response=$(curl -s "http://localhost:8001/quote/AAPL")
        
        if echo "$response" | jq . > /dev/null 2>&1; then
            echo -e "${GREEN}✅ Market data endpoint responding${NC}"
            echo "Response: $response" | jq '.'
        else
            echo -e "${YELLOW}⚠️  Market data endpoint returned non-JSON response${NC}"
            echo "Response: $response"
        fi
    else
        echo -e "${YELLOW}⚠️  Market Data service not running${NC}"
        echo "Start it with: cd rust/market_data && cargo run"
    fi
}

# Test rate limiting
test_rate_limiting() {
    echo -e "\n${BLUE}Testing Rate Limiting${NC}"
    echo "---------------------"
    
    echo "⏱️  Testing rate limits (3 requests per second)..."
    
    for i in {1..5}; do
        echo "Request $i..."
        start_time=$(date +%s%N)
        
        response=$(curl -s -w "\n%{http_code}" \
            "https://data.alpaca.markets/v2/stocks/bars/latest?symbols=AAPL" \
            -H "APCA-API-KEY-ID: $APCA_API_KEY_ID" \
            -H "APCA-API-SECRET-KEY: $APCA_API_SECRET_KEY")
        
        end_time=$(date +%s%N)
        duration=$(( (end_time - start_time) / 1000000 ))
        
        http_code=$(echo "$response" | tail -n1)
        if [ "$http_code" = "200" ]; then
            echo -e "  ${GREEN}✅ Success${NC} (${duration}ms)"
        else
            echo -e "  ${RED}❌ Failed${NC} (HTTP $http_code)"
        fi
        
        if [ $i -lt 5 ]; then
            sleep 1
        fi
    done
}

# Test error handling
test_error_handling() {
    echo -e "\n${BLUE}Testing Error Handling${NC}"
    echo "----------------------"
    
    echo "🚫 Testing invalid API key..."
    
    response=$(curl -s -w "\n%{http_code}" \
        "https://data.alpaca.markets/v2/stocks/bars/latest?symbols=AAPL" \
        -H "APCA-API-KEY-ID: invalid-key" \
        -H "APCA-API-SECRET-KEY: invalid-secret")
    
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "401" ] || [ "$http_code" = "403" ]; then
        echo -e "${GREEN}✅ Error handling working correctly${NC}"
        echo "HTTP Code: $http_code (expected for invalid credentials)"
    else
        echo -e "${YELLOW}⚠️  Unexpected error response${NC}"
        echo "HTTP Code: $http_code"
        echo "Response: $body"
    fi
}

# Main test execution
main() {
    echo "Starting Alpaca API integration tests..."
    echo "Time: $(date)"
    echo ""
    
    # Load environment variables
    if [ -f .env ]; then
        echo "Loading environment variables from .env file..."
        export $(grep -v '^#' .env | xargs)
    fi
    
    # Run all tests
    if check_alpaca_config; then
        test_alpaca_rest
        test_alpaca_websocket
        test_market_data_integration
        test_rate_limiting
        test_error_handling
        
        echo -e "\n${GREEN}🎉 Alpaca integration tests completed!${NC}"
        echo ""
        echo "Next steps:"
        echo "1. Configure your .env file with Alpaca API keys"
        echo "2. Start the market data service: cd rust/market_data && cargo run"
        echo "3. Monitor the logs for real-time data streaming"
        echo "4. Test the frontend at http://localhost:5173"
    else
        echo -e "\n${RED}❌ Alpaca integration tests failed - configure API keys first${NC}"
        exit 1
    fi
}

# Run main function
main "$@"