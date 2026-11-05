import JayJayCore
import SwiftUI

struct ThemeSection: View {
    @Environment(AppSettings.self) private var settings
    @Environment(\.colorScheme) private var colorScheme
    @State private var editedScheme: ColorScheme?

    private var activeScheme: ColorScheme {
        editedScheme ?? colorScheme
    }

    private var isEditingDark: Bool {
        activeScheme == .dark
    }

    /// Theme names are localized, so they resolve again whenever the language changes.
    private var localizedThemes: [ColorTheme] {
        _ = settings.language
        return colorThemes()
    }

    private var selectedThemeId: Binding<String> {
        Binding(
            get: { isEditingDark ? settings.darkColorTheme.id : settings.lightColorTheme.id },
            set: { id in
                if isEditingDark {
                    settings.diffPaletteDark = id
                } else {
                    settings.diffPaletteLight = id
                }
            }
        )
    }

    var body: some View {
        let allThemes = localizedThemes
        let themes = allThemes.filter { $0.isDark == isEditingDark }
        let defaultDarkThemeName = allThemes.first { $0.id == AppSettings.defaultDarkColorThemeId }?.name ?? ""
        return Section {
            Picker(selection: Binding(
                get: { activeScheme },
                set: { editedScheme = $0 }
            )) {
                Text(Localized.Settings.appearanceLight).tag(ColorScheme.light)
                Text(Localized.Settings.appearanceDark).tag(ColorScheme.dark)
            } label: {
                SettingsLabel(Localized.Settings.theme, icon: "paintpalette")
            }
            .pickerStyle(.segmented)
            .accessibilityIdentifier(AID.Settings.themeSide)

            VStack(alignment: .leading, spacing: 8) {
                themeList(themes)
                ThemePreview(scheme: activeScheme)
            }
            .padding(.vertical, 4)

            Toggle(isOn: Binding(
                get: { settings.effectiveTintWindowWithWallpaper },
                set: { settings.tintWindowWithWallpaper = $0 }
            )) {
                SettingsLabel(Localized.Settings.wallpaperTint, icon: "rectangle.fill")
                Text(
                    settings.usesDefaultDarkColorTheme
                        ? Localized.Settings.wallpaperTintDetail
                        : Localized.Settings.wallpaperTintUnavailable(
                            theme: defaultDarkThemeName
                        )
                )
            }
            .disabled(!settings.usesDefaultDarkColorTheme)
        } header: {
            Text(Localized.Settings.colors)
        } footer: {
            Text(isEditingDark ? Localized.Settings.themeUsedDark : Localized.Settings.themeUsedLight)
        }
    }

    private func themeList(_ themes: [ColorTheme]) -> some View {
        List(themes, id: \.id, selection: selectedThemeId) { theme in
            let isSelected = theme.id == selectedThemeId.wrappedValue
            HStack(spacing: 8) {
                swatchTile(theme)
                Text(theme.name)
                    .lineLimit(1)
                Spacer(minLength: 0)
                if isSelected {
                    Image(systemName: "checkmark")
                        .font(.system(size: 11, weight: .semibold))
                        .foregroundStyle(Color.accentColor)
                }
            }
            .tag(theme.id)
            .accessibilityAddTraits(isSelected ? .isSelected : [])
            .accessibilityIdentifier(AID.Settings.themeRow(theme.id))
        }
        .listStyle(.bordered)
        .frame(maxWidth: .infinity)
        .frame(height: 150)
        .accessibilityIdentifier(AID.Settings.themeList)
    }

    private func swatchTile(_ theme: ColorTheme) -> some View {
        HStack(spacing: 2) {
            swatchBar(theme.repo.graphLine)
            swatchBar(theme.colors.gutterAddedFg)
            swatchBar(theme.colors.gutterRemovedFg)
        }
        .frame(width: 22, height: 16)
        .background(Color(rgb: theme.seed.background), in: RoundedRectangle(cornerRadius: 4, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 4, style: .continuous)
                .stroke(Color.primary.opacity(0.18), lineWidth: 1)
        )
    }

    private func swatchBar(_ rgb: UInt32) -> some View {
        RoundedRectangle(cornerRadius: 1, style: .continuous)
            .fill(Color(rgb: rgb))
            .frame(width: 3, height: 10)
    }
}
