#!/bin/bash
#
# integrate_ruvector_xcode.sh
# Automates Xcode integration steps for Ruvector
#

set -e

PROJECT_DIR="/Volumes/black box/github/pkm/hackathon-tv5/apps/vibecheck-ios"
cd "$PROJECT_DIR"

echo "🔧 Ruvector Xcode Integration"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Step 1: Verify files exist
echo "📦 Step 1: Verifying files..."
if [ ! -f "VibeCheck/Engine/RuvectorBridge.swift" ]; then
    echo "❌ RuvectorBridge.swift not found"
    exit 1
fi

if [ ! -f "VibeCheck/Resources/ruvector.wasm" ]; then
    echo "❌ ruvector.wasm not found"
    exit 1
fi

if [ ! -f "VibeCheckTests/RuvectorBridgeTests.swift" ]; then
    echo "❌ RuvectorBridgeTests.swift not found"
    exit 1
fi

echo "✅ All files present"
echo ""

# Step 2: Add Package.swift for WasmKit dependency
echo "📦 Step 2: Creating Package.swift for WasmKit..."

cat > Package.swift << 'PKGEOF'
// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "VibeCheck",
    platforms: [
        .iOS(.v17)
    ],
    dependencies: [
        .package(url: "https://github.com/swiftwasm/WasmKit", from: "0.1.0")
    ],
    targets: [
        .target(
            name: "VibeCheck",
            dependencies: [
                .product(name: "WasmKit", package: "WasmKit")
            ]
        )
    ]
)
PKGEOF

echo "✅ Package.swift created"
echo ""

# Step 3: Instructions for manual Xcode steps
echo "📋 Step 3: Manual Xcode Steps Required"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Please complete these steps in Xcode:"
echo ""
echo "1️⃣  Add WasmKit Package:"
echo "   • File → Add Package Dependencies"
echo "   • URL: https://github.com/swiftwasm/WasmKit"
echo "   • Version: 0.1.0 or later"
echo "   • Add to target: VibeCheck"
echo ""
echo "2️⃣  Add Source Files:"
echo "   • Right-click 'VibeCheck/Engine' folder"
echo "   • Add Files → Select 'RuvectorBridge.swift'"
echo "   • ✅ Copy items if needed"
echo "   • ✅ Add to targets: VibeCheck"
echo ""
echo "   • Right-click 'VibeCheckTests' folder"
echo "   • Add Files → Select 'RuvectorBridgeTests.swift'"
echo "   • ✅ Add to targets: VibeCheckTests"
echo ""
echo "3️⃣  Add WASM Resource:"
echo "   • Right-click 'VibeCheck/Resources' folder"
echo "   • Add Files → Select 'ruvector.wasm'"
echo "   • ✅ Copy items if needed"
echo "   • ✅ Add to targets: VibeCheck"
echo ""
echo "4️⃣  Build & Test:"
echo "   • Press ⌘+B to build"
echo "   • Press ⌘+U to run tests"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Setup complete!"
echo "   Next: Open VibeCheck.xcodeproj and follow steps above"
