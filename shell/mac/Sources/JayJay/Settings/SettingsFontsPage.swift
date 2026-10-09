import SwiftUI

struct SettingsFontsPage: View {
    @Environment(AppSettings.self) private var settings
    @State private var fontGroups: [FontFamilyGroup] = []

    var body: some View {
        @Bindable var settings = settings
        Form {
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
            } footer: {
                Text("Used for diffs, editors, revsets, and identifiers.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Section("Preview") {
                FontSpecimenView()
            }

            Section {
                Button("Reset Code Font") { settings.resetCodeFont() }
                    .accessibilityIdentifier(AID.Settings.resetCodeFont)
            }
        }
        .formStyle(.grouped)
        .task {
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
