// swift-tools-version: 5.7
import PackageDescription

let package = Package(
    name: "Coinswap",
    platforms: [
        .iOS(.v13),
        .macOS(.v10_15)
    ],
    products: [
        .library(
            name: "Coinswap",
            targets: ["Coinswap"]
        )
    ],
    targets: [
        .binaryTarget(
            name: "CoinswapFFI",
            path: "coinswap_ffi.xcframework"
        ),
        .target(
            name: "Coinswap",
            dependencies: ["CoinswapFFI"],
            path: "Sources/Coinswap"
        ),
        .testTarget(
            name: "CoinswapTests",
            dependencies: ["Coinswap"],
            path: "Tests/CoinswapTests"
        )
    ]
)
