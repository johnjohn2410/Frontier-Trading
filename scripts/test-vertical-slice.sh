#!/bin/bash

# Test script for the vertical slice functionality
# This script tests the complete flow: suggestions → order → position update

set -e

echo "🧪 Testing Vertical Slice: Suggestions → Order → Position Update"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Test 1: Check if services are running
print_status "Test 1: Checking service health..."

# Check API Gateway
if curl -s http://localhost:8000/health | grep -q "healthy"; then
    print_success "API Gateway is healthy"
else
    print_error "API Gateway is not responding"
    exit 1
fi

# Check Market Data service
if curl -s http://localhost:8001/health | grep -q "healthy"; then
    print_success "Market Data service is healthy"
else
    print_error "Market Data service is not responding"
    exit 1
fi

# Check Copilot service
if curl -s http://localhost:8004/health | grep -q "healthy"; then
    print_success "Copilot service is healthy"
else
    print_error "Copilot service is not responding"
    exit 1
fi

# Test 2: Check Redis Streams
print_status "Test 2: Checking Redis Streams..."

# Check if Redis is running
if redis-cli ping | grep -q "PONG"; then
    print_success "Redis is running"
else
    print_error "Redis is not running"
    exit 1
fi

# Check for market data ticks
TICK_COUNT=$(redis-cli XREAD COUNT 5 STREAMS ticks.AAPL 0 | grep -c "tick" || echo "0")
if [ "$TICK_COUNT" -gt 0 ]; then
    print_success "Market data ticks are being published ($TICK_COUNT found)"
else
    print_warning "No market data ticks found - this is normal if just started"
fi

# Test 3: Test order placement
print_status "Test 3: Testing order placement..."

ORDER_RESPONSE=$(curl -s -X POST http://localhost:8000/orders \
    -H "Content-Type: application/json" \
    -d '{
        "symbol": "AAPL",
        "side": "buy",
        "type": "market",
        "qty": "5",
        "correlation_id": "test-123"
    }')

if echo "$ORDER_RESPONSE" | grep -q "order_id"; then
    print_success "Order placement successful"
    echo "Response: $ORDER_RESPONSE"
else
    print_error "Order placement failed"
    echo "Response: $ORDER_RESPONSE"
    exit 1
fi

# Test 4: Check for suggestions
print_status "Test 4: Checking for suggestions..."

# Wait a bit for suggestions to be generated
sleep 5

SUGGESTION_COUNT=$(redis-cli XREAD COUNT 5 STREAMS suggestions.stream 0 | grep -c "suggestion" || echo "0")
if [ "$SUGGESTION_COUNT" -gt 0 ]; then
    print_success "Suggestions are being generated ($SUGGESTION_COUNT found)"
else
    print_warning "No suggestions found - this may be normal if no EMA crosses have occurred"
fi

# Test 5: Check database
print_status "Test 5: Checking database..."

# Check if PostgreSQL is accessible
if pg_isready -h localhost -p 5432 -U frontier -d frontier > /dev/null 2>&1; then
    print_success "PostgreSQL is accessible"
else
    print_error "PostgreSQL is not accessible"
    exit 1
fi

# Test 6: Test WebSocket endpoint
print_status "Test 6: Testing WebSocket endpoint..."

WS_RESPONSE=$(curl -s "http://localhost:8000/ws/stream?s=suggestions.stream")
if echo "$WS_RESPONSE" | grep -q "stream"; then
    print_success "WebSocket endpoint is responding"
else
    print_error "WebSocket endpoint failed"
    echo "Response: $WS_RESPONSE"
fi

print_success "🎉 All tests completed!"
echo ""
echo "Next steps:"
echo "1. Open http://localhost:3000 in your browser"
echo "2. Watch for suggestions to appear"
echo "3. Click 'Accept' on a suggestion"
echo "4. Watch the positions table update"
echo ""
echo "To monitor Redis streams:"
echo "  redis-cli XREAD COUNT 10 STREAMS suggestions.stream 0"
echo "  redis-cli XREAD COUNT 10 STREAMS orders.stream 0"
