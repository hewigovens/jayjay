import Foundation

enum AppWindows {
    static let about = "about-window"
}

enum AppMetadata {
    static let appName = "JayJay"
    static let tagline = "A native GUI for Jujutsu"
    static let sponsorURL = URL(string: "https://github.com/sponsors/hewigovens")!

    static var shortVersion: String {
        Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "0.1.0"
    }

    static var buildNumber: String {
        Bundle.main.object(forInfoDictionaryKey: "CFBundleVersion") as? String ?? "1"
    }

    static var versionLabel: String {
        "Version \(shortVersion)"
    }

    static var detailedVersionLabel: String {
        "Version \(shortVersion) • Build \(buildNumber)"
    }

    static var compactVersionLabel: String {
        "\(shortVersion)(\(buildNumber))"
    }
}
