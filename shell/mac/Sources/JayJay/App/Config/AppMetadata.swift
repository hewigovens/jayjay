import Foundation
import JayJayDiffUI

enum AppWindows {
    static let main = "main-window"
    static let repo = "repo-window"
    static let about = "about-window"
    static let shortcuts = "shortcuts-window"
    static let welcome = "welcome-window"
}

enum URLScheme {
    static let scheme = DeepLink.scheme
    static let hostOpen = DeepLink.Host.open
    static let paramPath = "path"
}

enum AppMetadata {
    static let appName = "JayJay"
    static let tagline = "A native GUI for Jujutsu"
    static let sponsorURL = URL(string: "https://github.com/sponsors/hewigovens")!
    static let githubURL = URL(string: "https://github.com/hewigovens/jayjay")!

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
