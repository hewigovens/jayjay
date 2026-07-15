import SwiftUI

#if !DEBUG
    import Sparkle
#endif

/// Sparkle is unavailable in DEBUG (no signed feed), so the controller is release-only and every accessor degrades to a no-op stub under DEBUG.
final class SparkleUpdater: ObservableObject {
    #if !DEBUG
        private let controller: SPUStandardUpdaterController
    #endif

    init() {
        #if !DEBUG
            controller = SPUStandardUpdaterController(
                startingUpdater: true,
                updaterDelegate: nil,
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
