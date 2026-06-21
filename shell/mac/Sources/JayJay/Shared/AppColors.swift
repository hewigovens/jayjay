import JayJayCore
import SwiftUI

/// App-specific brand colors. Values that both shells share live in core's `theme`
/// module so the SwiftUI and GPUI shells stay in sync.
enum AppColors {
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
