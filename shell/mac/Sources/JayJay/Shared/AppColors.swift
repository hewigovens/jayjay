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
        Color(rgb: repo(scheme).graphLine)
    }

    static func workspace(_ scheme: ColorScheme) -> Color {
        Color(rgb: repo(scheme).workspace)
    }

    static func bookmark(_ scheme: ColorScheme) -> Color {
        Color(rgb: repo(scheme).bookmark)
    }

    static func commitIdPrefix(_ scheme: ColorScheme) -> Color {
        Color(rgb: repo(scheme).commitIdPrefix)
    }

    static func changeIdPrefix(_ scheme: ColorScheme) -> Color {
        Color(rgb: repo(scheme).changeIdPrefix)
    }

    private static let darkRepoColors = repoColors(isDark: true)
    private static let lightRepoColors = repoColors(isDark: false)

    private static func repo(_ scheme: ColorScheme) -> RepoColors {
        scheme == .dark ? darkRepoColors : lightRepoColors
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
