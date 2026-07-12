import AppKit

@objc(JayJayDockTilePlugin)
final class JayJayDockTilePlugin: NSObject, NSDockTilePlugIn {
    func setDockTile(_ dockTile: NSDockTile?) {
        guard let dockTile,
              let appBundle
        else { return }
        DockIconView.install(on: dockTile, bundle: appBundle)
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
