import Foundation
import JayJayCore
import JayJayDiffUI

enum AppWindows {
    static let onboarding = "onboarding-window"
    static let repoList = "repo-list-window"
    static let repo = "repo-window"
    static let about = "about-window"
    static let shortcuts = "shortcuts-window"
    static let overview = "overview-window"
    static let cloneRepository = "clone-repository-window"
    static let fileHistory = "file-history-window"
}

enum URLScheme {
    static let scheme = DeepLink.scheme
    static let hostOpen = DeepLink.Host.open
    static let paramPath = "path"
}

enum AppMetadata {
    static let appName = "JayJay"
    static let sponsorURL = URL(string: "https://github.com/sponsors/hewigovens")!
    static let githubURL = URL(string: "https://github.com/hewigovens/jayjay")!
    static let jjDocsURL = URL(string: "https://jj-vcs.github.io/jj/latest/")!
    static let revsetDocsURL = URL(string: "https://jj-vcs.github.io/jj/latest/revsets/")!

    static var shortVersion: String {
        Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "0.1.0"
    }

    static var buildNumber: String {
        Bundle.main.object(forInfoDictionaryKey: "CFBundleVersion") as? String ?? "1"
    }

    static var detailedVersionLabel: String {
        Localized.Settings.versionBuild(version: shortVersion, build: buildNumber)
    }

    static var compactVersionLabel: String {
        "\(shortVersion)(\(buildNumber))"
    }
}
