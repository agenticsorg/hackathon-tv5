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
