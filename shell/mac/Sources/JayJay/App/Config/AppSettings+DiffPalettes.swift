import JayJayCore
import JayJayDiffUI

extension AppSettings {
    static let defaultLightDiffPaletteId = defaultDiffPaletteId(isDark: false)
    static let defaultDarkDiffPaletteId = defaultDiffPaletteId(isDark: true)

    var diffPalettes: DiffPalettes {
        DiffPalettes(light: lightDiffPalette.colors, dark: darkDiffPalette.colors)
    }

    var usesDefaultDarkDiffPalette: Bool {
        darkDiffPalette.id == Self.defaultDarkDiffPaletteId
    }
}
