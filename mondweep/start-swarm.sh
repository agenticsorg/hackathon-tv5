#!/bin/bash

# Nexus-UMMID 13-Agent Swarm Launcher (Persistent Daemon Mode)
# Starts all 13 agents in parallel background loops to ensure they stay active.

# Load Env
export $(grep -v '^#' .env | xargs)

LOG_DIR="logs/swarm"
mkdir -p $LOG_DIR
echo "🚀 Launching Nexus-UMMID 13-Agent Swarm (Persistent Mode)..."
echo "📂 Logs directory: $LOG_DIR"
echo "⏳ Strategy: Batch launch + Auto-restart loops"

# verify API key
if [ -z "$GOOGLE_GEMINI_API_KEY" ]; then
    echo "❌ GOOGLE_GEMINI_API_KEY missing in .env"
    exit 1
fi

echo "🤖 Provider: Google Gemini (gemini-2.0-flash-exp)"

# Function to spawn agent in a generic loop
spawn_agent() {
    AGENT_TYPE=$1
    AGENT_ROLE=$2
    TASK=$3
    COLOR=$4
    PORT=$5
    
    echo -e "${COLOR}Starting Agent Daemon: $AGENT_ROLE ($AGENT_TYPE) on port $PORT...${NC}"
    
    # Run in a loop inside a background subshell
    (
        while true; do
            echo "[$(date)] Starting task cycle..." >> "$LOG_DIR/$AGENT_ROLE.log"
            
            PROXY_PORT=$PORT npx agentic-flow --agent $AGENT_TYPE \
                --provider gemini \
                --model gemini-2.0-flash-exp \
                --task "CONTEXT: You are a persistent agent in a swarm.
                ROLE: $AGENT_ROLE. 
                OBJECTIVE: $TASK
                
                INSTRUCTION: check the current state of the project, identify the next missing component, and IMPLEMENT IT. Do not just plan. Write code.
                If you have completed a step, move to the next one immediately.
                " \
                >> "$LOG_DIR/$AGENT_ROLE.log" 2>&1
            
            echo "[$(date)] Task cycle finished. Restarting in 10s..." >> "$LOG_DIR/$AGENT_ROLE.log"
            sleep 10
        done
    ) &
    
    echo $! > "$LOG_DIR/$AGENT_ROLE.pid"
}

# --- BATCH 1: CRITICAL CORE (0s) ---
spawn_agent "adaptive-coordinator" "adaptive-coordinator" "Orchestrate Phase 2. Review 'docs/13-AGENT_SWARM_STRATEGY.md' and ensure all teams are delivering artifacts." "\033[1;35m" 3000
spawn_agent "coder" "backend-dev" "Implement Metadata API. Check 'apps/metadata-api'. If missing, run 'npm init' and create 'src/index.ts'." "\033[0;34m" 3001
spawn_agent "coder" "database-architect" "Create Firestore schema. Check 'apps/metadata-api/src/db'. Create 'schema.ts' if missing." "\033[0;34m" 3002

echo "⏳ Batch 1 daemonized. Waiting 30s..."
sleep 30

# --- BATCH 2: API & QA (30s) ---
spawn_agent "coder" "api-docs" "Generate OpenAPI spec. Check 'apps/metadata-api/docs/openapi.yaml'." "\033[0;34m" 3003
spawn_agent "coder" "platform-integrator" "Create platform connectors. Check 'apps/metadata-api/src/connectors'." "\033[0;34m" 3004
spawn_agent "tdd-london-swarm" "tdd-london-swarm" "Write TDD tests. Check 'apps/metadata-api/tests'. Create a failing test for MetadataService." "\033[0;32m" 3005

echo "⏳ Batch 2 daemonized. Waiting 30s..."
sleep 30

# --- BATCH 3: QA & TESTING (60s) ---
spawn_agent "sparc-coder" "sparc-agent" "Validate architecture. implementation against 'docs/ENHANCED_IMPLEMENTATION_PLAN.md'." "\033[0;32m" 3006
spawn_agent "tester" "tester" "Create k6 load test script. Check 'apps/metadata-api/tests/load/k6-script.js'." "\033[0;32m" 3007
spawn_agent "production-validator" "production-validator" "Check security. Scan dependencies for vulnerabilities." "\033[0;32m" 3008

echo "⏳ Batch 3 daemonized. Waiting 30s..."
sleep 30

# --- BATCH 4: DEVOPS & DATA (90s) ---
spawn_agent "coder" "cicd-engineer" "Create GitHub Actions workflow. Check '.github/workflows/ci.yml'." "\033[0;33m" 3009
spawn_agent "planner" "release-manager" "Check release versioning. Update 'package.json' version if needed." "\033[0;33m" 3010
spawn_agent "planner" "system-architect" "Review scalability. Update 'docs/SCALABILITY.md' with 400M user strategy." "\033[0;33m" 3011
spawn_agent "coder" "data-scientist" "Generate synthetic data. Create 'data/synthetic_metadata.json' with 10 sample records." "\033[0;36m" 3012

echo "✅ All 13 agents daemonized!"
echo "📡 They will now run continuously."
echo "   ./monitor-swarm.sh"
