#!/bin/bash

# Production build script for Gensonnet-rs documentation site
# This script sets the correct base URL for production deployment

# Default production URL - can be overridden by setting BASE_URL environment variable
PRODUCTION_URL="${BASE_URL:-https://goedelsoup.github.io/gensonnet-rs}"

echo "Building Gensonnet-rs documentation for production..."
echo "Base URL will be: $PRODUCTION_URL"
echo ""

# Set the base URL for production
export BASE_URL="$PRODUCTION_URL"

# Build the site
zola build

echo ""
echo "Build complete! Site generated in 'public' directory"
echo "Base URL: $PRODUCTION_URL"
