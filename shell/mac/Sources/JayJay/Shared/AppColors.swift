import JayJayCore
import SwiftUI

/// App-specific brand colors. Values that both shells share live in core's `theme`
/// module so the SwiftUI and GPUI shells stay in sync.
enum AppColors {
    static func readingBackground(_ scheme: ColorScheme) -> Color {
        Color(rgb: diffThemeColors(isDark: scheme == .dark).contextBg)
    }

    static func navigationBackground(_ scheme: ColorScheme) -> Color {
        scheme == .dark ? Color(rgb: 0x1B2330) : Color(nsColor: .controlBackgroundColor)
    }

    static func graphLine(_ scheme: ColorScheme) -> Color {
        Color(rgb: scheme == .dark ? 0x3478F6 : 0x5982B8)
    }

    static func workspace(_ scheme: ColorScheme) -> Color {
        Color(rgb: scheme == .dark ? 0x42D96B : 0x128A3E)
    }

    static func bookmark(_ scheme: ColorScheme) -> Color {
        Color(rgb: scheme == .dark ? 0xD86BF2 : 0x9635C9)
    }

    static func commitIdPrefix(_ scheme: ColorScheme) -> Color {
        Color(rgb: scheme == .dark ? 0x78B7FF : 0x175CD3)
    }

    /// The shortest-unique change-id prefix highlight is shared with the GPUI shell.
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
