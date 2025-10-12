#!/bin/bash

# Docker run script for NSQ Rust
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Default values
ENVIRONMENT="dev"
ACTION="up"
DETACHED=true
BUILD=false
CLEANUP=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --env)
            ENVIRONMENT="$2"
            shift 2
            ;;
        --action)
            ACTION="$2"
            shift 2
            ;;
        --build)
            BUILD=true
            shift
            ;;
        --cleanup)
            CLEANUP=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --env ENV       Environment (dev, prod, test) [default: dev]"
            echo "  --action ACTION Action (up, down, restart, logs) [default: up]"
            echo "  --build         Build images before running"
            echo "  --cleanup       Clean up containers and volumes"
            echo "  --help          Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0 --env dev --action up"
            echo "  $0 --env prod --action up --build"
            echo "  $0 --env test --action down --cleanup"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Validate environment
if [[ "$ENVIRONMENT" != "dev" && "$ENVIRONMENT" != "prod" && "$ENVIRONMENT" != "test" ]]; then
    print_error "Invalid environment: $ENVIRONMENT. Must be dev, prod, or test."
    exit 1
fi

# Validate action
if [[ "$ACTION" != "up" && "$ACTION" != "down" && "$ACTION" != "restart" && "$ACTION" != "logs" ]]; then
    print_error "Invalid action: $ACTION. Must be up, down, restart, or logs."
    exit 1
fi

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    print_error "Docker is not running. Please start Docker and try again."
    exit 1
fi

# Check if Docker Compose is available
if ! command -v docker-compose &> /dev/null; then
    print_error "Docker Compose is not installed. Please install Docker Compose and try again."
    exit 1
fi

# Determine compose file
COMPOSE_FILE="docker-compose.yml"
if [[ "$ENVIRONMENT" == "dev" ]]; then
    COMPOSE_FILE="docker-compose.dev.yml"
elif [[ "$ENVIRONMENT" == "prod" ]]; then
    COMPOSE_FILE="docker-compose.prod.yml"
elif [[ "$ENVIRONMENT" == "test" ]]; then
    COMPOSE_FILE="docker-compose.test.yml"
fi

# Check if compose file exists
if [[ ! -f "$COMPOSE_FILE" ]]; then
    print_error "Compose file not found: $COMPOSE_FILE"
    exit 1
fi

# Function to run docker-compose command
run_compose() {
    local cmd=$1
    local args=$2
    
    print_status "Running: docker-compose -f $COMPOSE_FILE $cmd $args"
    
    if docker-compose -f "$COMPOSE_FILE" $cmd $args; then
        print_status "Command completed successfully"
    else
        print_error "Command failed"
        exit 1
    fi
}

# Build images if requested
if [[ "$BUILD" == true ]]; then
    print_status "Building images..."
    run_compose "build"
fi

# Handle different actions
case $ACTION in
    "up")
        if [[ "$DETACHED" == true ]]; then
            run_compose "up -d"
        else
            run_compose "up"
        fi
        
        # Show status
        print_status "Container status:"
        docker-compose -f "$COMPOSE_FILE" ps
        
        # Show logs for a few seconds
        print_status "Recent logs:"
        timeout 5 docker-compose -f "$COMPOSE_FILE" logs --tail=10 || true
        ;;
        
    "down")
        run_compose "down"
        
        if [[ "$CLEANUP" == true ]]; then
            print_status "Cleaning up volumes..."
            run_compose "down -v"
        fi
        ;;
        
    "restart")
        run_compose "restart"
        ;;
        
    "logs")
        run_compose "logs -f"
        ;;
esac

# Show final status
if [[ "$ACTION" == "up" ]]; then
    print_status "NSQ Rust is running in $ENVIRONMENT environment"
    print_status "Access points:"
    
    if [[ "$ENVIRONMENT" == "dev" ]]; then
        echo "  NSQD:      http://localhost:4151"
        echo "  NSQLookupd: http://localhost:4161"
        echo "  NSQAdmin:   http://localhost:4171"
    elif [[ "$ENVIRONMENT" == "prod" ]]; then
        echo "  NSQD:      http://localhost:4151, 4153, 4155"
        echo "  NSQLookupd: http://localhost:4161, 4163, 4165"
        echo "  NSQAdmin:   http://localhost:4171"
    elif [[ "$ENVIRONMENT" == "test" ]]; then
        echo "  NSQD:      http://localhost:4151"
        echo "  NSQLookupd: http://localhost:4161"
        echo "  NSQAdmin:   http://localhost:4171"
    fi
    
    print_status "Use '$0 --env $ENVIRONMENT --action logs' to view logs"
    print_status "Use '$0 --env $ENVIRONMENT --action down' to stop"
fi
