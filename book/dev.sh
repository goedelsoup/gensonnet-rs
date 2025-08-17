#!/bin/bash

# Development script for Gensonnet-rs documentation site
# This script sets the correct base URL for local development

echo "Starting Gensonnet-rs documentation development server..."
echo "Base URL will be: http://127.0.0.1:1111"
echo ""

# Set the base URL for local development
export BASE_URL="http://127.0.0.1:1111"

# Start the Zola development server
zola serve --interface 127.0.0.1 --port 1111 --open
