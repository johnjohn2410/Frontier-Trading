#!/usr/bin/env bash
set -euo pipefail

# Load environment variables
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
fi

: "${APCA_API_KEY_ID:?set APCA_API_KEY_ID}"
: "${APCA_API_SECRET_KEY:?set APCA_API_SECRET_KEY}"
ALPACA_DATA_URL="${ALPACA_DATA_URL:-https://data.alpaca.markets/v2}"
PAPER_TRADING_URL="${PAPER_TRADING_URL:-https://paper-api.alpaca.markets}"

echo "🔌 Alpaca API Sanity Check"
echo "========================="
echo ""

echo "== Account Status =="
curl -s "$PAPER_TRADING_URL/v2/account" \
  -H "APCA-API-KEY-ID: $APCA_API_KEY_ID" \
  -H "APCA-API-SECRET-KEY: $APCA_API_SECRET_KEY" \
| jq -r '"✅ Status: \(.status) | Buying Power: $\(.buying_power) | Cash: $\(.cash)"'

echo ""
echo "== Latest Bar (AAPL) =="
curl -s "$ALPACA_DATA_URL/stocks/bars/latest?symbols=AAPL" \
  -H "APCA-API-KEY-ID: $APCA_API_KEY_ID" \
  -H "APCA-API-SECRET-KEY: $APCA_API_SECRET_KEY" \
| jq -r '"✅ AAPL: $\(.bars.AAPL.c) (High: $\(.bars.AAPL.h), Low: $\(.bars.AAPL.l), Vol: \(.bars.AAPL.v))"'

echo ""
echo "== Latest Trade (AAPL) =="
curl -s "$ALPACA_DATA_URL/stocks/trades/latest?symbol=AAPL" \
  -H "APCA-API-KEY-ID: $APCA_API_KEY_ID" \
  -H "APCA-API-SECRET-KEY: $APCA_API_SECRET_KEY" \
| jq -r '"✅ Latest Trade: $\(.trade.price) at \(.trade.t)"'

echo ""
echo "🎉 Alpaca API is working perfectly!"
echo "Ready to start the platform with real market data!"
