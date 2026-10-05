import SwiftUI

extension AppSettings {
    func readingBackground(_ scheme: ColorScheme) -> Color {
        tintWindowWithWallpaper ? .clear : AppColors.readingBackground(scheme)
    }

    func navigationBackground(_ scheme: ColorScheme) -> Color {
        tintWindowWithWallpaper ? .clear : AppColors.navigationBackground(scheme)
    }
}
