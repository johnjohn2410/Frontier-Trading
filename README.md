# Frontier Trading Platform

A modern, multi-asset trading platform built with C++, Rust, and React, featuring an AI-powered Copilot for intelligent trading insights and alerts.

## 🚀 Current Status

### ✅ Completed Components

1. **C++ Trading Engine** - Core trading logic with paper trading support
   - Order management and execution
   - Position tracking and P&L calculation
   - Risk management and limits
   - Real-time market data processing
   - Comprehensive test suite

2. **AI Copilot Alert System** - Intelligent stock monitoring and notifications
   - Price alerts (above/below targets, percentage changes)
   - News monitoring with sentiment analysis
   - Technical analysis alerts (moving averages, support/resistance)
   - Volume spike detection
   - AI-powered alert suggestions
   - Real-time notifications via multiple channels

3. **Frontend Components** - React-based user interface
   - Alert Manager with AI insights
   - Real-time notification center
   - Modern, responsive design
   - TypeScript for type safety

4. **Backend Infrastructure** - Rust-based microservices
   - Notification service with REST API
   - Database layer with PostgreSQL
   - WebSocket support for real-time updates
   - Comprehensive data models

### 🔧 Development Environment

- **C++**: Trading engine with CMake build system
- **Rust**: Backend services with Axum web framework
- **React**: Frontend with TypeScript and modern tooling
- **Python**: AI Copilot with OpenAI integration
- **PostgreSQL**: Primary database
- **Redis**: Caching and real-time data

## 🛠️ Quick Start

### Prerequisites

- macOS (recommended) or Linux
- Homebrew (macOS)
- Node.js 18+
- Rust
- Python 3.11+
- PostgreSQL 15+
- Redis

### Automated Setup

```bash
# Clone the repository
git clone <repository-url>
cd Frontier-Trading

# Run the automated setup script
./scripts/setup-dev.sh
```

This script will:
- Install all system dependencies
- Set up PostgreSQL and Redis
- Build C++ trading engine
- Build Rust backend services
- Install frontend dependencies
- Set up Python environment for AI Copilot
- Create development scripts

### Manual Setup

If you prefer manual setup or are on a different platform:

1. **Install Dependencies**
   ```bash
   # macOS
   brew install cmake googletest postgresql@15 redis node@18 rust python@3.11
   
   # Start services
   brew services start postgresql@15
   brew services start redis
   ```

2. **Set up Database**
   ```bash
   createdb frontier_trading
   ```

3. **Build C++ Engine**
   ```bash
   cd cpp
   export CC=/usr/bin/clang
   export CXX=/usr/bin/clang++
   cmake -S . -B build -G "Unix Makefiles" -DCMAKE_OSX_ARCHITECTURES=arm64
   cmake --build build -j
   cd ..
   ```

4. **Build Rust Services**
   ```bash
   cd rust
   cargo build --release
   cd ..
   ```

5. **Install Frontend Dependencies**
   ```bash
   cd frontend
   npm install
   cd ..
   ```

6. **Set up Python Environment**
   ```bash
   cd ai
   python3 -m venv venv
   source venv/bin/activate
   pip install -r requirements.txt
   cd ..
   ```

## 🚀 Running the Platform

### Start All Services

```bash
./scripts/start-dev.sh
```

This starts:
- Frontend development server (http://localhost:5173)
- API service (http://localhost:8000)
- WebSocket service (ws://localhost:8001)
- Notification service (http://localhost:8002)
- AI Copilot (background process)

### Stop All Services

```bash
./scripts/stop-dev.sh
```

## 🧪 Testing

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

## 📁 Project Structure

```
Frontier-Trading/
├── cpp/                    # C++ Trading Engine
│   ├── include/frontier/   # Header files
│   ├── src/               # Source files
│   ├── tests/             # Unit tests
│   └── CMakeLists.txt     # Build configuration
├── rust/                  # Rust Backend Services
│   ├── api/              # REST API service
│   ├── notifications/    # Alert & notification service
│   ├── shared/           # Shared types and utilities
│   └── Cargo.toml        # Workspace configuration
├── frontend/             # React Frontend
│   ├── src/              # Source code
│   ├── components/       # React components
│   └── package.json      # Dependencies
├── ai/                   # AI Copilot
│   ├── copilot.py        # Main AI service
│   └── requirements.txt  # Python dependencies
├── scripts/              # Development scripts
├── config/               # Configuration files
└── docs/                 # Documentation
```

## 🔧 Configuration

### Environment Variables

Create a `.env` file in the root directory:

```env
# Database Configuration
DATABASE_URL=postgres://localhost/frontier_trading
REDIS_URL=redis://localhost:6379

# Server Configuration
API_PORT=8000
WEBSOCKET_PORT=8001
NOTIFICATION_PORT=8002

# AI Configuration
OPENAI_API_KEY=your-openai-api-key-here

# Market Data APIs
ALPHA_VANTAGE_API_KEY=your-alpha-vantage-key-here
BINANCE_API_KEY=your-binance-key-here
BINANCE_SECRET_KEY=your-binance-secret-here

# Logging
RUST_LOG=info
LOG_LEVEL=info

# Development
ENVIRONMENT=development
DEBUG=true
```

## 🎯 Next Steps

### Immediate Priorities

1. **Complete Backend Services**
   - Implement authentication and user management
   - Add market data integration (Yahoo Finance, Alpha Vantage)
   - Implement WebSocket real-time data streaming
   - Add order execution and portfolio management

2. **Enhance Frontend**
   - Create main trading dashboard
   - Add real-time charts and technical indicators
   - Implement order entry and management interface
   - Add portfolio overview and P&L tracking

3. **AI Copilot Integration**
   - Connect AI insights to real market data
   - Implement natural language query processing
   - Add trading strategy suggestions
   - Create risk assessment and recommendations

4. **Testing & Quality**
   - Add comprehensive integration tests
   - Implement end-to-end testing
   - Add performance monitoring
   - Set up CI/CD pipeline

### Future Enhancements

- **Advanced Trading Features**
  - Options and derivatives trading
  - Algorithmic trading strategies
  - Backtesting framework
  - Paper trading competitions

- **AI Capabilities**
  - Predictive analytics
  - Sentiment analysis from social media
  - Automated portfolio rebalancing
  - Personalized trading recommendations

- **Platform Features**
  - Mobile app (React Native)
  - Social trading features
  - Educational content and tutorials
  - Community features

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

For support and questions:
- Create an issue in the GitHub repository
- Check the documentation in the `docs/` folder
- Review the troubleshooting guide

---

**Note**: This is a development platform. Always test thoroughly before using with real money. The platform includes paper trading mode for safe experimentation.
