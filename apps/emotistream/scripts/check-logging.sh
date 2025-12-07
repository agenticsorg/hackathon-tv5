#!/bin/bash

# Script to analyze console.log usage in the codebase
# Helps identify files that need logger migration

echo "============================================"
echo "EmotiStream Logging Analysis"
echo "============================================"
echo ""

# Count total console.log statements
echo "📊 Console Statement Counts:"
echo "----------------------------"
TOTAL=$(grep -roh "console\.[a-z]*" src/ 2>/dev/null | wc -l)
DEBUG=$(grep -roh "console\.debug" src/ 2>/dev/null | wc -l)
LOG=$(grep -roh "console\.log" src/ 2>/dev/null | wc -l)
INFO=$(grep -roh "console\.info" src/ 2>/dev/null | wc -l)
WARN=$(grep -roh "console\.warn" src/ 2>/dev/null | wc -l)
ERROR=$(grep -roh "console\.error" src/ 2>/dev/null | wc -l)

echo "Total console.* calls:    $TOTAL"
echo "  - console.log:          $LOG"
echo "  - console.debug:        $DEBUG"
echo "  - console.info:         $INFO"
echo "  - console.warn:         $WARN"
echo "  - console.error:        $ERROR"
echo ""

# Count files using logger
echo "📝 Logger Usage:"
echo "----------------------------"
LOGGER_IMPORTS=$(grep -rl "from.*logger" src/ 2>/dev/null | wc -l)
LOGGER_USES=$(grep -roh "logger\.\(debug\|info\|warn\|error\)" src/ 2>/dev/null | wc -l)

echo "Files importing logger:   $LOGGER_IMPORTS"
echo "Logger method calls:      $LOGGER_USES"
echo ""

# Top files with console.log
echo "🔥 Top 10 files with most console.* calls:"
echo "----------------------------"
grep -r "console\." src/ 2>/dev/null | cut -d: -f1 | sort | uniq -c | sort -rn | head -10 | while read count file; do
  echo "  $count - $file"
done
echo ""

# Files that should be migrated first
echo "⚠️  Priority files for migration (>10 console calls):"
echo "----------------------------"
grep -r "console\." src/ 2>/dev/null | cut -d: -f1 | sort | uniq -c | sort -rn | awk '$1 > 10 {print}' | while read count file; do
  echo "  $count - $file"
done
echo ""

# Check environment configuration
echo "🔧 Environment Configuration:"
echo "----------------------------"
if [ -f .env ]; then
  LOG_LEVEL=$(grep "LOG_LEVEL=" .env 2>/dev/null | cut -d= -f2)
  NODE_ENV=$(grep "NODE_ENV=" .env 2>/dev/null | cut -d= -f2)
  echo "LOG_LEVEL:               ${LOG_LEVEL:-not set}"
  echo "NODE_ENV:                ${NODE_ENV:-not set}"
else
  echo "⚠️  No .env file found"
fi
echo ""

# Migration progress
echo "📈 Migration Progress:"
echo "----------------------------"
if [ $TOTAL -gt 0 ]; then
  MIGRATED_PERCENT=$((LOGGER_USES * 100 / TOTAL))
  echo "Console.* calls:          $TOTAL"
  echo "Logger calls:             $LOGGER_USES"
  echo "Migration progress:       ~${MIGRATED_PERCENT}%"
else
  echo "✅ No console.* calls found!"
fi
echo ""

echo "============================================"
echo "💡 Next Steps:"
echo "============================================"
echo "1. Review the migration guide: docs/logging-migration-guide.md"
echo "2. Start with the priority files listed above"
echo "3. Replace console.log with logger.info or logger.debug"
echo "4. Replace console.error with logger.error"
echo "5. Run tests to ensure nothing breaks"
echo ""
echo "Example migration:"
echo "  Before: console.log('User logged in:', userId);"
echo "  After:  logger.info('User logged in', { userId });"
echo ""
