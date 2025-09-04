#!/usr/bin/env bash
set -euo pipefail
URL=${1:-http://localhost:8001/health}
for i in {1..30}; do
  if curl -sf "$URL" >/dev/null; then
    echo "✅ Market Data Service is running!"
    exit 0
  fi
  echo "…waiting ($i/30)"
  sleep 2
done
echo "❌ Market Data Service did not become healthy in time"
exit 1
