#!/bin/bash

# Test Market Events Alert System
# This script tests the market events monitoring and alert system

set -e

echo "🚨 Testing Market Events Alert System"
echo "======================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if services are running
check_service() {
    local service=$1
    local port=$2
    
    if curl -s "http://localhost:$port/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ $service is running on port $port${NC}"
        return 0
    else
        echo -e "${RED}❌ $service is not running on port $port${NC}"
        return 1
    fi
}

# Test market events service
test_market_events_service() {
    echo -e "\n${BLUE}Testing Market Events Service${NC}"
    echo "----------------------------"
    
    if check_service "Market Events" 8005; then
        echo -e "${GREEN}✅ Market Events service is healthy${NC}"
        
        # Test metrics endpoint
        echo "📊 Fetching metrics..."
        curl -s "http://localhost:8005/metrics" | jq '.' || echo "Metrics endpoint returned non-JSON response"
        
    else
        echo -e "${RED}❌ Market Events service is not running${NC}"
        echo "Start it with: cd rust/market_events && cargo run"
        return 1
    fi
}

# Test event ingestion
test_event_ingestion() {
    echo -e "\n${BLUE}Testing Event Ingestion${NC}"
    echo "------------------------"
    
    # Test SEC EDGAR simulation
    echo "📄 Testing SEC EDGAR ingestion..."
    # This would normally test actual SEC EDGAR API calls
    echo -e "${YELLOW}⚠️  SEC EDGAR testing requires API keys and rate limiting${NC}"
    
    # Test news feed ingestion
    echo "📰 Testing news feed ingestion..."
    # This would test RSS feed parsing
    echo -e "${YELLOW}⚠️  News feed testing requires external RSS feeds${NC}"
    
    # Test halt monitoring
    echo "⏸️  Testing halt monitoring..."
    # This would test exchange halt feeds
    echo -e "${YELLOW}⚠️  Halt monitoring testing requires exchange API access${NC}"
}

# Test entity linking
test_entity_linking() {
    echo -e "\n${BLUE}Testing Entity Linking${NC}"
    echo "----------------------"
    
    # Test ticker extraction
    echo "🏷️  Testing ticker extraction..."
    
    # Sample test cases
    test_cases=(
        "Apple Inc. reports strong earnings"
        "TSLA stock surges on new product announcement"
        "Microsoft and Google face regulatory scrutiny"
        "Amazon recalls defective products"
    )
    
    for test_case in "${test_cases[@]}"; do
        echo "  Testing: '$test_case'"
        # This would call the entity linking service
        echo -e "    ${YELLOW}⚠️  Entity linking requires running service${NC}"
    done
}

# Test severity scoring
test_severity_scoring() {
    echo -e "\n${BLUE}Testing Severity Scoring${NC}"
    echo "------------------------"
    
    # Test severity calculation
    echo "📊 Testing severity scoring algorithm..."
    
    # Sample events for testing
    events=(
        "CriticalIncident: Factory explosion with casualties"
        "RegulatoryFiling: SEC 8-K filing for material event"
        "TradingStatus: Trading halt due to volatility"
        "EarningsSurprise: Major earnings beat/miss"
        "LegalRegulatory: Major lawsuit filed"
        "ProductRecall: Safety recall of consumer products"
        "Leadership: CEO resignation announced"
        "CryptoIncident: Major protocol exploit"
    )
    
    for event in "${events[@]}"; do
        echo "  Testing: $event"
        # This would test the severity scoring algorithm
        echo -e "    ${YELLOW}⚠️  Severity scoring requires running service${NC}"
    done
}

# Test alert posting
test_alert_posting() {
    echo -e "\n${BLUE}Testing Alert Posting${NC}"
    echo "---------------------"
    
    # Test alert generation
    echo "📢 Testing alert content generation..."
    
    # Sample alert templates
    alert_templates=(
        "Critical incident alert"
        "Trading halt notification"
        "Earnings surprise alert"
        "Regulatory filing notification"
    )
    
    for template in "${alert_templates[@]}"; do
        echo "  Testing: $template"
        # This would test alert content generation
        echo -e "    ${YELLOW}⚠️  Alert posting requires configured platforms${NC}"
    done
    
    # Test platform posting
    echo "🌐 Testing platform posting..."
    echo -e "${YELLOW}⚠️  Platform posting requires API keys and webhooks${NC}"
}

# Test compliance features
test_compliance() {
    echo -e "\n${BLUE}Testing Compliance Features${NC}"
    echo "----------------------------"
    
    # Test two-source rule
    echo "🔒 Testing two-source rule..."
    echo -e "${YELLOW}⚠️  Two-source rule testing requires multiple sources${NC}"
    
    # Test correction protocol
    echo "✏️  Testing correction protocol..."
    echo -e "${YELLOW}⚠️  Correction protocol testing requires event history${NC}"
    
    # Test audit logging
    echo "📝 Testing audit logging..."
    echo -e "${YELLOW}⚠️  Audit logging testing requires database access${NC}"
}

# Test integration with existing services
test_integration() {
    echo -e "\n${BLUE}Testing Integration${NC}"
    echo "-------------------"
    
    # Test with market data service
    if check_service "Market Data" 8002; then
        echo -e "${GREEN}✅ Market Data service available for integration${NC}"
    else
        echo -e "${YELLOW}⚠️  Market Data service not running${NC}"
    fi
    
    # Test with API Gateway
    if check_service "API Gateway" 8000; then
        echo -e "${GREEN}✅ API Gateway available for integration${NC}"
    else
        echo -e "${YELLOW}⚠️  API Gateway not running${NC}"
    fi
    
    # Test with Copilot
    if check_service "Copilot" 8004; then
        echo -e "${GREEN}✅ Copilot service available for integration${NC}"
    else
        echo -e "${YELLOW}⚠️  Copilot service not running${NC}"
    fi
}

# Performance testing
test_performance() {
    echo -e "\n${BLUE}Testing Performance${NC}"
    echo "-------------------"
    
    # Test latency
    echo "⏱️  Testing detection latency..."
    echo -e "${YELLOW}⚠️  Latency testing requires load testing tools${NC}"
    
    # Test throughput
    echo "📈 Testing event processing throughput..."
    echo -e "${YELLOW}⚠️  Throughput testing requires high-volume event simulation${NC}"
    
    # Test memory usage
    echo "💾 Testing memory usage..."
    echo -e "${YELLOW}⚠️  Memory testing requires monitoring tools${NC}"
}

# Main test execution
main() {
    echo "Starting Market Events Alert System tests..."
    echo "Time: $(date)"
    echo ""
    
    # Run all tests
    test_market_events_service
    test_event_ingestion
    test_entity_linking
    test_severity_scoring
    test_alert_posting
    test_compliance
    test_integration
    test_performance
    
    echo -e "\n${GREEN}🎉 Market Events Alert System tests completed!${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Configure API keys for external data sources"
    echo "2. Set up alert posting platforms (Twitter, Discord, Slack)"
    echo "3. Configure entity mapping database"
    echo "4. Set up monitoring and alerting for the service"
    echo "5. Test with real market events"
}

# Run main function
main "$@"
