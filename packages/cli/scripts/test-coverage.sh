#!/bin/bash
# Generate test coverage report

set -e

echo "📊 Generating Test Coverage Report"
echo "=================================="
echo ""

# Check if tarpaulin is installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin
fi

# Generate coverage
echo "Running tests with coverage..."
cargo tarpaulin \
    --out Html \
    --out Xml \
    --output-dir coverage \
    --exclude-files tests/* \
    --timeout 300

# Display summary
echo ""
echo "✓ Coverage report generated!"
echo ""
echo "HTML report: coverage/tarpaulin-report.html"
echo "XML report: coverage/cobertura.xml"
echo ""

# Open HTML report if on macOS
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Opening HTML report..."
    open coverage/tarpaulin-report.html
fi
