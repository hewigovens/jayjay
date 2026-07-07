// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "JayJayDiffUI",
    platforms: [.macOS("15.0")],
    products: [
        .library(name: "JayJayDiffUI", targets: ["JayJayDiffUI"])
    ],
    dependencies: [
        .package(path: "../.."), // JayJayCore from shell/mac/Package.swift
        .package(url: "https://github.com/gonzalezreal/textual", from: "0.5.0")
    ],
    targets: [
        .target(
            name: "JayJayDiffUI",
            dependencies: [
                .product(name: "JayJayCore", package: "mac"),
                .product(name: "Textual", package: "textual")
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
