#!/bin/bash
# Install Node.js scraper dependencies
#
# This script installs dependencies for the node-agent scraper

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
NODE_AGENT_DIR="$ROOT_DIR/scrapers/node-agent"

echo "Installing Node.js agent dependencies..."

cd "$NODE_AGENT_DIR"

# Check for npm
if ! command -v npm &> /dev/null; then
    echo "Error: npm is not installed"
    exit 1
fi

# Install dependencies
npm install

# Build TypeScript
npm run build

echo "Node.js agent dependencies installed successfully!"
