#!/bin/bash

# DeAI Node Client Setup Script

set -e

echo "🚀 DeAI Node Client Setup"
echo "========================="

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check if Python is installed
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 is not installed. Please install Python 3.8+ first."
    exit 1
fi

echo "✅ Rust and Python found"

# Create directories
echo "📁 Creating directories..."
mkdir -p models_cache
mkdir -p logs

# Install Python dependencies
echo "🐍 Installing Python dependencies..."
python3 -m pip install --user -r ai_engine/requirements.txt

# Build Rust binary
echo "🦀 Building Rust node client..."
cargo build --release

# Make scripts executable
chmod +x setup.sh
chmod +x ai_engine/ai_worker.py

# Copy default config if it doesn't exist
if [ ! -f "node_config.toml" ]; then
    echo "📋 Creating default configuration file..."
    echo "⚠️  Please edit node_config.toml with your specific values!"
fi

echo ""
echo "🎉 Setup completed successfully!"
echo ""
echo "Next steps:"
echo "1. Edit node_config.toml with your node details"
echo "2. Run: ./target/release/deai-node-client register"
echo "3. Run: ./target/release/deai-node-client start"
echo ""
echo "For help: ./target/release/deai-node-client --help"