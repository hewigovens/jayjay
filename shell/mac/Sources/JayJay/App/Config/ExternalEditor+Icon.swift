import AppKit
import SwiftUI

extension AppSettings.ExternalEditor {
    /// The installed app's own icon, so the button names the editor it opens; terminal editors fall back to a generic glyph.
    @ViewBuilder
    var toolbarIcon: some View {
        if let bundleId, let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: bundleId) {
            Image(nsImage: NSWorkspace.shared.icon(forFile: url.path))
                .resizable()
                .frame(width: 18, height: 18)
        } else {
            Image(systemName: "curlybraces")
        }
    }
}
