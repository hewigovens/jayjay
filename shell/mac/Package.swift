// swift-tools-version:5.10
import PackageDescription

let package = Package(
    name: "JayJay",
    platforms: [.macOS(.v14)],
    products: [
        .library(name: "JayJayCore", targets: ["JayJayCore"]),
    ],
    targets: [
        .binaryTarget(
            name: "JayJayFFI",
            path: "JayJayFFI.xcframework"
        ),
        .target(
            name: "JayJayCore",
            dependencies: ["JayJayFFI"],
            path: "Sources/JayJayCore"
        ),
    ]
)
