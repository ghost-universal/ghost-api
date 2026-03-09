#!/bin/bash
# check-polyglots.sh - Health check for all polyglot workers
#
# Usage:
#   ./scripts/check-polyglots.sh [--deep] [--json]
#
# Options:
#   --deep    Perform deep health check (may be slower)
#   --json    Output results as JSON

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Parse arguments
DEEP_CHECK=false
JSON_OUTPUT=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --deep|-d)
            DEEP_CHECK=true
            shift
            ;;
        --json|-j)
            JSON_OUTPUT=true
            shift
            ;;
        *)
            shift
            ;;
    esac
done

cd "$PROJECT_ROOT"

# Results array
declare -a RESULTS

check_worker() {
    local worker_dir=$1
    local worker_name=$(basename "$worker_dir")
    
    if [ ! -f "$worker_dir/manifest.json" ]; then
        return
    fi
    
    # Parse manifest
    local runtime=$(python3 -c "import json; print(json.load(open('$worker_dir/manifest.json'))['runtime']['type'])" 2>/dev/null || echo "unknown")
    local worker_id=$(python3 -c "import json; print(json.load(open('$worker_dir/manifest.json'))['id'])" 2>/dev/null || echo "$worker_name")
    
    local status="healthy"
    local message=""
    local checks=()
    
    case $runtime in
        python)
            # Check Python
            if [ -d "$worker_dir/.venv" ]; then
                checks+=("venv: ok")
            else
                checks+=("venv: missing")
                status="degraded"
                message="Virtual environment not found"
            fi
            
            # Check dependencies
            if [ -f "$worker_dir/requirements.txt" ]; then
                if "$worker_dir/.venv/bin/pip" check &>/dev/null; then
                    checks+=("deps: ok")
                else
                    checks+=("deps: issues")
                    status="degraded"
                    message="Dependency issues detected"
                fi
            fi
            
            # Check Playwright
            if grep -q "playwright" "$worker_dir/requirements.txt" 2>/dev/null; then
                if "$worker_dir/.venv/bin/python" -c "from playwright.async_api import async_playwright" 2>/dev/null; then
                    checks+=("playwright: ok")
                else
                    checks+=("playwright: missing")
                    status="unhealthy"
                    message="Playwright not installed"
                fi
            fi
            
            # Run Python health check
            if [ -f "$worker_dir/src/${worker_name//-/_}/worker.py" ]; then
                local health_result=$("$worker_dir/.venv/bin/python" -c "
import json
import sys
sys.path.insert(0, 'src')
from ${worker_name//-/_}.worker import ThreadsWorker, health_check
print(health_check())
" 2>/dev/null || echo '{"status":"error"}')
                
                if echo "$health_result" | grep -q '"status":"healthy"'; then
                    checks+=("health: ok")
                else
                    checks+=("health: degraded")
                    [ "$status" = "healthy" ] && status="degraded"
                fi
            fi
            ;;
            
        nodejs|node)
            # Check node_modules
            if [ -d "$worker_dir/node_modules" ]; then
                checks+=("node_modules: ok")
            else
                checks+=("node_modules: missing")
                status="unhealthy"
                message="Node modules not installed"
            fi
            
            # Check npm audit
            if [ "$DEEP_CHECK" = true ]; then
                cd "$worker_dir"
                if npm audit --audit-level=high &>/dev/null; then
                    checks+=("audit: ok")
                else
                    checks+=("audit: warnings")
                fi
                cd "$PROJECT_ROOT"
            fi
            ;;
            
        go)
            # Check Go module
            if [ -f "$worker_dir/go.sum" ]; then
                checks+=("modules: ok")
            else
                checks+=("modules: missing")
                status="unhealthy"
                message="Go modules not downloaded"
            fi
            
            # Check go vet
            if [ "$DEEP_CHECK" = true ]; then
                cd "$worker_dir"
                if go vet ./... &>/dev/null; then
                    checks+=("vet: ok")
                else
                    checks+=("vet: issues")
                    status="degraded"
                fi
                cd "$PROJECT_ROOT"
            fi
            ;;
            
        *)
            checks+=("runtime: unknown")
            status="unknown"
            message="Unknown runtime: $runtime"
            ;;
    esac
    
    # Store result
    if [ "$JSON_OUTPUT" = true ]; then
        local checks_json=$(printf '%s\n' "${checks[@]}" | jq -R . | jq -s .)
        RESULTS+=("{\"id\":\"$worker_id\",\"name\":\"$worker_name\",\"runtime\":\"$runtime\",\"status\":\"$status\",\"message\":\"$message\",\"checks\":$checks_json}")
    else
        local color=$GREEN
        [ "$status" = "degraded" ] && color=$YELLOW
        [ "$status" = "unhealthy" ] && color=$RED
        [ "$status" = "unknown" ] && color=$BLUE
        
        echo -e "${color}[$status]${NC} $worker_id ($runtime)"
        for check in "${checks[@]}"; do
            echo "       $check"
        done
        [ -n "$message" ] && echo "       Message: $message"
        echo ""
    fi
}

# Check external submodules
check_submodules() {
    if [ -d "scrapers/external" ]; then
        echo -e "${BLUE}[INFO]${NC} Checking external submodules..."
        
        for submodule in scrapers/external/*/; do
            if [ -d "$submodule/.git" ]; then
                local name=$(basename "$submodule")
                cd "$submodule"
                
                local current=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
                local upstream=$(git rev-parse --short '@{u}' 2>/dev/null || echo "unknown")
                
                cd "$PROJECT_ROOT"
                
                if [ "$current" = "$upstream" ]; then
                    echo -e "  ${GREEN}✓${NC} $name (up to date: $current)"
                else
                    echo -e "  ${YELLOW}!${NC} $name (current: $current, upstream: $upstream)"
                fi
            fi
        done
        echo ""
    fi
}

# Main
if [ "$JSON_OUTPUT" = false ]; then
    echo ""
    echo "================================"
    echo "  Polyglot Health Check Report"
    echo "================================"
    echo ""
fi

# Check each worker
for worker_dir in scrapers/*/; do
    if [ "$(basename "$worker_dir")" != "external" ]; then
        check_worker "$worker_dir"
    fi
done

# Check submodules
if [ "$JSON_OUTPUT" = false ]; then
    check_submodules
fi

# Output JSON if requested
if [ "$JSON_OUTPUT" = true ]; then
    echo "["
    for i in "${!RESULTS[@]}"; do
        echo -n "${RESULTS[$i]}"
        [ $i -lt $((${#RESULTS[@]} - 1)) ] && echo ","
    done
    echo ""
    echo "]"
fi

# Exit code based on health
if [ "$JSON_OUTPUT" = false ]; then
    echo "Run with --deep for more detailed checks"
    echo "Run with --json for machine-readable output"
fi
