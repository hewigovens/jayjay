import JayJayCore
import SwiftUI

struct DiffPaletteSection: View {
    @Environment(AppSettings.self) private var settings
    @State private var previewScheme: ColorScheme?

    private static let lightPalettes = diffPalettes().filter { !$0.isDark }
    private static let darkPalettes = diffPalettes().filter(\.isDark)

    private func palettePicker(for scheme: ColorScheme) -> some View {
        let isDark = scheme == .dark
        let palettes = isDark ? Self.darkPalettes : Self.lightPalettes
        let keyPath: ReferenceWritableKeyPath<AppSettings, String> = isDark ? \.diffPaletteDark : \.diffPaletteLight
        let selection = Binding(
            get: { isDark ? settings.darkDiffPalette.id : settings.lightDiffPalette.id },
            set: {
                settings[keyPath: keyPath] = $0
                previewScheme = scheme
            }
        )
        return Picker(selection: selection) {
            ForEach(palettes, id: \.id) { palette in
                Text(palette.name).tag(palette.id)
            }
        } label: {
            SettingsLabel(
                isDark ? Localized.Settings.darkTheme : Localized.Settings.lightTheme,
                icon: isDark ? "moon" : "sun.max"
            )
        }
        .accessibilityIdentifier(isDark ? AID.Settings.diffDarkPalette : AID.Settings.diffLightPalette)
    }

    var body: some View {
        Section(Localized.Settings.colors) {
            palettePicker(for: .light)
            palettePicker(for: .dark)
            Toggle(isOn: Binding(
                get: { settings.effectiveTintWindowWithWallpaper },
                set: { settings.tintWindowWithWallpaper = $0 }
            )) {
                SettingsLabel(Localized.Settings.wallpaperTint, icon: "rectangle.fill")
                Text(
                    settings.usesDefaultDarkDiffPalette
                        ? Localized.Settings.wallpaperTintDetail
                        : Localized.Settings.wallpaperTintUnavailable(theme: diffPalette(id: defaultDiffPaletteId(isDark: true), isDark: true).name)
                )
            }
            .disabled(!settings.usesDefaultDarkDiffPalette)
        }

        Section(Localized.Common.preview) {
            SettingsPreview(scheme: previewScheme)
        }
    }
}
