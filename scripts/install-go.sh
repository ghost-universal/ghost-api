#!/bin/bash
# Install Go scraper dependencies
#
# This script installs dependencies for the go-client scraper

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
GO_CLIENT_DIR="$ROOT_DIR/scrapers/go-client"

echo "Installing Go client dependencies..."

cd "$GO_CLIENT_DIR"

# Check for Go
if ! command -v go &> /dev/null; then
    echo "Error: go is not installed"
    exit 1
fi

# Download dependencies
go mod download

# Build the client
go build -o go-client .

echo "Go client dependencies installed successfully!"
