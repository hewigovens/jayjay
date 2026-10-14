import JayJayCore
import SwiftUI

struct DiffPaletteSection: View {
    @Environment(AppSettings.self) private var settings
    @Environment(\.colorScheme) private var colorScheme
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
                isDark ? "Dark theme" : "Light theme",
                icon: isDark ? "moon" : "sun.max"
            )
        }
        .accessibilityIdentifier(isDark ? AID.Settings.diffDarkPalette : AID.Settings.diffLightPalette)
    }

    @ViewBuilder
    var body: some View {
        Section("Colors") {
            palettePicker(for: .light)
            palettePicker(for: .dark)
            Toggle(isOn: Binding(
                get: { settings.effectiveTintWindowWithWallpaper },
                set: { settings.tintWindowWithWallpaper = $0 }
            )) {
                SettingsLabel("Tint window background with wallpaper color", icon: "rectangle.fill")
                Text(
                    settings.usesDefaultDarkDiffPalette
                        ? "Allow macOS wallpaper tinting in dark mode. Turn off for a flat, neutral background."
                        : "Wallpaper tinting is available with the Default Dark theme."
                )
            }
            .disabled(!settings.usesDefaultDarkDiffPalette)
        }

        Section("Preview") {
            SettingsPreview(scheme: previewScheme ?? colorScheme)
        }
    }
}
