import JayJayDiffUI
import SwiftUI

extension AppSettings {
    var effectiveTintWindowWithWallpaper: Bool {
        tintWindowWithWallpaper && usesDefaultDarkColorTheme
    }

    /// Wallpaper tinting is a dark-mode effect, and a non-default dark theme paints its own reading background.
    func tintsWindow(in scheme: ColorScheme) -> Bool {
        scheme == .dark && effectiveTintWindowWithWallpaper
    }

    func readingBackground(_ scheme: ColorScheme) -> Color {
        tintsWindow(in: scheme) ? .clear : appThemes.colors(for: scheme).readingBackground
    }

    func navigationBackground(_ scheme: ColorScheme) -> Color {
        tintsWindow(in: scheme) ? .clear : appThemes.colors(for: scheme).navigationBackground
    }
}
