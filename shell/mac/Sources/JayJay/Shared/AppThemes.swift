import JayJayCore
import SwiftUI

struct AppThemes: Equatable {
    var light: ColorTheme
    var dark: ColorTheme

    static let standard = AppThemes(
        light: colorTheme(id: AppSettings.defaultLightColorThemeId, isDark: false),
        dark: colorTheme(id: AppSettings.defaultDarkColorThemeId, isDark: true)
    )

    func colors(for scheme: ColorScheme) -> AppColors {
        let dark = scheme == .dark
        let resolved = dark ? self.dark : light
        return AppColors(
            theme: resolved,
            isDefault: resolved.id == (dark ? AppSettings.defaultDarkColorThemeId : AppSettings.defaultLightColorThemeId)
        )
    }
}

private struct AppThemesKey: EnvironmentKey {
    static let defaultValue = AppThemes.standard
}

extension EnvironmentValues {
    var appThemes: AppThemes {
        get { self[AppThemesKey.self] }
        set { self[AppThemesKey.self] = newValue }
    }
}
