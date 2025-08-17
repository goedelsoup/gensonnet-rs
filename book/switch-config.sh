#!/bin/bash

# Script to switch between development and production configurations

if [ "$1" = "dev" ]; then
    echo "Switching to development configuration..."
    cp config.dev.toml config.toml
    echo "✅ Development configuration activated"
    echo "Base URL: http://127.0.0.1:1111"
elif [ "$1" = "prod" ]; then
    echo "Switching to production configuration..."
    cp config.prod.toml config.toml
    echo "✅ Production configuration activated"
    echo "Base URL: https://goedelsoup.github.io/gensonnet-rs"
else
    echo "Usage: $0 {dev|prod}"
    echo ""
    echo "  dev  - Switch to development configuration"
    echo "  prod - Switch to production configuration"
    echo ""
    echo "Current configuration:"
    if grep -q "127.0.0.1" config.toml; then
        echo "  🟢 Development (http://127.0.0.1:1111)"
    else
        echo "  🔴 Production (https://goedelsoup.github.io/gensonnet-rs)"
    fi
fi
