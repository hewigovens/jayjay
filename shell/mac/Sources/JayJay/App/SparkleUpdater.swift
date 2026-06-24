import SwiftUI

#if !DEBUG
    import Sparkle
#endif

/// Sparkle is unavailable in DEBUG (no signed feed), so the controller is release-only and every accessor degrades to a no-op stub under DEBUG.
final class SparkleUpdater: ObservableObject {
    #if !DEBUG
        private let controller: SPUStandardUpdaterController
        private let feedDelegate = UpdaterFeedDelegate()
    #endif

    init() {
        #if !DEBUG
            controller = SPUStandardUpdaterController(
                startingUpdater: true,
                updaterDelegate: feedDelegate,
                userDriverDelegate: nil
            )
        #endif
    }

    func checkForUpdates() {
        #if !DEBUG
            controller.checkForUpdates(nil)
        #endif
    }

    var canCheckForUpdates: Bool {
        #if DEBUG
            false
        #else
            controller.updater.canCheckForUpdates
        #endif
    }

    var autoChecksEnabled: Bool {
        get {
            #if DEBUG
                false
            #else
                controller.updater.automaticallyChecksForUpdates
            #endif
        }
        set {
            #if DEBUG
                _ = newValue
            #else
                controller.updater.automaticallyChecksForUpdates = newValue
            #endif
        }
    }
}

// Routes the appcast through the telemetry worker only when the user opts in; when off, checks hit the direct appcast so no request reaches the worker.
#if !DEBUG
    private final class UpdaterFeedDelegate: NSObject, SPUUpdaterDelegate {
        private static let direct = "https://raw.githubusercontent.com/hewigovens/jayjay/main/docs/appcast.xml"
        private static let stats = "https://jayjay.hewigovens.workers.dev/appcast.xml"

        func feedURLString(for _: SPUUpdater) -> String? {
            let enabled = UserDefaults.standard.object(forKey: AppSettings.sendsAnonymousStatsKey) as? Bool ?? false
            return enabled ? Self.statsURL() : Self.direct
        }

        private static func statsURL() -> String {
            var components = URLComponents(string: stats)
            components?.queryItems = [
                URLQueryItem(name: "version", value: AppMetadata.shortVersion),
                URLQueryItem(name: "build", value: AppMetadata.buildNumber),
                URLQueryItem(name: "platform", value: "macos"),
                URLQueryItem(name: "os", value: "macos")
            ]
            return components?.string ?? stats
        }
    }
#endif
