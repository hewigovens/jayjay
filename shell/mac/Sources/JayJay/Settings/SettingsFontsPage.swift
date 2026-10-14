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
                    systemTitle: "System",
                    groups: [FontFamilyGroup(title: "My Fonts", families: userFamilies)]
                )
                FontSizeStepper(value: $settings.interfaceFontSize, id: AID.Settings.interfaceFontSize)
            } header: {
                Text("Interface Font")
            } footer: {
                Text("Press \(AppShortcut.zoomIn.symbol) or \(AppShortcut.zoomOut.symbol) to change the interface and code font sizes, \(AppShortcut.resetZoom.symbol) to reset them.")
            }

            Section {
                FontFamilyPicker(
                    selection: $settings.codeFontFamily,
                    systemTitle: "System Mono",
                    groups: fontGroups
                )
                FontSizeStepper(value: $settings.codeFontSize, id: AID.Settings.codeFontSize)
                HStack {
                    SettingsLabel("Line spacing", icon: "arrow.up.and.down.text.horizontal")
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
                Text("Code Font")
            } footer: {
                Text("Used for diffs, editors, revsets, and identifiers.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Section("Preview") {
                SettingsPreview()
            }

            Section {
                Button("Reset Fonts") { settings.resetFonts() }
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
            SettingsLabel("Ligatures", icon: "character")
        }
        .disabled(!available)
        .help(available ? "" : "\(family == .system ? "The system font" : family.rawValue) has no ligatures.")
        .accessibilityIdentifier(AID.Settings.codeLigatures)
    }
}
