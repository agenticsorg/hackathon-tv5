#!/bin/bash

# Nexus-UMMID Claude-Flow Swarm Launcher
# Uses Claude Code + Claude-Flow for reliable agent orchestration
# Replaces agentic-flow with Gemini (which had proxy issues)

set -e

echo "🧠 Nexus-UMMID Claude-Flow Swarm Launcher"
echo "=========================================="
echo "📦 Using: Claude-Flow v2.7.42 + Claude Code SDK"
echo "🤖 Models: Claude Sonnet/Opus via Anthropic API"
echo ""

# Check prerequisites
if ! command -v claude &> /dev/null; then
    echo "❌ Claude Code CLI not found. Install with: npm install -g @anthropic-ai/claude-code"
    exit 1
fi

if ! command -v npx &> /dev/null; then
    echo "❌ npx not found. Install Node.js 18+"
    exit 1
fi

echo "✅ Prerequisites verified"
echo ""

# Initialize directories
LOG_DIR="logs/claude-flow"
mkdir -p $LOG_DIR
mkdir -p apps/metadata-api/src/connectors

echo "📂 Log directory: $LOG_DIR"
echo ""

# Phase 2 Remaining Tasks
echo "🎯 Phase 2 Remaining Tasks:"
echo "  1. Platform Connectors (Netflix IMF, Amazon MEC, FAST)"
echo "  2. AgentDB Pattern Learning Integration"
echo "  3. RuVector Semantic Search Integration"
echo ""

# Option 1: Use Hive Mind for persistent swarm
echo "🐝 Starting Hive Mind Swarm..."
echo ""

# Create task file for the swarm
cat > /tmp/phase2-tasks.md << 'EOF'
# Phase 2 Remaining Tasks for Nexus-UMMID

## Context
- Project: /home/user/hackathon-tv5
- Metadata API: apps/metadata-api/
- Reference docs: mondweep/docs/MASTER_PRD.md, mondweep/docs/ENHANCED_IMPLEMENTATION_PLAN.md

## Task 1: Netflix IMF Connector
Create apps/metadata-api/src/connectors/netflix-imf.ts
- Implement Netflix IMF (Interoperable Master Format) package generation
- Include Dolby Vision metadata support
- Validate against Netflix content specs
- Export: generateIMFPackage(), validateNetflixContent()

## Task 2: Amazon MEC Connector
Create apps/metadata-api/src/connectors/amazon-mec.ts
- Implement Amazon Prime Video MEC (Media Entertainment Core) feed
- Support Amazon content requirements
- Export: generateMECFeed(), validateAmazonContent()

## Task 3: FAST Platform Connector
Create apps/metadata-api/src/connectors/fast-mrss.ts
- Implement MRSS (Media RSS) feed for FAST platforms (Pluto, Tubi, etc.)
- Support linear TV scheduling metadata
- Export: generateMRSSFeed(), validateFASTContent()

## Task 4: Connector Index
Create apps/metadata-api/src/connectors/index.ts
- Export all connectors with unified interface
- Create PlatformConnector base interface
- Add connector factory function

## Task 5: AgentDB Integration
Create apps/metadata-api/src/learning/agentdb-client.ts
- Integrate with AgentDB for pattern learning
- Store successful enrichment patterns
- Retrieve learned patterns for new content
- Support ReasoningBank memory

## Task 6: Tests for Connectors
Create apps/metadata-api/tests/connectors.test.ts
- Unit tests for all platform connectors
- Mock external APIs
- Test validation logic
EOF

echo "📋 Task file created at /tmp/phase2-tasks.md"
echo ""

# Launch the swarm using Claude-Flow hive-mind
echo "🚀 Launching Claude-Flow Hive Mind..."
echo ""
echo "Option A: Interactive Wizard (recommended)"
echo "  npx claude-flow@alpha hive-mind wizard"
echo ""
echo "Option B: Direct spawn with objective"
echo "  npx claude-flow@alpha hive-mind spawn 'Complete Phase 2: Platform connectors for Netflix, Amazon, FAST' --claude"
echo ""
echo "Option C: Use Claude Code Task tool (current session)"
echo "  Already using Claude Opus - agents can be spawned directly"
echo ""

# Show quick start for manual execution
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📖 Quick Reference Commands:"
echo ""
echo "# Check swarm status"
echo "npx claude-flow@alpha hive-mind status"
echo ""
echo "# Monitor memory"
echo "npx claude-flow@alpha memory search --namespace 'nexus-ummid'"
echo ""
echo "# SPARC modes available"
echo "npx claude-flow@alpha sparc modes"
echo ""
echo "# View logs"
echo "tail -f $LOG_DIR/*.log"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo ""
echo "✅ Claude-Flow swarm launcher ready!"
echo "🎯 Use the Claude Code Task tool in your current session to spawn agents"
