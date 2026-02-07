#!/usr/bin/env bash
set -e

# Start dev server in background
result/bin/dev --local &
PID=$!

# Wait for server to start
sleep 15

# Test health endpoint
echo "=== Testing /health ==="
curl -s http://localhost:8787/health
echo -e "\n"

# Test MCP initialize
echo "=== Testing MCP initialize ==="
curl -s -X POST http://localhost:8787/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize"}'
echo -e "\n"

# Test tools/list
echo "=== Testing tools/list ==="
curl -s -X POST http://localhost:8787/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/list"}'
echo -e "\n"

# Cleanup
kill $PID
