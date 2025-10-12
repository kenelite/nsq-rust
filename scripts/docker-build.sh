#!/bin/bash

# Docker build script for NSQ Rust
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
BUILD_ALL=true
BUILD_NSQD=false
BUILD_NSQLOOKUPD=false
BUILD_NSQADMIN=false
BUILD_TEST=false
PUSH_IMAGES=false
TAG="latest"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --nsqd)
            BUILD_NSQD=true
            BUILD_ALL=false
            shift
            ;;
        --nsqlookupd)
            BUILD_NSQLOOKUPD=true
            BUILD_ALL=false
            shift
            ;;
        --nsqadmin)
            BUILD_NSQADMIN=true
            BUILD_ALL=false
            shift
            ;;
        --test)
            BUILD_TEST=true
            BUILD_ALL=false
            shift
            ;;
        --push)
            PUSH_IMAGES=true
            shift
            ;;
        --tag)
            TAG="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --nsqd          Build NSQD image only"
            echo "  --nsqlookupd    Build NSQLookupd image only"
            echo "  --nsqadmin      Build NSQAdmin image only"
            echo "  --test          Build test image only"
            echo "  --push          Push images to registry"
            echo "  --tag TAG       Set image tag (default: latest)"
            echo "  --help          Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    print_error "Docker is not running. Please start Docker and try again."
    exit 1
fi

# Build UI first if needed
if [[ "$BUILD_ALL" == true || "$BUILD_NSQADMIN" == true ]]; then
    print_status "Building NSQAdmin UI..."
    cd nsqadmin-ui
    if [ ! -d "node_modules" ]; then
        print_status "Installing Node.js dependencies..."
        npm install
    fi
    print_status "Building UI..."
    npm run build
    cd ..
fi

# Function to build image
build_image() {
    local dockerfile=$1
    local image_name=$2
    local tag=$3
    
    print_status "Building $image_name:$tag..."
    
    if docker build -f "$dockerfile" -t "$image_name:$tag" .; then
        print_status "Successfully built $image_name:$tag"
    else
        print_error "Failed to build $image_name:$tag"
        exit 1
    fi
}

# Function to push image
push_image() {
    local image_name=$1
    local tag=$2
    
    print_status "Pushing $image_name:$tag..."
    
    if docker push "$image_name:$tag"; then
        print_status "Successfully pushed $image_name:$tag"
    else
        print_error "Failed to push $image_name:$tag"
        exit 1
    fi
}

# Build images based on options
if [[ "$BUILD_ALL" == true ]]; then
    build_image "Dockerfile.nsqd" "nsq-rust-nsqd" "$TAG"
    build_image "Dockerfile.nsqlookupd" "nsq-rust-nsqlookupd" "$TAG"
    build_image "Dockerfile.nsqadmin" "nsq-rust-nsqadmin" "$TAG"
    build_image "Dockerfile.test" "nsq-rust-test" "$TAG"
elif [[ "$BUILD_NSQD" == true ]]; then
    build_image "Dockerfile.nsqd" "nsq-rust-nsqd" "$TAG"
elif [[ "$BUILD_NSQLOOKUPD" == true ]]; then
    build_image "Dockerfile.nsqlookupd" "nsq-rust-nsqlookupd" "$TAG"
elif [[ "$BUILD_NSQADMIN" == true ]]; then
    build_image "Dockerfile.nsqadmin" "nsq-rust-nsqadmin" "$TAG"
elif [[ "$BUILD_TEST" == true ]]; then
    build_image "Dockerfile.test" "nsq-rust-test" "$TAG"
fi

# Push images if requested
if [[ "$PUSH_IMAGES" == true ]]; then
    if [[ "$BUILD_ALL" == true ]]; then
        push_image "nsq-rust-nsqd" "$TAG"
        push_image "nsq-rust-nsqlookupd" "$TAG"
        push_image "nsq-rust-nsqadmin" "$TAG"
        push_image "nsq-rust-test" "$TAG"
    elif [[ "$BUILD_NSQD" == true ]]; then
        push_image "nsq-rust-nsqd" "$TAG"
    elif [[ "$BUILD_NSQLOOKUPD" == true ]]; then
        push_image "nsq-rust-nsqlookupd" "$TAG"
    elif [[ "$BUILD_NSQADMIN" == true ]]; then
        push_image "nsq-rust-nsqadmin" "$TAG"
    elif [[ "$BUILD_TEST" == true ]]; then
        push_image "nsq-rust-test" "$TAG"
    fi
fi

print_status "Build completed successfully!"
