// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "JayJayDiffUI",
    platforms: [.macOS("26.0")],
    products: [
        .library(name: "JayJayDiffUI", targets: ["JayJayDiffUI"])
    ],
    dependencies: [
        .package(name: "JayJayCore", path: "../..") // Named, so the folder holding shell/mac/Package.swift can have any name.
    ],
    targets: [
        .target(
            name: "JayJayDiffUI",
            dependencies: [
                .product(name: "JayJayCore", package: "JayJayCore")
            ],
            path: "Sources/JayJayDiffUI",
            exclude: ["Resources/Mermaid"]
        ),
        .testTarget(
            name: "JayJayDiffUITests",
            dependencies: ["JayJayDiffUI"],
            path: "Tests/JayJayDiffUITests"
        )
    ]
)
