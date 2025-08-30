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
    
    # Check C++ compiler
    if ! command -v g++ &> /dev/null && ! command -v clang++ &> /dev/null; then
        print_error "C++ compiler (g++ or clang++) not found"
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
    
    # Check CMake
    if ! command -v cmake &> /dev/null; then
        print_error "CMake not found"
        exit 1
    fi
    
    print_success "All dependencies found"
}

# Build C++ trading engine
build_cpp_engine() {
    print_status "Building C++ trading engine..."
    
    cd cpp
    
    # Create build directory
    mkdir -p build
    cd build
    
    # Configure with CMake
    cmake .. -DCMAKE_BUILD_TYPE=Debug
    
    # Build
    make -j$(nproc)
    
    print_success "C++ engine built successfully"
    cd ../..
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

# Install frontend dependencies
install_frontend_deps() {
    print_status "Installing frontend dependencies..."
    
    cd frontend
    
    # Install dependencies
    npm install
    
    print_success "Frontend dependencies installed"
    cd ..
}

# Start PostgreSQL (if not running)
start_postgres() {
    print_status "Checking PostgreSQL..."
    
    if ! pg_isready -q; then
        print_warning "PostgreSQL not running. Please start PostgreSQL manually:"
        echo "  brew services start postgresql@14  # macOS"
        echo "  sudo systemctl start postgresql    # Linux"
        echo "  # Or start your preferred database"
    else
        print_success "PostgreSQL is running"
    fi
}

# Start Redis (if not running)
start_redis() {
    print_status "Checking Redis..."
    
    if ! redis-cli ping &> /dev/null; then
        print_warning "Redis not running. Please start Redis manually:"
        echo "  brew services start redis          # macOS"
        echo "  sudo systemctl start redis         # Linux"
    else
        print_success "Redis is running"
    fi
}

# Start all services
start_services() {
    print_status "Starting services..."
    
    # Start C++ engine in background
    print_status "Starting C++ trading engine..."
    cd cpp/build
    ./frontier_trading &
    CPP_PID=$!
    cd ../..
    
    # Start Rust API server in background
    print_status "Starting Rust API server..."
    cd rust
    cargo run --bin api &
    RUST_PID=$!
    cd ..
    
    # Start frontend development server
    print_status "Starting frontend development server..."
    cd frontend
    npm run dev &
    FRONTEND_PID=$!
    cd ..
    
    # Save PIDs for cleanup
    echo $CPP_PID > .dev-pids
    echo $RUST_PID >> .dev-pids
    echo $FRONTEND_PID >> .dev-pids
    
    print_success "All services started"
    print_status "Services running on:"
    echo "  - Frontend: http://localhost:5173"
    echo "  - Rust API: http://localhost:8080"
    echo "  - C++ Engine: Running on port 8081"
    
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
    if [ ! -f "README.md" ] || [ ! -d "cpp" ] || [ ! -d "rust" ] || [ ! -d "frontend" ]; then
        print_error "Please run this script from the project root directory"
        exit 1
    fi
    
    # Check dependencies
    check_dependencies
    
    # Start database services
    start_postgres
    start_redis
    
    # Build components
    build_cpp_engine
    build_rust_services
    install_frontend_deps
    
    # Start services
    start_services
}

# Run main function
main "$@"
