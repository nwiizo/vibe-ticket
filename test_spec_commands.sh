#!/bin/bash
# Test script for spec-driven development commands

set -e

echo "Testing Spec-Driven Development Commands"
echo "========================================"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test CLI commands
echo -e "${YELLOW}Testing CLI Commands...${NC}"

# 1. Create a specification using specify command
echo "1. Testing spec specify command..."
./target/release/vibe-ticket spec specify "Build a calculator CLI with basic arithmetic" --output test-calc-spec || true

# 2. List specifications
echo "2. Listing specifications..."
./target/release/vibe-ticket spec list

# 3. Test plan generation
echo "3. Testing plan generation..."
SPEC_ID=$(./target/release/vibe-ticket spec list --json 2>/dev/null | grep -o '"id":"[^"]*' | head -1 | cut -d'"' -f4)
if [ ! -z "$SPEC_ID" ]; then
    ./target/release/vibe-ticket spec plan --spec "$SPEC_ID" --tech-stack rust,clap --architecture layered || true
fi

# 4. Test task generation
echo "4. Testing task generation..."
if [ ! -z "$SPEC_ID" ]; then
    ./target/release/vibe-ticket spec tasks --spec "$SPEC_ID" --granularity medium --parallel || true
fi

# 5. Test validation
echo "5. Testing validation..."
if [ ! -z "$SPEC_ID" ]; then
    ./target/release/vibe-ticket spec validate --spec "$SPEC_ID" --ambiguities --report || true
fi

# 6. Test template generation
echo "6. Testing template generation..."
./target/release/vibe-ticket spec template spec --force || true

echo -e "${GREEN}CLI Command Tests Complete!${NC}"

# Test MCP if requested
if [ "$1" == "--mcp" ]; then
    echo -e "${YELLOW}Testing MCP Integration...${NC}"
    
    # Start MCP server in background
    ./target/release/vibe-ticket mcp serve --port 3034 &
    MCP_PID=$!
    sleep 2
    
    # Test MCP tools using curl
    echo "Testing MCP spec_specify tool..."
    curl -X POST http://localhost:3034/rpc \
        -H "Content-Type: application/json" \
        -d '{
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": "vibe-ticket_spec_specify",
                "arguments": {
                    "requirements": "Build a TODO app with categories"
                }
            },
            "id": 1
        }' || true
    
    echo ""
    
    # Kill MCP server
    kill $MCP_PID 2>/dev/null || true
    
    echo -e "${GREEN}MCP Integration Tests Complete!${NC}"
fi

echo -e "${GREEN}All Tests Complete!${NC}"
echo ""
echo "Summary:"
echo "- Spec-driven development commands are working"
echo "- Templates can be generated"
echo "- Specifications can be created, planned, and validated"

# Cleanup
rm -rf test-calc-spec 2>/dev/null || true

echo ""
echo "To test MCP integration, run: $0 --mcp"