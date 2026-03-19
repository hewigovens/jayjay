import SwiftUI

struct SettingsView: View {
    @Environment(AppSettings.self) private var settings

    var body: some View {
        Form {
            Section {
                Picker("Theme", selection: appearanceBinding) {
                    ForEach(AppSettings.AppearanceMode.allCases) { mode in
                        Text(mode.title).tag(mode)
                    }
                }
                .pickerStyle(.radioGroup)

                HStack {
                    Text("Font Size")
                    Slider(value: fontScaleBinding, in: 0.85...1.45, step: 0.05)
                    Text(fontScaleLabel)
                        .foregroundStyle(.secondary)
                        .frame(width: 52, alignment: .trailing)
                }

                Button("Reset Defaults") {
                    settings.fontScale = 1.0
                    settings.appearanceMode = .system
                }
            } header: {
                Text("Appearance")
            } footer: {
                Text("Match System follows macOS light and dark mode automatically.")
            }

            Section {
                Picker("Diff Theme", selection: diffThemeBinding) {
                    ForEach(AppSettings.DiffTheme.allCases) { theme in
                        Text(theme.title).tag(theme)
                    }
                }
                .pickerStyle(.radioGroup)
            } header: {
                Text("Diff View")
            } footer: {
                Text("\"Match App Theme\" uses light or dark Monaco theme based on your appearance setting.")
            }
        }
        .formStyle(.grouped)
        .padding(24)
        .frame(width: 420)
    }

    private var appearanceBinding: Binding<AppSettings.AppearanceMode> {
        Binding(
            get: { settings.appearanceMode },
            set: { settings.appearanceMode = $0 }
        )
    }

    private var fontScaleBinding: Binding<Double> {
        Binding(
            get: { settings.fontScale },
            set: { settings.fontScale = $0 }
        )
    }

    private var diffThemeBinding: Binding<AppSettings.DiffTheme> {
        Binding(
            get: { settings.diffTheme },
            set: { settings.diffTheme = $0 }
        )
    }

    private var fontScaleLabel: String {
        "\(Int(settings.fontScale * 100))%"
    }
}
