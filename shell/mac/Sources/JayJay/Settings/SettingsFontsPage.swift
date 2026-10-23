import JayJayCore
import SwiftUI

struct SettingsFontsPage: View {
    @Environment(AppSettings.self) private var settings
    @State private var fontGroups: [FontFamilyGroup] = []
    @State private var userFamilies: [AppSettings.FontFamily] = []

    var body: some View {
        @Bindable var settings = settings
        Form {
            Section {
                FontFamilyPicker(
                    selection: $settings.interfaceFontFamily,
                    systemTitle: Localized.Settings.fontSystem,
                    groups: [FontFamilyGroup(title: Localized.Settings.myFonts, families: userFamilies)]
                )
                FontSizeStepper(value: $settings.interfaceFontSize, id: AID.Settings.interfaceFontSize)
            } header: {
                Text(Localized.Settings.interfaceFont)
            } footer: {
                Text(Localized.Settings.fontZoomHint(zoomIn: AppShortcut.zoomIn.symbol, zoomOut: AppShortcut.zoomOut.symbol, resetZoom: AppShortcut.resetZoom.symbol))
            }

            Section {
                FontFamilyPicker(
                    selection: $settings.codeFontFamily,
                    systemTitle: Localized.Settings.fontSystemMono,
                    groups: fontGroups
                )
                FontSizeStepper(value: $settings.codeFontSize, id: AID.Settings.codeFontSize)
                HStack {
                    SettingsLabel(Localized.Settings.lineSpacing, icon: "arrow.up.and.down.text.horizontal")
                    Spacer()
                    Text(settings.codeLineSpacing, format: .number.precision(.fractionLength(1)))
                        .foregroundStyle(.secondary)
                        .monospacedDigit()
                        .accessibilityIdentifier(AID.Settings.codeLineSpacing)
                    Stepper("", value: $settings.codeLineSpacing, in: AppSettings.lineSpacingRange, step: 0.1)
                        .labelsHidden()
                        .controlSize(.small)
                }
                ligaturesToggle(isOn: $settings.codeLigatures)
            } header: {
                Text(Localized.Settings.codeFont)
            } footer: {
                Text(Localized.Settings.codeFontDetail)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Section(Localized.Common.preview) {
                SettingsPreview()
            }

            Section {
                Button(Localized.Settings.resetFonts) { settings.resetFonts() }
                    .accessibilityIdentifier(AID.Settings.resetFonts)
            }
        }
        .formStyle(.grouped)
        .task {
            userFamilies = AppSettings.FontFamily.installedUserFamilies()
            fontGroups = await Task.detached { AppSettings.FontFamily.installedCodeFontGroups() }.value
        }
    }

    private func ligaturesToggle(isOn: Binding<Bool>) -> some View {
        let family = settings.codeFontFamily
        let available = family.hasLigatures
        return Toggle(isOn: available ? isOn : .constant(false)) {
            SettingsLabel(Localized.Settings.ligatures, icon: "character")
        }
        .disabled(!available)
        .help(available ? "" : Localized.Settings.ligaturesUnavailable(family: family == .system ? Localized.Settings.systemFont : family.rawValue))
        .accessibilityIdentifier(AID.Settings.codeLigatures)
    }
}
