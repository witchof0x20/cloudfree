#!/usr/bin/env bash
# Test script for MCP server

set -euo pipefail

BASE_URL="${1:-http://localhost:8787}"
AUTH_TOKEN="${2:-test-token}"

echo "Testing MCP server at $BASE_URL"
echo ""

echo "1. Health check..."
curl -s "$BASE_URL/health"
echo -e "\n"

echo "2. Initialize..."
curl -s -X POST "$BASE_URL/mcp" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $AUTH_TOKEN" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | jq .
echo ""

echo "3. List tools..."
curl -s -X POST "$BASE_URL/mcp" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $AUTH_TOKEN" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' | jq .
echo ""

echo "4. List resources..."
curl -s -X POST "$BASE_URL/mcp" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $AUTH_TOKEN" \
  -d '{"jsonrpc":"2.0","id":3,"method":"resources/list"}' | jq .
echo ""

echo "5. Read neuron usage resource..."
curl -s -X POST "$BASE_URL/mcp" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $AUTH_TOKEN" \
  -d '{"jsonrpc":"2.0","id":4,"method":"resources/read","params":{"uri":"neuron://usage"}}' | jq .
echo ""

echo "All tests completed!"
