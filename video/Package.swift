// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "Director",
    platforms: [.macOS(.v14)],
    products: [
        .executable(name: "director", targets: ["Director"])
    ],
    targets: [
        .executableTarget(name: "Director"),
        .testTarget(name: "DirectorTests", dependencies: ["Director"])
    ],
    swiftLanguageModes: [.v5]
)
