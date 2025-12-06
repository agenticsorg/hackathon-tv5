#!/bin/bash
# Monitor Swarm Status

echo "📊 Nexus-UMMID Swarm Status"
echo "--------------------------------"

LOG_DIR="logs/swarm"

if [ ! -d "$LOG_DIR" ]; then
    echo "No swarm logs found. Run ./start-swarm.sh first."
    exit 1
fi

echo "Process ID | Agent Name           | Status   | Last Log"
echo "-----------|----------------------|----------|-------------------------"

for pid_file in $LOG_DIR/*.pid; do
    if [ -f "$pid_file" ]; then
        PID=$(cat $pid_file)
        AGENT=$(basename $pid_file .pid)
        
        if ps -p $PID > /dev/null; then
            STATUS="🟢 ACTIVE"
        else
            STATUS="🔴 STOPPED"
        fi
        
        LAST_LOG=$(tail -n 1 "$LOG_DIR/$AGENT.log" | cut -c 1-50)
        printf "%-10s | %-20s | %s | %s...\n" "$PID" "$AGENT" "$STATUS" "$LAST_LOG"
    fi
done

echo "--------------------------------"
echo "To follow logs: tail -f logs/swarm/*.log"
