#!/bin/bash
# MCPサーバーテストスクリプト

echo "Testing installed vibe-ticket MCP server..."
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}' | /Users/nwiizo/.cargo/bin/vibe-ticket mcp serve 2>&1 | grep -q '"serverInfo"' && echo "✅ Installed version works!" || echo "❌ Installed version failed"

echo ""
echo "Testing debug build MCP server..."
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}' | /Users/nwiizo/ghq/github.com/nwiizo/vibe-ticket/target/debug/vibe-ticket mcp serve 2>&1 | grep -q '"serverInfo"' && echo "✅ Debug version works!" || echo "❌ Debug version failed"
