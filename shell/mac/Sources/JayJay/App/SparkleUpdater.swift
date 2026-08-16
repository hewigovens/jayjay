import Sparkle
import JayJayCore
import SwiftUI

private final class UpdaterChannelDelegate: NSObject, SPUUpdaterDelegate {
    let includesBetaUpdates: () -> Bool

    init(includesBetaUpdates: @escaping () -> Bool) {
        self.includesBetaUpdates = includesBetaUpdates
    }

    func allowedChannels(for updater: SPUUpdater) -> Set<String> {
        includesBetaUpdates() ? ["beta"] : []
    }
}

final class SparkleUpdater: ObservableObject {
    @Published private(set) var canCheckForUpdates = false

    private let controller: SPUStandardUpdaterController
    private let channelDelegate: UpdaterChannelDelegate

    private var canCheckForUpdatesObservation: NSKeyValueObservation?

    init(includesBetaUpdates: @escaping () -> Bool = {
        parseUpdateChannel(value: UserDefaults.standard.string(forKey: AppSettings.updateChannelKey) ?? "") == .beta
    }) {
        channelDelegate = UpdaterChannelDelegate(includesBetaUpdates: includesBetaUpdates)
        // Debug builds must not offer production releases as updates to the development app.
        #if DEBUG
        let startsUpdater = false
        #else
        let startsUpdater = true
        #endif
        controller = SPUStandardUpdaterController(
            startingUpdater: startsUpdater,
            updaterDelegate: channelDelegate,
            userDriverDelegate: nil
        )
        canCheckForUpdatesObservation = controller.updater.observe(
            \.canCheckForUpdates,
            options: [.initial, .new]
        ) { [weak self] updater, _ in
            self?.canCheckForUpdates = updater.canCheckForUpdates
        }
    }

    func checkForUpdates() {
        controller.checkForUpdates(nil)
    }

    func channelSelectionChanged() {
        controller.updater.resetUpdateCycle()
    }

    var autoChecksEnabled: Bool {
        get { controller.updater.automaticallyChecksForUpdates }
        set { controller.updater.automaticallyChecksForUpdates = newValue }
    }
}
