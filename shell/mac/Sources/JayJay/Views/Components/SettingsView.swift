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
            } header: {
                Text("Appearance")
            }

            Section {
                Toggle("Side-by-side diff", isOn: sideBySideBinding)
                Toggle("Ignore whitespace changes", isOn: ignoreWhitespaceBinding)
                Toggle("Tree view for files", isOn: treeFileListBinding)
            } header: {
                Text("Diff")
            }

            Button("Reset Defaults") {
                settings.fontScale = 1.0
                settings.appearanceMode = .system
                settings.sideBySideDiff = false
                settings.ignoreWhitespace = false
                settings.treeFileList = false
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

    private var sideBySideBinding: Binding<Bool> {
        Binding(
            get: { settings.sideBySideDiff },
            set: { settings.sideBySideDiff = $0 }
        )
    }

    private var ignoreWhitespaceBinding: Binding<Bool> {
        Binding(
            get: { settings.ignoreWhitespace },
            set: { settings.ignoreWhitespace = $0 }
        )
    }

    private var treeFileListBinding: Binding<Bool> {
        Binding(
            get: { settings.treeFileList },
            set: { settings.treeFileList = $0 }
        )
    }

    private var fontScaleLabel: String {
        "\(Int(settings.fontScale * 100))%"
    }
}
