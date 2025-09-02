#!/bin/bash

# Frontier Trading Platform - Development Startup Script

set -e

echo "🚀 Starting Frontier Trading Platform Development Environment"

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

# Check if required tools are installed
check_dependencies() {
    print_status "Checking dependencies..."
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        print_error "Docker not found. Please install Docker first."
        exit 1
    fi
    
    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        print_error "Docker Compose not found. Please install Docker Compose first."
        exit 1
    fi
    
    # Check Rust
    if ! command -v cargo &> /dev/null; then
        print_error "Rust (cargo) not found"
        exit 1
    fi
    
    # Check Node.js
    if ! command -v node &> /dev/null; then
        print_error "Node.js not found"
        exit 1
    fi
    
    print_success "All dependencies found"
}

# Start infrastructure with Docker Compose
start_infrastructure() {
    print_status "Starting infrastructure with Docker Compose..."
    
    # Start PostgreSQL and Redis
    docker compose up -d
    
    # Wait for services to be ready
    print_status "Waiting for infrastructure to be ready..."
    sleep 10
    
    # Check if services are running
    if docker compose ps | grep -q "Up"; then
        print_success "Infrastructure started successfully"
    else
        print_error "Failed to start infrastructure"
        exit 1
    fi
}

# Build Rust services
build_rust_services() {
    print_status "Building Rust services..."
    
    cd rust
    
    # Build all workspace members
    cargo build --workspace
    
    print_success "Rust services built successfully"
    cd ..
}

# Build C++ trading engine
build_cpp_engine() {
    print_status "Building C++ trading engine..."
    
    cd cpp
    
    # Create build directory
    mkdir -p build
    cd build
    
    # Configure with CMake
    cmake .. -DCMAKE_BUILD_TYPE=Release
    
    # Build
    make -j$(nproc)
    
    print_success "C++ engine built successfully"
    cd ../..
}

# Install frontend dependencies
install_frontend_deps() {
    print_status "Installing frontend dependencies..."
    
    cd frontend
    
    # Install dependencies
    npm install
    
    print_success "Frontend dependencies installed"
    cd ..
}

# Start all services
start_services() {
    print_status "Starting services..."
    
    # Start Market Data service
    print_status "Starting Market Data service on port 8001..."
    cd rust/market_data
    cargo run &
    MARKET_DATA_PID=$!
    cd ../..
    
    # Start Copilot service
    print_status "Starting Copilot service on port 8004..."
    cd rust/copilot
    cargo run &
    COPILOT_PID=$!
    cd ../..
    
    # Start API Gateway
    print_status "Starting API Gateway on port 8000..."
    cd rust/api_gateway
    cargo run &
    GATEWAY_PID=$!
    cd ../..
    
    # Start notification service
    print_status "Starting notification service on port 8002..."
    cd rust/notifications
    cargo run &
    NOTIFICATION_PID=$!
    cd ../..
    
    # Start C++ trading engine
    print_status "Starting C++ trading engine on port 8003..."
    cd cpp/build
    ./frontier_trading &
    CPP_ENGINE_PID=$!
    cd ../..
    
    # Start frontend development server
    print_status "Starting frontend development server..."
    cd frontend
    npm run dev &
    FRONTEND_PID=$!
    cd ..
    
    # Save PIDs for cleanup
    echo $MARKET_DATA_PID > .dev-pids
    echo $COPILOT_PID >> .dev-pids
    echo $GATEWAY_PID >> .dev-pids
    echo $NOTIFICATION_PID >> .dev-pids
    echo $CPP_ENGINE_PID >> .dev-pids
    echo $FRONTEND_PID >> .dev-pids
    
    print_success "All services started"
    print_status "Services running on:"
    echo "  - API Gateway: http://localhost:8000"
    echo "  - Market Data: http://localhost:8001"
    echo "  - Notifications: http://localhost:8002"
    echo "  - C++ Engine: http://localhost:8003"
    echo "  - Copilot: http://localhost:8004"
    echo "  - Frontend: http://localhost:3000"
    echo "  - PostgreSQL: localhost:5432"
    echo "  - Redis: localhost:6379"
    
    # Wait for user interrupt
    echo ""
    print_status "Press Ctrl+C to stop all services"
    
    # Function to cleanup on exit
    cleanup() {
        print_status "Stopping services..."
        
        if [ -f .dev-pids ]; then
            while read pid; do
                if kill -0 $pid 2>/dev/null; then
                    kill $pid
                    print_status "Stopped process $pid"
                fi
            done < .dev-pids
            rm .dev-pids
        fi
        
        print_status "Stopping infrastructure..."
        docker compose down
        
        print_success "All services stopped"
        exit 0
    }
    
    # Set up signal handlers
    trap cleanup SIGINT SIGTERM
    
    # Wait for background processes
    wait
}

# Main execution
main() {
    # Check if we're in the right directory
    if [ ! -f "README.md" ] || [ ! -f "docker-compose.yml" ]; then
        print_error "Please run this script from the project root directory"
        exit 1
    fi
    
    # Check dependencies
    check_dependencies
    
    # Start infrastructure
    start_infrastructure
    
    # Build components
    build_rust_services
    build_cpp_engine
    install_frontend_deps
    
    # Start services
    start_services
}

# Run main function
main "$@"
