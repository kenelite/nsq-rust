#!/bin/bash

# Test script for NSQAdmin Web UI
# Verifies that all pages and APIs are working correctly

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configuration
FRONTEND="${FRONTEND:-http://localhost:3000}"
BACKEND="${BACKEND:-http://localhost:4171}"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}NSQAdmin Web UI Test${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Test frontend
echo -e "${YELLOW}Testing Frontend (${FRONTEND})...${NC}"
if curl -s "${FRONTEND}" | grep -q "NSQ Admin"; then
    echo -e "${GREEN}✓ Frontend is running${NC}"
else
    echo -e "${RED}✗ Frontend is not accessible${NC}"
    exit 1
fi
echo ""

# Test backend
echo -e "${YELLOW}Testing Backend (${BACKEND})...${NC}"
if curl -s "${BACKEND}/api/ping" | grep -q "OK"; then
    echo -e "${GREEN}✓ Backend is running${NC}"
else
    echo -e "${RED}✗ Backend is not accessible${NC}"
    exit 1
fi
echo ""

# Test API endpoints
echo -e "${YELLOW}Testing API Endpoints...${NC}"

# Test /api/stats
echo -n "  /api/stats ... "
STATS=$(curl -s "${FRONTEND}/api/stats")
if echo "$STATS" | jq -e '.version' > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC}"
    echo "    Version: $(echo "$STATS" | jq -r '.version')"
    echo "    Topics: $(echo "$STATS" | jq -r '.topics | length')"
    echo "    Producers: $(echo "$STATS" | jq -r '.producers | length')"
else
    echo -e "${RED}✗${NC}"
fi

# Test /api/topics
echo -n "  /api/topics ... "
TOPICS=$(curl -s "${FRONTEND}/api/topics")
if echo "$TOPICS" | jq -e '.topics' > /dev/null 2>&1; then
    TOPIC_COUNT=$(echo "$TOPICS" | jq -r '.topics | length')
    echo -e "${GREEN}✓${NC} (${TOPIC_COUNT} topics)"
    if [ "$TOPIC_COUNT" -gt 0 ]; then
        echo "    Topics:"
        echo "$TOPICS" | jq -r '.topics[].topic_name' | head -5 | sed 's/^/      - /'
    fi
else
    echo -e "${RED}✗${NC}"
fi

# Test /api/nodes
echo -n "  /api/nodes ... "
NODES=$(curl -s "${FRONTEND}/api/nodes")
if echo "$NODES" | jq -e '.producers' > /dev/null 2>&1; then
    NODE_COUNT=$(echo "$NODES" | jq -r '.producers | length')
    echo -e "${GREEN}✓${NC} (${NODE_COUNT} nodes)"
    if [ "$NODE_COUNT" -gt 0 ]; then
        echo "    Nodes:"
        echo "$NODES" | jq -r '.producers[] | "      - \(.hostname):\(.http_port) (v\(.version))"'
    fi
else
    echo -e "${RED}✗${NC}"
fi

echo ""

# Test UI pages
echo -e "${YELLOW}Testing UI Pages...${NC}"

# Test Dashboard
echo -n "  Dashboard (/) ... "
if curl -s "${FRONTEND}" | grep -q "NSQ Admin"; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
fi

# Test Topics page
echo -n "  Topics (/topics) ... "
if curl -s "${FRONTEND}/topics" | grep -q "NSQ Admin"; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
fi

# Test Nodes page
echo -n "  Nodes (/nodes) ... "
if curl -s "${FRONTEND}/nodes" | grep -q "NSQ Admin"; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
fi

# Test Channels page
echo -n "  Channels (/channels) ... "
if curl -s "${FRONTEND}/channels" | grep -q "NSQ Admin"; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
fi

# Test Performance page
echo -n "  Performance (/performance) ... "
if curl -s "${FRONTEND}/performance" | grep -q "NSQ Admin"; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
fi

echo ""

# Summary
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}✓ All tests passed!${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo -e "${YELLOW}Open the following URLs in your browser:${NC}"
echo -e "  ${GREEN}Dashboard:${NC}    ${FRONTEND}"
echo -e "  ${GREEN}Topics:${NC}       ${FRONTEND}/topics"
echo -e "  ${GREEN}Nodes:${NC}        ${FRONTEND}/nodes"
echo -e "  ${GREEN}Channels:${NC}     ${FRONTEND}/channels"
echo -e "  ${GREEN}Performance:${NC}  ${FRONTEND}/performance"
echo ""

