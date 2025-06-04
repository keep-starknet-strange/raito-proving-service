#!/bin/bash

# Quick API Test Script for Raito Proving Service
# Assumes service is already running on localhost:8080

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

BASE_URL="http://localhost:8080"

echo -e "${BLUE}🧪 Quick API Test for Raito Proving Service${NC}"
echo -e "${YELLOW}Assuming service is running on $BASE_URL${NC}"
echo "=================================================="

# Test 1: Health Check
echo -e "\n${GREEN}1. Health Check${NC}"
curl -s "$BASE_URL/healthz" | jq '.'

# Test 2: Blocks List
echo -e "\n${GREEN}2. Blocks List${NC}"
curl -s "$BASE_URL/v1/blocks?limit=3" | jq '.blocks[] | {height, hash, tx_count, verified}'

# Test 3: Block Detail
echo -e "\n${GREEN}3. Block Detail (Height 869123)${NC}"
curl -s "$BASE_URL/v1/blocks/869123" | jq '{height, hash, tx_count, prev_hash, merkle_root, txids}'

# Test 4: STARK Proof
echo -e "\n${GREEN}4. STARK Proof (First 200 chars)${NC}"
curl -s "$BASE_URL/v1/blocks/869123/proof" | head -c 200
echo "..."

# Test 5: Transaction Status
echo -e "\n\n${GREEN}5. Transaction Status${NC}"
curl -s "$BASE_URL/v1/tx/a1b2c3d4e5f67890123456789012345678901234567890123456789012345678" | jq '.'

# Test 6: Header Status  
echo -e "\n${GREEN}6. Header Status${NC}"
curl -s "$BASE_URL/v1/header/00000000000000000264e1b06b0f6f8b0c7e9e5b8b8f9c9d8b7a6f5e4d3c2b1a" | jq '.'

# Test 7: Error Cases
echo -e "\n${GREEN}7. Error Cases${NC}"
echo -e "${YELLOW}Non-existent block:${NC}"
curl -s "$BASE_URL/v1/blocks/999999" | jq '.'

echo -e "\n${YELLOW}Invalid transaction ID:${NC}"
curl -s "$BASE_URL/v1/tx/invalid" | jq '.'

# Test 8: Documentation
echo -e "\n${GREEN}8. API Documentation${NC}"
if curl -s "$BASE_URL/docs" > /dev/null; then
    echo -e "${GREEN}✅ Documentation available at: $BASE_URL/docs${NC}"
else
    echo -e "${RED}❌ Documentation not available${NC}"
fi

echo -e "\n${GREEN}✅ API test completed!${NC}"
echo -e "${BLUE}📖 Full documentation: $BASE_URL/docs${NC}"
echo -e "${BLUE}📊 Metrics: $BASE_URL/metrics${NC}" 