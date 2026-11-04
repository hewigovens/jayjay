import JayJayCore
import JayJayDiffUI

extension AppSettings {
    static let defaultLightColorThemeId = defaultColorThemeId(isDark: false)
    static let defaultDarkColorThemeId = defaultColorThemeId(isDark: true)

    var appThemes: AppThemes {
        AppThemes(light: lightColorTheme, dark: darkColorTheme)
    }

    var diffPalettes: DiffPalettes {
        DiffPalettes(light: lightColorTheme.colors, dark: darkColorTheme.colors)
    }

    var usesDefaultDarkColorTheme: Bool {
        darkColorTheme.id == Self.defaultDarkColorThemeId
    }
}
