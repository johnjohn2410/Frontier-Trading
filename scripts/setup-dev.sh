#!/bin/bash

set -e

echo "🚀 Setting up Frontier Trading Development Environment"
echo "=================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
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

# Check if we're on macOS
if [[ "$OSTYPE" == "darwin"* ]]; then
    print_status "Detected macOS"
    
    # Check if Homebrew is installed
    if ! command -v brew &> /dev/null; then
        print_error "Homebrew is not installed. Please install it first:"
        echo "  /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
        exit 1
    fi
    
    print_status "Installing/updating system dependencies..."
    
    # Install/update system dependencies
    brew update
    brew install cmake
    brew install googletest
    brew install postgresql@15
    brew install redis
    brew install node@18
    brew install rust
    brew install python@3.11
    
    # Start PostgreSQL
    print_status "Starting PostgreSQL..."
    brew services start postgresql@15
    
    # Start Redis
    print_status "Starting Redis..."
    brew services start redis
    
    # Wait for services to be ready
    print_status "Waiting for database services to be ready..."
    sleep 5
    
    # Check if PostgreSQL is running
    if ! pg_isready -h localhost -p 5432 &> /dev/null; then
        print_error "PostgreSQL is not running. Please start it manually:"
        echo "  brew services start postgresql@15"
        exit 1
    fi
    
    # Check if Redis is running
    if ! redis-cli ping &> /dev/null; then
        print_error "Redis is not running. Please start it manually:"
        echo "  brew services start redis"
        exit 1
    fi
    
    print_success "System dependencies installed and services started"
    
else
    print_warning "This script is optimized for macOS. You may need to install dependencies manually for your system."
    print_status "Please ensure you have the following installed:"
    echo "  - CMake"
    echo "  - PostgreSQL"
    echo "  - Redis"
    echo "  - Node.js 18+"
    echo "  - Rust"
    echo "  - Python 3.11+"
fi

# Create database
print_status "Setting up database..."
createdb frontier_trading 2>/dev/null || print_warning "Database 'frontier_trading' already exists or could not be created"

# Set up environment variables
print_status "Setting up environment variables..."
if [ ! -f .env ]; then
    cat > .env << EOF
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
EOF
    print_success "Created .env file"
else
    print_warning ".env file already exists"
fi

# Build C++ components
print_status "Building C++ trading engine..."
cd cpp

# Clean previous build
rm -rf build

# Set compiler environment variables
export CC=/usr/bin/clang
export CXX=/usr/bin/clang++

# Configure and build
cmake -S . -B build -G "Unix Makefiles" -DCMAKE_OSX_ARCHITECTURES=arm64 -DCMAKE_BUILD_TYPE=Release
cmake --build build -j

# Run tests
print_status "Running C++ tests..."
ctest --test-dir build --output-on-failure

cd ..
print_success "C++ components built and tested"

# Build Rust components
print_status "Building Rust backend services..."
cd rust

# Install dependencies and build
cargo build --release

# Run database migrations for notifications service
print_status "Running database migrations..."
cd notifications
cargo sqlx database create
cargo sqlx migrate run
cd ..

cd ..
print_success "Rust components built"

# Install frontend dependencies
print_status "Installing frontend dependencies..."
cd frontend

# Install Node.js dependencies
npm install

# Build frontend
print_status "Building frontend..."
npm run build

cd ..
print_success "Frontend dependencies installed and built"

# Install Python dependencies for AI Copilot
print_status "Installing Python dependencies..."
cd ai

# Create virtual environment if it doesn't exist
if [ ! -d "venv" ]; then
    python3 -m venv venv
fi

# Activate virtual environment and install dependencies
source venv/bin/activate
pip install --upgrade pip
pip install -r requirements.txt

cd ..
print_success "Python dependencies installed"

# Create development startup script
print_status "Creating development startup script..."
cat > scripts/start-dev.sh << 'EOF'
#!/bin/bash

set -e

echo "🚀 Starting Frontier Trading Development Environment"
echo "=================================================="

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Function to check if a port is in use
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null ; then
        return 0
    else
        return 1
    fi
}

# Function to start service if port is available
start_service() {
    local name=$1
    local port=$2
    local command=$3
    
    if check_port $port; then
        echo "⚠️  Port $port is already in use. Skipping $name."
    else
        echo "🟢 Starting $name on port $port..."
        $command &
        echo $! > /tmp/frontier-$name.pid
    fi
}

# Kill any existing processes
echo "🧹 Cleaning up existing processes..."
pkill -f "frontier-trading" || true
pkill -f "notification-service" || true
pkill -f "ai-copilot" || true

# Start services
start_service "API" 8000 "cd rust/api && cargo run --release"
start_service "WebSocket" 8001 "cd rust/websocket && cargo run --release"
start_service "Notifications" 8002 "cd rust/notifications && cargo run --release"

# Start AI Copilot
echo "🤖 Starting AI Copilot..."
cd ai
source venv/bin/activate
python copilot.py &
echo $! > /tmp/frontier-ai-copilot.pid
cd ..

# Start frontend development server
echo "🌐 Starting frontend development server..."
cd frontend
npm run dev &
echo $! > /tmp/frontier-frontend.pid
cd ..

echo ""
echo "✅ Development environment started!"
echo ""
echo "Services running:"
echo "  - Frontend: http://localhost:5173"
echo "  - API: http://localhost:8000"
echo "  - WebSocket: ws://localhost:8001"
echo "  - Notifications: http://localhost:8002"
echo "  - AI Copilot: Running in background"
echo ""
echo "To stop all services, run: ./scripts/stop-dev.sh"
echo ""

# Wait for user input to stop
read -p "Press Enter to stop all services..."
./scripts/stop-dev.sh
EOF

chmod +x scripts/start-dev.sh

# Create stop script
cat > scripts/stop-dev.sh << 'EOF'
#!/bin/bash

echo "🛑 Stopping Frontier Trading Development Environment"
echo "=================================================="

# Kill processes by PID files
for pidfile in /tmp/frontier-*.pid; do
    if [ -f "$pidfile" ]; then
        pid=$(cat "$pidfile")
        if kill -0 "$pid" 2>/dev/null; then
            echo "Stopping process $pid..."
            kill "$pid"
        fi
        rm "$pidfile"
    fi
done

# Kill any remaining frontier processes
pkill -f "frontier-trading" || true
pkill -f "notification-service" || true
pkill -f "ai-copilot" || true

echo "✅ All services stopped"
EOF

chmod +x scripts/stop-dev.sh

print_success "Development environment setup complete!"
echo ""
echo "🎉 Setup Summary:"
echo "  ✅ System dependencies installed"
echo "  ✅ Database services started"
echo "  ✅ C++ trading engine built and tested"
echo "  ✅ Rust backend services built"
echo "  ✅ Frontend dependencies installed"
echo "  ✅ Python AI Copilot dependencies installed"
echo "  ✅ Environment configuration created"
echo "  ✅ Development scripts created"
echo ""
echo "🚀 To start the development environment:"
echo "  ./scripts/start-dev.sh"
echo ""
echo "📝 Next steps:"
echo "  1. Update .env file with your API keys"
echo "  2. Run ./scripts/start-dev.sh to start all services"
echo "  3. Open http://localhost:5173 in your browser"
echo ""
echo "🔧 Useful commands:"
echo "  ./scripts/stop-dev.sh     - Stop all services"
echo "  cd cpp && ./build/frontier_trading - Run C++ demo"
echo "  cd rust && cargo test     - Run Rust tests"
echo "  cd frontend && npm test   - Run frontend tests"
