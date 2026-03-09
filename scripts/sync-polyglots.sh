#!/bin/bash
# sync-polyglots.sh - Sync git submodules for external scrapers
#
# Usage:
#   ./scripts/sync-polyglots.sh [--update] [--status]
#
# Options:
#   --update    Update submodules to latest upstream
#   --status    Show status of each submodule

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
EXTERNAL_DIR="$PROJECT_ROOT/scrapers/external"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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
UPDATE_MODE=false
STATUS_MODE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --update|-u)
            UPDATE_MODE=true
            shift
            ;;
        --status|-s)
            STATUS_MODE=true
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

cd "$PROJECT_ROOT"

# Check if .gitmodules exists
if [ ! -f ".gitmodules" ]; then
    log_warning "No .gitmodules file found"
    exit 0
fi

log_info "Syncing polyglot submodules..."

# Initialize and update submodules
git submodule update --init --recursive

if [ "$UPDATE_MODE" = true ]; then
    log_info "Updating submodules to latest upstream..."
    git submodule update --remote --merge
fi

# Check each external scraper
if [ -d "$EXTERNAL_DIR" ]; then
    for submodule in "$EXTERNAL_DIR"/*/; do
        if [ -d "$submodule/.git" ]; then
            name=$(basename "$submodule")
            cd "$submodule"
            
            current=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
            branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
            
            # Check for upstream changes
            git fetch --quiet 2>/dev/null || true
            upstream=$(git rev-parse --short '@{u}' 2>/dev/null || echo "unknown")
            
            if [ "$STATUS_MODE" = true ]; then
                echo ""
                log_info "📦 $name"
                echo "   Branch: $branch"
                echo "   Current: $current"
                echo "   Upstream: $upstream"
                
                if [ "$current" != "$upstream" ] && [ "$upstream" != "unknown" ]; then
                    log_warning "   Updates available! Run with --update to sync."
                else
                    log_success "   Up to date"
                fi
                
                # Show last commit info
                echo "   Last commit: $(git log -1 --format='%h - %s (%cr)' 2>/dev/null || echo 'unknown')"
            fi
            
            cd "$PROJECT_ROOT"
        fi
    done
fi

echo ""
log_success "Polyglot sync complete"

# List loaded workers
if [ "$STATUS_MODE" = true ]; then
    echo ""
    log_info "Loaded polyglot workers:"
    for worker_dir in "$PROJECT_ROOT"/scrapers/*/; do
        if [ -f "$worker_dir/manifest.json" ]; then
            name=$(basename "$worker_dir")
            if [ "$name" != "external" ]; then
                # Extract id from manifest
                worker_id=$(python3 -c "import json; print(json.load(open('$worker_dir/manifest.json'))['id'])" 2>/dev/null || echo "$name")
                echo "   - $worker_id ($name)"
            fi
        fi
    done
fi
