#!/bin/bash
# install-polyglots.sh - Install dependencies for all polyglot workers
#
# Usage:
#   ./scripts/install-polyglots.sh [--python|--node|--go] [--dev]
#
# Options:
#   --python    Install Python dependencies only
#   --node      Install Node.js dependencies only
#   --go        Install Go dependencies only
#   --dev       Install development dependencies
#   --all       Install all dependencies (default)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Parse arguments
INSTALL_PYTHON=false
INSTALL_NODE=false
INSTALL_GO=false
INSTALL_DEV=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --python|-p)
            INSTALL_PYTHON=true
            shift
            ;;
        --node|-n)
            INSTALL_NODE=true
            shift
            ;;
        --go|-g)
            INSTALL_GO=true
            shift
            ;;
        --dev|-d)
            INSTALL_DEV=true
            shift
            ;;
        --all|-a)
            INSTALL_PYTHON=true
            INSTALL_NODE=true
            INSTALL_GO=true
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Default to all if nothing specified
if [ "$INSTALL_PYTHON" = false ] && [ "$INSTALL_NODE" = false ] && [ "$INSTALL_GO" = false ]; then
    INSTALL_PYTHON=true
    INSTALL_NODE=true
    INSTALL_GO=true
fi

cd "$PROJECT_ROOT"

# =============================================================================
# Python Workers
# =============================================================================

if [ "$INSTALL_PYTHON" = true ]; then
    log_info "Installing Python polyglot dependencies..."
    
    # Check Python version
    PYTHON_VERSION=$(python3 --version 2>&1 | cut -d' ' -f2 | cut -d'.' -f1,2)
    REQUIRED_VERSION="3.11"
    
    if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$PYTHON_VERSION" | sort -V | head -n1)" != "$REQUIRED_VERSION" ]; then
        log_warning "Python $REQUIRED_VERSION+ recommended, found $PYTHON_VERSION"
    fi
    
    # Install each Python worker
    for worker_dir in scrapers/py-*/; do
        if [ -f "$worker_dir/requirements.txt" ]; then
            worker_name=$(basename "$worker_dir")
            log_info "Installing $worker_name dependencies..."
            
            # Create virtual environment if it doesn't exist
            if [ ! -d "$worker_dir/.venv" ]; then
                python3 -m venv "$worker_dir/.venv"
                log_info "Created virtual environment for $worker_name"
            fi
            
            # Install dependencies
            "$worker_dir/.venv/bin/pip" install --upgrade pip --quiet
            "$worker_dir/.venv/bin/pip" install -r "$worker_dir/requirements.txt" --quiet
            
            # Install development dependencies
            if [ "$INSTALL_DEV" = true ] && [ -f "$worker_dir/pyproject.toml" ]; then
                "$worker_dir/.venv/bin/pip" install -e "$worker_dir[dev]" --quiet 2>/dev/null || true
            fi
            
            # Install Playwright browsers if needed
            if grep -q "playwright" "$worker_dir/requirements.txt" 2>/dev/null; then
                log_info "Installing Playwright browser for $worker_name..."
                "$worker_dir/.venv/bin/playwright" install chromium --quiet 2>/dev/null || \
                    log_warning "Playwright browser installation failed for $worker_name"
            fi
            
            log_success "$worker_name installed"
        fi
    done
    
    # Global Python dependencies for bridge
    if [ -f "requirements.txt" ]; then
        log_info "Installing global Python dependencies..."
        pip install -r requirements.txt --quiet
    fi
fi

# =============================================================================
# Node.js Workers
# =============================================================================

if [ "$INSTALL_NODE" = true ]; then
    log_info "Installing Node.js polyglot dependencies..."
    
    # Check Node.js version
    if command -v node &> /dev/null; then
        NODE_VERSION=$(node --version | cut -d'v' -f2 | cut -d'.' -f1)
        if [ "$NODE_VERSION" -lt 18 ]; then
            log_warning "Node.js 18+ recommended, found v$NODE_VERSION"
        fi
    else
        log_warning "Node.js not found, skipping Node.js workers"
        INSTALL_NODE=false
    fi
    
    if [ "$INSTALL_NODE" = true ]; then
        # Install each Node.js worker
        for worker_dir in scrapers/node-*/; do
            if [ -f "$worker_dir/package.json" ]; then
                worker_name=$(basename "$worker_dir")
                log_info "Installing $worker_name dependencies..."
                
                cd "$worker_dir"
                npm install --silent
                cd "$PROJECT_ROOT"
                
                log_success "$worker_name installed"
            fi
        done
    fi
fi

# =============================================================================
# Go Workers
# =============================================================================

if [ "$INSTALL_GO" = true ]; then
    log_info "Installing Go polyglot dependencies..."
    
    # Check Go version
    if command -v go &> /dev/null; then
        GO_VERSION=$(go version | cut -d' ' -f3 | cut -d'.' -f2)
        if [ "$GO_VERSION" -lt 21 ]; then
            log_warning "Go 1.21+ recommended"
        fi
    else
        log_warning "Go not found, skipping Go workers"
        INSTALL_GO=false
    fi
    
    if [ "$INSTALL_GO" = true ]; then
        # Install each Go worker
        for worker_dir in scrapers/go-*/; do
            if [ -f "$worker_dir/go.mod" ]; then
                worker_name=$(basename "$worker_dir")
                log_info "Installing $worker_name dependencies..."
                
                cd "$worker_dir"
                go mod download
                cd "$PROJECT_ROOT"
                
                log_success "$worker_name installed"
            fi
        done
    fi
fi

# =============================================================================
# Summary
# =============================================================================

echo ""
log_success "Polyglot installation complete!"
echo ""
log_info "Installed workers:"

for worker_dir in scrapers/*/; do
    if [ -f "$worker_dir/manifest.json" ]; then
        name=$(basename "$worker_dir")
        if [ "$name" != "external" ]; then
            runtime=$(python3 -c "import json; print(json.load(open('$worker_dir/manifest.json'))['runtime']['type'])" 2>/dev/null || echo "unknown")
            version=$(python3 -c "import json; print(json.load(open('$worker_dir/manifest.json'))['version'])" 2>/dev/null || echo "unknown")
            echo "   - $name ($runtime v$version)"
        fi
    fi
done

echo ""
log_info "Run './scripts/check-polyglots.sh' to verify installation"
