#!/bin/bash

# VenvCleaner Build Script
# Builds the project with different feature configurations

set -e  # Exit on any error

PROJECT_NAME="venv_cleaner"
BUILD_DIR="target"
DIST_DIR="dist"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
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

# Function to build a specific configuration
build_config() {
    local config_name=$1
    local features=$2
    local target_suffix=$3

    print_status "Building $config_name configuration..."

    if [ -n "$features" ]; then
        cargo build --release --features "$features"
    else
        cargo build --release
    fi

    if [ $? -eq 0 ]; then
        # Copy binary to dist directory with appropriate name
        mkdir -p "$DIST_DIR"
        cp "$BUILD_DIR/release/$PROJECT_NAME" "$DIST_DIR/${PROJECT_NAME}${target_suffix}"
        print_success "$config_name build completed successfully"
    else
        print_error "$config_name build failed"
        return 1
    fi
}

# Function to run tests
run_tests() {
    print_status "Running tests..."
    cargo test
    if [ $? -eq 0 ]; then
        print_success "All tests passed"
    else
        print_error "Some tests failed"
        return 1
    fi
}

# Function to check code formatting and linting
check_code_quality() {
    print_status "Checking code formatting..."

    # Check if rustfmt is available
    if command -v rustfmt &> /dev/null; then
        cargo fmt --check
        if [ $? -ne 0 ]; then
            print_warning "Code formatting issues found. Run 'cargo fmt' to fix them."
        else
            print_success "Code formatting is correct"
        fi
    else
        print_warning "rustfmt not found, skipping format check"
    fi

    # Check if clippy is available
    if command -v cargo-clippy &> /dev/null; then
        print_status "Running clippy lints..."
        cargo clippy -- -D warnings
        if [ $? -eq 0 ]; then
            print_success "No clippy warnings found"
        else
            print_warning "Clippy warnings found"
        fi
    else
        print_warning "clippy not found, skipping lint check"
    fi
}

# Function to clean build artifacts
clean_build() {
    print_status "Cleaning build artifacts..."
    cargo clean
    rm -rf "$DIST_DIR"
    print_success "Clean completed"
}

# Function to show help
show_help() {
    echo "VenvCleaner Build Script"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  all         Build all configurations (default)"
    echo "  cli         Build CLI-only version"
    echo "  tui         Build with TUI support"
    echo "  gui         Build with GUI support"
    echo "  full        Build with all features"
    echo "  test        Run tests only"
    echo "  check       Run code quality checks"
    echo "  clean       Clean build artifacts"
    echo "  help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0           # Build all configurations"
    echo "  $0 cli       # Build CLI-only version"
    echo "  $0 test      # Run tests"
    echo "  $0 clean     # Clean build artifacts"
}

# Main script logic
main() {
    local command=${1:-all}

    case $command in
        "all")
            print_status "Building all configurations..."
            run_tests
            build_config "CLI" "" ""
            build_config "TUI" "tui" "-tui"
            build_config "GUI" "gui" "-gui"
            build_config "Full" "tui,gui" "-full"
            print_success "All builds completed successfully"
            echo ""
            print_status "Built binaries are available in the $DIST_DIR directory:"
            ls -la "$DIST_DIR"
            ;;
        "cli")
            run_tests
            build_config "CLI" "" ""
            ;;
        "tui")
            run_tests
            build_config "TUI" "tui" "-tui"
            ;;
        "gui")
            run_tests
            build_config "GUI" "gui" "-gui"
            ;;
        "full")
            run_tests
            build_config "Full" "tui,gui" "-full"
            ;;
        "test")
            run_tests
            ;;
        "check")
            check_code_quality
            ;;
        "clean")
            clean_build
            ;;
        "help"|"-h"|"--help")
            show_help
            ;;
        *)
            print_error "Unknown command: $command"
            echo ""
            show_help
            exit 1
            ;;
    esac
}

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    print_error "Cargo not found. Please install Rust and Cargo."
    exit 1
fi

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Cargo.toml not found. Please run this script from the project root directory."
    exit 1
fi

# Run main function with all arguments
main "$@"
