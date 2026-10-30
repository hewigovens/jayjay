import AppKit

@objc(JayJayDockTilePlugin)
final class JayJayDockTilePlugin: NSObject, NSDockTilePlugIn {
    private var variantObserver: NSObjectProtocol?

    func setDockTile(_ dockTile: NSDockTile?) {
        if let variantObserver {
            DistributedNotificationCenter.default().removeObserver(variantObserver)
            self.variantObserver = nil
        }
        guard let dockTile,
              let appBundle
        else { return }
        let variant = UserDefaults(suiteName: "dev.hewig.jayjay")?.string(forKey: DockIconView.preferenceKey)
        DockIconView.install(on: dockTile, bundle: appBundle, variant: variant)
        // Debug and test builds share these defaults, so only the plugin's own app may change its tile.
        variantObserver = DistributedNotificationCenter.default().addObserver(
            forName: DockIconView.variantDidChange,
            object: appBundle.bundlePath,
            queue: .main
        ) { [weak dockTile] notification in
            guard let dockTile else { return }
            let variant = notification.userInfo?[DockIconView.preferenceKey] as? String
            DockIconView.install(on: dockTile, bundle: appBundle, variant: variant)
        }
    }

    private var appBundle: Bundle? {
        let pluginURL = Bundle(for: type(of: self)).bundleURL
        let appURL = pluginURL
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
        return Bundle(url: appURL)
    }
}
