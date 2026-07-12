import AppKit

enum DockIcon {
    static func install() {
        let dockTile = NSApplication.shared.dockTile
        DockIconView.install(on: dockTile, bundle: .main)
    }
}
