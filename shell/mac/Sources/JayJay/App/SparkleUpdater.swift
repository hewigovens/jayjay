import Sparkle
import SwiftUI

final class SparkleUpdater: ObservableObject {
    private let controller: SPUStandardUpdaterController
    private let feedDelegate = UpdaterFeedDelegate()

    init() {
        controller = SPUStandardUpdaterController(
            startingUpdater: true,
            updaterDelegate: feedDelegate,
            userDriverDelegate: nil
        )
    }

    func checkForUpdates() {
        controller.checkForUpdates(nil)
    }

    var canCheckForUpdates: Bool {
        controller.updater.canCheckForUpdates
    }

    var autoChecksEnabled: Bool {
        get { controller.updater.automaticallyChecksForUpdates }
        set { controller.updater.automaticallyChecksForUpdates = newValue }
    }
}

/// Routes the appcast through the telemetry worker only when the user opts in;
/// when off, checks hit the direct appcast so no request reaches the worker.
private final class UpdaterFeedDelegate: NSObject, SPUUpdaterDelegate {
    private static let direct = "https://raw.githubusercontent.com/hewigovens/jayjay/main/docs/appcast.xml"
    private static let stats = "https://jayjay.hewigovens.workers.dev/appcast.xml"

    func feedURLString(for _: SPUUpdater) -> String? {
        let enabled = UserDefaults.standard.object(forKey: AppSettings.sendsAnonymousStatsKey) as? Bool ?? false
        return enabled ? Self.stats : Self.direct
    }
}
