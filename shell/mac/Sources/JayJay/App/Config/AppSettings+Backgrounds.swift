import JayJayDiffUI
import SwiftUI

extension AppSettings {
    var effectiveTintWindowWithWallpaper: Bool {
        tintWindowWithWallpaper && usesDefaultDarkDiffPalette
    }

    /// Wallpaper tinting is a dark-mode effect, and a non-default dark palette paints its own reading background.
    func tintsWindow(in scheme: ColorScheme) -> Bool {
        scheme == .dark && effectiveTintWindowWithWallpaper
    }

    func readingBackground(_ scheme: ColorScheme) -> Color {
        tintsWindow(in: scheme) ? .clear : AppColors.readingBackground(scheme, palettes: diffPalettes)
    }

    func navigationBackground(_ scheme: ColorScheme) -> Color {
        tintsWindow(in: scheme) ? .clear : AppColors.navigationBackground(scheme)
    }
}
