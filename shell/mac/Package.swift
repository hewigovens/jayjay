// swift-tools-version:5.10
import PackageDescription

let package = Package(
    name: "JayJay",
    platforms: [.macOS("26.0")],
    products: [
        .library(name: "JayJayCore", targets: ["JayJayCore"])
    ],
    dependencies: [
        .package(url: "https://github.com/sparkle-project/Sparkle", from: "2.7.0")
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
        )
    ]
)
