#!/bin/bash
# Validate Prometheus alert rules

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ALERTS_FILE="${SCRIPT_DIR}/alerts.yml"

echo "Validating Prometheus alert rules..."

# Check if promtool is available
if command -v promtool &> /dev/null; then
    echo "Running promtool validation..."
    promtool check rules "$ALERTS_FILE"
    echo "✓ Alert rules are valid"
else
    echo "Warning: promtool not found. Install Prometheus to validate rules."
    echo "Performing basic YAML syntax check..."

    # Basic YAML check
    if command -v yamllint &> /dev/null; then
        yamllint "$ALERTS_FILE"
        echo "✓ YAML syntax is valid"
    elif command -v python3 &> /dev/null; then
        python3 -c "import yaml; yaml.safe_load(open('$ALERTS_FILE'))"
        echo "✓ YAML syntax is valid"
    else
        echo "Warning: No YAML validator found. Install yamllint or python3-yaml."
    fi
fi

# Count alerts
ALERT_COUNT=$(grep -c "^  - alert:" "$ALERTS_FILE" || true)
GROUP_COUNT=$(grep -c "^  - name:" "$ALERTS_FILE" || true)

echo ""
echo "Statistics:"
echo "  Alert groups: $GROUP_COUNT"
echo "  Total alerts: $ALERT_COUNT"
echo ""

# List all alerts by group
echo "Alerts by group:"
awk '/^  - name:/ { group=$3; next } /^      - alert:/ { print "  " group ": " $3 }' "$ALERTS_FILE"

echo ""
echo "Validation complete!"
