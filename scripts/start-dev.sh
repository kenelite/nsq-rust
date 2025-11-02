#!/bin/bash

# NSQ Development Environment Startup Script
# This script starts all NSQ services with proper configuration

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Starting NSQ Development Environment${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if services are already running
check_port() {
    local port=$1
    local service=$2
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1 ; then
        echo -e "${YELLOW}⚠ Port $port already in use (${service})${NC}"
        return 1
    fi
    return 0
}

# Stop function
cleanup() {
    echo -e "\n${YELLOW}Stopping services...${NC}"
    pkill -f "nsqlookupd" 2>/dev/null || true
    pkill -f "nsqd" 2>/dev/null || true
    pkill -f "nsqadmin" 2>/dev/null || true
    pkill -f "vite" 2>/dev/null || true
    echo -e "${GREEN}✓ Services stopped${NC}"
    exit 0
}

trap cleanup SIGINT SIGTERM

# Build all services
echo -e "${YELLOW}Building services...${NC}"
cd "$PROJECT_ROOT"
cargo build 2>&1 | grep -E "(Compiling|Finished)" || true
echo -e "${GREEN}✓ Build complete${NC}"
echo ""

# Start NSQLookupd
echo -e "${YELLOW}Starting NSQLookupd...${NC}"
if check_port 4160 "NSQLookupd TCP"; then
    ./target/debug/nsqlookupd \
        --tcp-address=127.0.0.1:4160 \
        --http-address=127.0.0.1:4161 \
        > /tmp/nsqlookupd.log 2>&1 &
    LOOKUPD_PID=$!
    sleep 1
    if ps -p $LOOKUPD_PID > /dev/null; then
        echo -e "${GREEN}✓ NSQLookupd started (PID: $LOOKUPD_PID)${NC}"
        echo -e "  TCP:  127.0.0.1:4160"
        echo -e "  HTTP: http://127.0.0.1:4161"
    else
        echo -e "${RED}✗ Failed to start NSQLookupd${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}  Skipping (already running)${NC}"
fi
echo ""

# Start NSQd
echo -e "${YELLOW}Starting NSQd...${NC}"
if check_port 4150 "NSQd TCP"; then
    ./target/debug/nsqd \
        --tcp-address=127.0.0.1:4150 \
        --http-address=127.0.0.1:4151 \
        --lookupd-tcp-addresses=127.0.0.1:4160 \
        > /tmp/nsqd.log 2>&1 &
    NSQD_PID=$!
    sleep 1
    if ps -p $NSQD_PID > /dev/null; then
        echo -e "${GREEN}✓ NSQd started (PID: $NSQD_PID)${NC}"
        echo -e "  TCP:  127.0.0.1:4150"
        echo -e "  HTTP: http://127.0.0.1:4151"
    else
        echo -e "${RED}✗ Failed to start NSQd${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}  Skipping (already running)${NC}"
fi
echo ""

# Start NSQAdmin
echo -e "${YELLOW}Starting NSQAdmin...${NC}"
if check_port 4171 "NSQAdmin"; then
    ./target/debug/nsqadmin \
        --http-address=127.0.0.1:4171 \
        --lookupd-http-addresses=127.0.0.1:4161 \
        --nsqd-http-addresses=127.0.0.1:4151 \
        > /tmp/nsqadmin.log 2>&1 &
    NSQADMIN_PID=$!
    sleep 1
    if ps -p $NSQADMIN_PID > /dev/null; then
        echo -e "${GREEN}✓ NSQAdmin started (PID: $NSQADMIN_PID)${NC}"
        echo -e "  HTTP: http://127.0.0.1:4171"
    else
        echo -e "${RED}✗ Failed to start NSQAdmin${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}  Skipping (already running)${NC}"
fi
echo ""

# Start Frontend Dev Server
echo -e "${YELLOW}Starting Frontend (Vite)...${NC}"
if check_port 3000 "Vite"; then
    cd "$PROJECT_ROOT/nsqadmin-ui"
    if [ ! -d "node_modules" ]; then
        echo -e "${YELLOW}  Installing dependencies...${NC}"
        npm install > /dev/null 2>&1
    fi
    npm run dev > /tmp/vite.log 2>&1 &
    VITE_PID=$!
    sleep 3
    if ps -p $VITE_PID > /dev/null; then
        echo -e "${GREEN}✓ Frontend started (PID: $VITE_PID)${NC}"
        echo -e "  HTTP: http://localhost:3000"
    else
        echo -e "${RED}✗ Failed to start Frontend${NC}"
        cat /tmp/vite.log
        exit 1
    fi
else
    echo -e "${YELLOW}  Skipping (already running)${NC}"
fi
echo ""

# Summary
echo -e "${BLUE}========================================${NC}"
echo -e "${GREEN}✓ All services started successfully!${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo -e "${YELLOW}Services:${NC}"
echo -e "  • NSQLookupd: http://127.0.0.1:4161"
echo -e "  • NSQd:       http://127.0.0.1:4151"
echo -e "  • NSQAdmin:   http://127.0.0.1:4171"
echo -e "  • Frontend:   ${GREEN}http://localhost:3000${NC}"
echo ""
echo -e "${YELLOW}Web UI:${NC}"
echo -e "  ${GREEN}➜ Dashboard: http://localhost:3000${NC}"
echo -e "  ${GREEN}➜ Topics:    http://localhost:3000/topics${NC}"
echo -e "  ${GREEN}➜ Nodes:     http://localhost:3000/nodes${NC}"
echo -e "  ${GREEN}➜ Channels:  http://localhost:3000/channels${NC}"
echo ""
echo -e "${YELLOW}Logs:${NC}"
echo -e "  tail -f /tmp/nsqlookupd.log"
echo -e "  tail -f /tmp/nsqd.log"
echo -e "  tail -f /tmp/nsqadmin.log"
echo -e "  tail -f /tmp/vite.log"
echo ""
echo -e "${YELLOW}Press Ctrl+C to stop all services${NC}"
echo ""

# Keep script running
wait

