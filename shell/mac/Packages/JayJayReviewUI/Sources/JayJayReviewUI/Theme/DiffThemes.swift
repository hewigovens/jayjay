@preconcurrency import JayJayCore
import SwiftUI

public struct DiffThemes: Sendable, Equatable {
    public var light: ColorTheme
    public var dark: ColorTheme

    public init(light: ColorTheme = .standard(dark: false), dark: ColorTheme = .standard(dark: true)) {
        self.light = light
        self.dark = dark
    }

    public func colors(for scheme: ColorScheme) -> DiffThemeColors {
        (scheme == .dark ? dark : light).colors
    }
}

public extension ColorTheme {
    static func standard(dark: Bool) -> ColorTheme {
        colorTheme(id: defaultColorThemeId(isDark: dark), isDark: dark)
    }
}

public extension EnvironmentValues {
    @Entry var diffThemes = DiffThemes()
}
