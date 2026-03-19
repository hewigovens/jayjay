// swift-tools-version:5.10
import PackageDescription

let package = Package(
    name: "JayJay",
    platforms: [.macOS(.v14)],
    products: [
        .library(name: "JayJayBindings", targets: ["JayJayBindings"]),
    ],
    targets: [
        .binaryTarget(
            name: "JayJayFFI",
            path: "JayJayFFI.xcframework"
        ),
        .target(
            name: "JayJayBindings",
            dependencies: ["JayJayFFI"],
            path: "Sources/JayJayBindings"
        ),
    ]
)
