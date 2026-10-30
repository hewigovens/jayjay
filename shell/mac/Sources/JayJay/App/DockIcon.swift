import AppKit

enum DockIcon {
    static func install(variant: String?) {
        let dockTile = NSApplication.shared.dockTile
        DockIconView.install(on: dockTile, bundle: .main, variant: variant)
    }

    /// The plugin keeps drawing the tile after the app quits, so it has to hear about every change.
    static func update(variant: String?) {
        install(variant: variant)
        DistributedNotificationCenter.default().postNotificationName(
            DockIconView.variantDidChange,
            object: Bundle.main.bundlePath,
            userInfo: variant.map { [DockIconView.preferenceKey: $0] },
            deliverImmediately: true
        )
    }
}
