cmake_minimum_required(VERSION 3.20)
project(FrontierTrading VERSION 1.0.0 LANGUAGES CXX)

# Set C++ standard
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Build type
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()

# Compiler flags
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -Wextra -Wpedantic")
set(CMAKE_CXX_FLAGS_DEBUG "${CMAKE_CXX_FLAGS_DEBUG} -g -O0 -DDEBUG")
set(CMAKE_CXX_FLAGS_RELEASE "${CMAKE_CXX_FLAGS_RELEASE} -O3 -DNDEBUG")

# Find packages
find_package(Threads REQUIRED)
find_package(OpenSSL REQUIRED)
find_package(websocketpp REQUIRED)
find_package(nlohmann_json REQUIRED)
find_package(spdlog REQUIRED)

# Include directories
include_directories(${CMAKE_SOURCE_DIR}/include)

# ---- trading_engine (library) ----
add_library(trading_engine
    src/engine.cpp
    src/rpc.cpp
)
target_link_libraries(trading_engine
    PRIVATE spdlog::spdlog nlohmann_json::nlohmann_json
)
target_compile_features(trading_engine PUBLIC cxx_std_17)

# ---- frontier_trading (executable) ----
add_executable(frontier_trading
    src/main.cpp
)
target_link_libraries(frontier_trading
    PRIVATE trading_engine spdlog::spdlog
)

# ---- tests (gtest) ----
enable_testing()
find_package(GTest REQUIRED)
add_executable(trading_tests
    tests/test_engine.cpp
)
target_link_libraries(trading_tests
    PRIVATE trading_engine GTest::gtest GTest::gtest_main
)
add_test(NAME trading_tests COMMAND trading_tests)

# Installation
install(TARGETS frontier_trading trading_engine
    RUNTIME DESTINATION bin
    LIBRARY DESTINATION lib
    ARCHIVE DESTINATION lib
)

install(DIRECTORY src/ DESTINATION include/trading
    FILES_MATCHING PATTERN "*.h"
)
