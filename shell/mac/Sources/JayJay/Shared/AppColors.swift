import JayJayCore
import SwiftUI

/// App-specific brand colors. Values that both shells share live in core's `theme`
/// module so the SwiftUI and GPUI shells stay in sync.
enum AppColors {
    static let commitIdPrefix = Color(nsColor: NSColor(name: nil) { appearance in
        appearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
            ? NSColor(srgbRed: 102 / 255, green: 181 / 255, blue: 1, alpha: 1)
            : NSColor(srgbRed: 0, green: 99 / 255, blue: 197 / 255, alpha: 1)
    })

    static let contentBackground = Color(nsColor: NSColor(name: nil) { appearance in
        appearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
            ? NSColor(srgbRed: 8 / 255, green: 13 / 255, blue: 21 / 255, alpha: 1)
            : .textBackgroundColor
    })

    static let navigationBackground = Color(nsColor: NSColor(name: nil) { appearance in
        appearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
            ? NSColor(srgbRed: 16 / 255, green: 24 / 255, blue: 34 / 255, alpha: 1)
            : .controlBackgroundColor
    })

    /// The shortest-unique change-id / commit-id prefix highlight — sourced from
    /// core so it matches the GPUI shell exactly.
    static func changeIdPrefix(_ scheme: ColorScheme) -> Color {
        Color(rgb: changeIdPrefixColor(isDark: scheme == .dark))
    }
}

private extension Color {
    /// From a packed `0xRRGGBB` value (core's shared design tokens are `u32`).
    init(rgb: UInt32) {
        self.init(
            red: Double((rgb >> 16) & 0xFF) / 255.0,
            green: Double((rgb >> 8) & 0xFF) / 255.0,
            blue: Double(rgb & 0xFF) / 255.0
        )
    }
}
