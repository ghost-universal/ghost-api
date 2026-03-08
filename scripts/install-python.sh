#!/bin/bash
# Install Python scraper dependencies
#
# This script installs dependencies for the py-stealth scraper

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
PY_STEALTH_DIR="$ROOT_DIR/scrapers/py-stealth"

echo "Installing Python stealth scraper dependencies..."

cd "$PY_STEALTH_DIR"

# Check for Python
if ! command -v python3 &> /dev/null; then
    echo "Error: python3 is not installed"
    exit 1
fi

# Check for pip
if ! command -v pip3 &> /dev/null; then
    echo "Error: pip3 is not installed"
    exit 1
fi

# Create virtual environment if it doesn't exist
if [ ! -d "venv" ]; then
    echo "Creating virtual environment..."
    python3 -m venv venv
fi

# Activate virtual environment
source venv/bin/activate

# Install dependencies
pip3 install -e .

# Install Playwright browsers
if command -v playwright &> /dev/null; then
    echo "Installing Playwright browsers..."
    playwright install chromium
fi

echo "Python stealth scraper dependencies installed successfully!"
