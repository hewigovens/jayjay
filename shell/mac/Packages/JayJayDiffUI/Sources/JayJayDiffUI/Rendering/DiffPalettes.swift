import JayJayCore
import SwiftUI

public struct DiffPalettes: Equatable {
    public var light: DiffThemeColors
    public var dark: DiffThemeColors

    public static let standard = DiffPalettes(light: diffThemeColors(isDark: false), dark: diffThemeColors(isDark: true))

    public init(light: DiffThemeColors, dark: DiffThemeColors) {
        self.light = light
        self.dark = dark
    }

    public func colors(for scheme: ColorScheme) -> DiffColors {
        scheme == .dark ? DiffColors(palette: dark, isDark: true) : DiffColors(palette: light, isDark: false)
    }
}

private struct DiffPalettesKey: EnvironmentKey {
    static let defaultValue = DiffPalettes.standard
}

public extension EnvironmentValues {
    var diffPalettes: DiffPalettes {
        get { self[DiffPalettesKey.self] }
        set { self[DiffPalettesKey.self] = newValue }
    }
}
