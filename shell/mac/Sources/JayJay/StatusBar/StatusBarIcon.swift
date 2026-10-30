import AppKit
import SwiftUI

/// An SF Symbol, or one of the bundled template images such as the SSH host platform logos.
struct StatusBarIcon: View {
    @Environment(\.jayjayTypography) private var typography

    let name: String

    var body: some View {
        if NSImage(named: name) != nil {
            // Decorative: the item's text already names the state, and VoiceOver would read the asset name.
            Image(decorative: name)
                .resizable()
                .scaledToFit()
                .frame(width: 10 * typography.interfaceScale, height: 10 * typography.interfaceScale)
        } else {
            Image(systemName: name).jayjayFont(10)
        }
    }
}
