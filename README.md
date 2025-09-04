# Frontier Trading Platform

A modern, multi-asset trading platform built with C++, Rust, and React, featuring an AI-powered Copilot for intelligent trading insights and real-time order execution.

## Current Status

### Completed Components

1. **C++ Trading Engine** - Core trading logic with paper trading support
   - Order management and execution via JSON-RPC
   - Position tracking and P&L calculation
   - Risk management and limits
   - Real-time market data processing
   - Comprehensive test suite

2. **Rust Microservices Architecture** - Event-driven backend services
   - **API Gateway**: Single entry point with WebSocket support
   - **Market Data Service**: Multi-provider market data with Redis Streams
   - **Copilot Service**: EMA cross detection and suggestion generation
   - **Notification Service**: Alert management and delivery
   - **Market Events Service**: Real-time event monitoring and alert system
   - **Event Bus**: Redis Streams for inter-service communication

3. **AI Copilot System** - Intelligent trading suggestions
   - Real-time market analysis with EMA band crosses
   - Volume spike detection and technical indicators
   - Structured suggestions with confidence scoring
   - Risk impact analysis and guardrail checks
   - What-if analysis for order simulation

4. **React Frontend** - Real-time trading interface
   - **Dashboard**: Suggestions and positions panels
   - **Suggestions Component**: Accept/Dismiss copilot recommendations
   - **Positions Component**: Real-time position updates
   - **WebSocket Integration**: Live data streaming
   - **Order Placement**: One-click order execution

5. **Market Events Alert System** - Real-time event monitoring
   - **Event Ingestion**: SEC EDGAR, news feeds, halt monitoring
   - **Entity Linking**: Automatic ticker and CIK code extraction
   - **Severity Scoring**: Multi-factor severity assessment (A/B/C levels)
   - **Alert Posting**: Multi-platform alerts (Twitter, Discord, Slack, Webhooks)
   - **Compliance**: Two-source rule, correction protocols, audit logging
   - **Performance**: Sub-60s detection latency, 98%+ precision target

6. **Infrastructure** - Production-ready foundation
   - **Docker Compose**: PostgreSQL and Redis services
   - **Database Schema**: Complete trading tables with proper indexing
   - **Event Contracts**: Versioned JSON schemas for all events
   - **Fixed-point Money Math**: rust_decimal for precision
   - **Risk Guardrails**: Comprehensive risk management system

## Quick Start

### Prerequisites

- macOS (recommended) or Linux
- Docker and Docker Compose
- Node.js 18+
- Rust
- Python 3.11+

### Automated Setup

```bash
# Clone the repository
git clone https://github.com/johnjohn2410/Frontier-Trading.git
cd Frontier-Trading

# Start infrastructure and all services
./scripts/start-dev.sh
```

This will:
- Start PostgreSQL and Redis with Docker Compose
- Build and start all Rust microservices
- Start the React frontend development server
- Initialize the AI Copilot service

### Manual Setup

If you prefer manual setup:

1. **Start Infrastructure**
   ```bash
   docker compose up -d
   ```

2. **Build Rust Services**
   ```bash
   cd rust
   cargo build --release
   cd ..
   ```

3. **Install Frontend Dependencies**
   ```bash
   cd frontend
   npm install
   cd ..
   ```

4. **Set up Python Environment**
   ```bash
   cd ai
   python3 -m venv venv
   source venv/bin/activate
   pip install -r requirements.txt
   cd ..
   ```

## Running the Platform

### Start All Services

```bash
./scripts/start-dev.sh
```

This starts:
- **API Gateway**: http://localhost:8000
- **Market Data Service**: http://localhost:8001
- **Notification Service**: http://localhost:8002
- **Copilot Service**: http://localhost:8004
- **Market Events Service**: http://localhost:8005
- **Frontend**: http://localhost:3000
- **PostgreSQL**: localhost:5432
- **Redis**: localhost:6379

### Test the Vertical Slice

```bash
./scripts/test-vertical-slice.sh
```

This validates the complete flow:
- Service health checks
- Redis Stream validation
- Order placement testing
- WebSocket connectivity
- Database connectivity

### Stop All Services

```bash
# Stop application services
pkill -f "cargo run"
pkill -f "npm run dev"

# Stop infrastructure
docker compose down
```

## Testing

### Vertical Slice Testing
```bash
./scripts/test-vertical-slice.sh
```

### C++ Trading Engine
```bash
cd cpp
ctest --test-dir build --output-on-failure
```

### Rust Services
```bash
cd rust
cargo test
```

### Frontend
```bash
cd frontend
npm test
```

## Project Structure

```
Frontier-Trading/
├── cpp/                    # C++ Trading Engine
│   ├── include/frontier/   # Header files
│   ├── src/               # Source files
│   ├── tests/             # Unit tests
│   └── CMakeLists.txt     # Build configuration
├── rust/                  # Rust Microservices
│   ├── api_gateway/       # API Gateway with WebSocket
│   ├── market_data/       # Market data simulation
│   ├── copilot/           # AI Copilot service
│   ├── notifications/     # Alert & notification service
│   ├── shared/            # Shared types and event bus
│   └── migrations/        # Database migrations
├── frontend/              # React Frontend
│   ├── src/
│   │   ├── components/    # React components
│   │   ├── hooks/         # Custom hooks (useStream)
│   │   ├── pages/         # Page components
│   │   └── App.tsx        # Main application
│   └── package.json       # Dependencies
├── ai/                    # AI Copilot (Python)
│   ├── copilot.py         # Main AI service
│   └── requirements.txt   # Python dependencies
├── schemas/               # Event contracts
│   └── events/            # JSON schemas for all events
├── scripts/               # Development scripts
├── docker-compose.yml     # Infrastructure setup
└── README.md             # This file
```

## Architecture

### Event-Driven Microservices

The platform uses a modern event-driven architecture:

1. **Market Data Service** publishes simulated ticks to Redis Streams
2. **Copilot Service** consumes ticks, detects EMA crosses, generates suggestions
3. **API Gateway** handles order placement and WebSocket streaming
4. **Event Bus** (Redis Streams) enables loose coupling between services

### Data Flow

```
Market Data → Redis Streams → Copilot → Suggestions → UI
                                    ↓
Orders ← API Gateway ← UI ← WebSocket ← Event Bus
```

### Key Technologies

- **Backend**: Rust with Axum, Tokio, SQLx
- **Frontend**: React with TypeScript, Tailwind CSS
- **Database**: PostgreSQL with proper money types (NUMERIC)
- **Message Bus**: Redis Streams for event-driven communication
- **Infrastructure**: Docker Compose for development
- **AI**: Python with OpenAI integration

## Configuration

### Environment Variables

Create a `.env` file in the root directory:

```env
# Database Configuration
DATABASE_URL=postgres://frontier:frontier@localhost:5432/frontier
REDIS_URL=redis://localhost:6379

# Service Ports
API_GATEWAY_PORT=8000
MARKET_DATA_PORT=8001
NOTIFICATION_PORT=8002
COPILOT_PORT=8004
MARKET_EVENTS_PORT=8005

# AI Configuration
OPENAI_API_KEY=your-openai-api-key-here

# Alpaca API Configuration (Optional - for real market data)
ALPACA_API_KEY=your-alpaca-api-key-here
ALPACA_SECRET_KEY=your-alpaca-secret-key-here
ALPACA_PAPER_TRADING=true
ALPACA_SYMBOLS=AAPL,GOOGL,MSFT,TSLA,SPY,QQQ
ALPACA_UPDATE_INTERVAL_MS=1000

# Development
RUST_LOG=info
LOG_LEVEL=info
ENVIRONMENT=development
```

## Market Events Alert System

The Market Events system provides real-time monitoring and alerting for material market events, designed for active traders, analysts, and risk teams.

### Event Categories

- **Critical Incidents**: Accidents, safety events, production outages, data breaches
- **Regulatory Filings**: SEC 8-K, material 6-Ks, guidance changes
- **Trading Status**: Halts/resumptions, material short-sale restrictions
- **Earnings Surprises**: Material beats/misses, guidance revisions
- **Legal & Regulatory**: Major lawsuits, FTC/DoJ/EU actions, consent decrees
- **Product Recalls**: Recalls, FDA actions, withdrawals
- **Leadership Changes**: CEO/CFO departures/appointments
- **Crypto Incidents**: Large protocol exploits, exchange incidents, stablecoin depegs

### Severity Levels

- **Level A**: Loss of life/catastrophic accident, regulatory shutdown, massive breach (>10M records)
- **Level B**: Major recall, guidance withdrawal, CEO resignation, significant litigation/penalty
- **Level C**: Plant outage, product delay, localized incident, exec reshuffle

### Alert Format

```
$TICKER — [CATEGORY]: Headline (Location if relevant) [Developing/Confirmed]
• Source: <link1> (+ <link2> if needed)
• Early move (5–15m): −1.8% vs S&P −0.2%
• Potential impact: operations/liability/risk monitoring
Not investment advice. Time: 14:32 ET
```

### Compliance & Safety

- **Two-Source Rule**: Sensitive incidents require multiple sources unless official
- **Correction Protocol**: 10-minute SLA for corrections
- **Audit Logging**: Complete decision trail for all events
- **Rate Limiting**: Controlled posting cadence to prevent spam
- **Manual Override**: Emergency stop for sensitive events

### Performance Targets

- **Detection Latency**: <60s median, <180s p95
- **Precision**: ≥98% alerts not corrected
- **Recall Coverage**: ≥95% of severe incidents captured
- **Thread Discipline**: ≤1 alert per incident + ≤5 updates until resolution

### Multi-Provider Market Data Setup

The platform uses a smart multi-provider system for comprehensive market coverage:

#### **Provider Stack (Free Tier)**

1. **US Equities (Real-time)**: Alpaca Market Data Basic
   - 30 symbols, ~200 REST calls/min
   - Real-time via IEX (15-min delay for major exchanges)
   - Configure with API keys (optional)

2. **Crypto Majors (Real-time)**: Binance Spot WebSocket
   - BTC, ETH, ADA, DOT, etc.
   - 1200 requests/min, public & free
   - No API key required

3. **Meme Coins & Long-tail**: DEXScreener + Birdeye
   - DEXScreener: 300 req/min, no API key
   - Birdeye: 30,000 CUs/month, 1 RPS
   - Chain:contract format (e.g., `SOL:So111...`)

#### **Configuration**

```bash
# Optional: Alpaca for US equities
ALPACA_API_KEY=your-api-key-here
ALPACA_SECRET_KEY=your-secret-key-here
ALPACA_PAPER_TRADING=true
ALPACA_SYMBOLS=AAPL,GOOGL,MSFT,TSLA,SPY

# Binance and DEXScreener are always enabled (free)
```

#### **Symbol Formats**

- **US Equities**: `AAPL`, `GOOGL`, `MSFT`
- **Crypto Majors**: `BTCUSD.BIN`, `ETHUSD.BIN`, `ADAUSD.BIN`
- **Meme Coins**: `SOL:So11111111111111111111111111111111111111112`

#### **Testing**

```bash
# Test multi-provider system
./scripts/test-multi-provider.sh

# Test individual providers
./scripts/test-alpaca-integration.sh
```

The platform automatically routes symbols to the best provider and falls back to simulation if needed.

## Monitoring

### Redis Streams
```bash
# Monitor suggestions
redis-cli XREAD COUNT 10 STREAMS suggestions.stream 0

# Monitor orders
redis-cli XREAD COUNT 10 STREAMS orders.stream 0

# Monitor market data
redis-cli XREAD COUNT 10 STREAMS ticks.AAPL 0
```

### Service Health
```bash
curl http://localhost:8000/health  # API Gateway
curl http://localhost:8001/health  # Market Data
curl http://localhost:8004/health  # Copilot
```

## Next Steps

### Immediate Priorities

1. **C++ Engine Integration**
   - Complete JSON-RPC interface
   - Connect to API Gateway
   - Add real order execution

2. **Real Market Data**
   - Implement adapter pattern
   - Add Polygon/Alpaca/IEX integration
   - Toggle between sim and live data

3. **Authentication & Security**
   - JWT-based authentication
   - Per-user portfolio isolation
   - API rate limiting

4. **Enhanced Frontend**
   - Real-time charts with technical indicators
   - Order management interface
   - Portfolio analytics dashboard

### Future Enhancements

- **Strategy Backtesting**
  - Vectorized backtesting engine
  - Strategy catalog and sharing
  - Performance analytics

- **Advanced AI Features**
  - Natural language queries
  - Predictive analytics
  - Automated portfolio optimization

- **Production Features**
  - Kubernetes deployment
  - Monitoring and observability
  - CI/CD pipeline
  - Security hardening

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

For support and questions:
- Create an issue in the GitHub repository
- Check the documentation in the `docs/` folder
- Review the troubleshooting guide

---

**Note**: This is a development platform. Always test thoroughly before using with real money. The platform includes paper trading mode for safe experimentation.
