#!/bin/bash

# Build and test script for DeAI Compute Contract

echo "🔧 Building contract..."
cargo near build

echo "🧪 Running tests..."
cargo test

echo "✅ Contract build and test completed!"