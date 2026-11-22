// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "JayJayReviewUI",
    platforms: [.iOS("26.0"), .macOS("26.0")],
    products: [
        .library(name: "JayJayReviewUI", targets: ["JayJayReviewUI"])
    ],
    dependencies: [
        .package(name: "JayJayCore", path: "../..") // Named, so the folder holding shell/mac/Package.swift can have any name.
    ],
    targets: [
        .target(
            name: "JayJayReviewUI",
            dependencies: [.product(name: "JayJayCore", package: "JayJayCore")],
            path: "Sources/JayJayReviewUI"
        ),
        .testTarget(name: "JayJayReviewUITests", dependencies: ["JayJayReviewUI"], path: "Tests/JayJayReviewUITests")
    ]
)
