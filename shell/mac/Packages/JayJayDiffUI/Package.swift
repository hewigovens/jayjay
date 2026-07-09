// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "JayJayDiffUI",
    platforms: [.macOS("15.0")],
    products: [
        .library(name: "JayJayDiffUI", targets: ["JayJayDiffUI"])
    ],
    dependencies: [
        .package(path: "../..") // JayJayCore from shell/mac/Package.swift
    ],
    targets: [
        .target(
            name: "JayJayDiffUI",
            dependencies: [
                .product(name: "JayJayCore", package: "mac")
            ],
            path: "Sources/JayJayDiffUI"
        ),
        .testTarget(
            name: "JayJayDiffUITests",
            dependencies: ["JayJayDiffUI"],
            path: "Tests/JayJayDiffUITests"
        )
    ]
)
