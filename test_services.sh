#!/bin/bash

echo "🚀 Testing Frontier Trading Platform Services"
echo "=============================================="

# Test API Gateway Health
echo "📊 Testing API Gateway Health..."
curl -s http://localhost:8000/health | jq . || echo "❌ API Gateway not responding"

echo ""

# Test Quote Endpoint
echo "📈 Testing Quote Endpoint..."
curl -s "http://localhost:8000/quote?symbol=AAPL" | jq . || echo "❌ Quote endpoint failed"

echo ""

# Test Positions
echo "💼 Testing Positions..."
curl -s http://localhost:8000/positions | jq . || echo "❌ Positions endpoint failed"

echo ""

# Test Suggestions
echo "🤖 Testing Suggestions..."
curl -s http://localhost:8000/suggestions | jq . || echo "❌ Suggestions endpoint failed"

echo ""

# Test Order Placement
echo "📋 Testing Order Placement..."
curl -s -X POST http://localhost:8000/orders \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "AAPL",
    "side": "buy",
    "qty": "5",
    "order_type": "market"
  }' | jq . || echo "❌ Order placement failed"

echo ""

# Test Alpaca API Direct
echo "🔗 Testing Alpaca API Direct..."
if [ -z "$APCA_API_KEY_ID" ] || [ -z "$APCA_API_SECRET_KEY" ]; then
  echo "⚠️  Alpaca API keys not set in environment variables"
  echo "   Set APCA_API_KEY_ID and APCA_API_SECRET_KEY to test Alpaca API"
else
  curl -s -H "APCA-API-KEY-ID: ${APCA_API_KEY_ID}" \
    -H "APCA-API-SECRET-KEY: ${APCA_API_SECRET_KEY}" \
    "https://data.alpaca.markets/v2/stocks/bars/latest?symbols=AAPL" | jq '.bars.AAPL.c' || echo "❌ Alpaca API failed"
fi

echo ""

# Check Redis Streams
echo "📡 Checking Redis Streams..."
docker exec frontier-trading-redis-1 redis-cli KEYS "ticks.*" | wc -l | xargs echo "Streams found:"

echo ""

# Check Frontend
echo "🌐 Testing Frontend..."
curl -s -o /dev/null -w "Frontend Status: %{http_code}\n" http://localhost:5173/

echo ""
echo "✅ Service testing complete!"
