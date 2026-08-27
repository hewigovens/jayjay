import SwiftUI

/// The scene launch presents; the others are suppressed.
enum LaunchScene: Equatable {
    case externalTool
    case onboarding(nextRepo: String?)
    case repo(String)
    case repoList

    init(isExternalTool: Bool, hasCompletedOnboarding: Bool, initialPath: String?) {
        let launchPath = initialPath.flatMap { $0.isEmpty ? nil : $0 }
        if isExternalTool {
            self = .externalTool
        } else if !hasCompletedOnboarding {
            self = .onboarding(nextRepo: launchPath)
        } else if let launchPath {
            self = .repo(launchPath)
        } else {
            self = .repoList
        }
    }

    var repoPath: String {
        if case let .repo(path) = self {
            path
        } else {
            ""
        }
    }

    var onboardingNextRepo: String? {
        if case let .onboarding(nextRepo) = self {
            nextRepo
        } else {
            nil
        }
    }

    func isPresented(by sceneID: String) -> Bool {
        switch self {
            case .externalTool: false
            case .onboarding: sceneID == AppWindows.onboarding
            case .repo: sceneID == AppWindows.repo
            case .repoList: sceneID == AppWindows.repoList
        }
    }

    var onboardingBehavior: SceneLaunchBehavior {
        behavior(presented: isOnboarding)
    }

    var repoListBehavior: SceneLaunchBehavior {
        behavior(presented: self == .repoList)
    }

    var repoBehavior: SceneLaunchBehavior {
        behavior(presented: !repoPath.isEmpty)
    }

    private var isOnboarding: Bool {
        if case .onboarding = self {
            true
        } else {
            false
        }
    }

    private func behavior(presented: Bool) -> SceneLaunchBehavior {
        presented ? .presented : .suppressed
    }
}
