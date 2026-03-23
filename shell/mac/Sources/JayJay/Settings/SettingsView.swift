import SwiftUI

struct SettingsView: View {
    @Environment(AppSettings.self) private var settings
    @ObservedObject var updater: SparkleUpdater
    @State private var cliInstalled = CLIInstaller.isInstalled

    var body: some View {
        TabView {
            appearanceTab
                .tabItem { Label("Appearance", systemImage: "paintbrush") }
            diffTab
                .tabItem { Label("Diff", systemImage: "doc.text.magnifyingglass") }
            toolsTab
                .tabItem { Label("Tools", systemImage: "wrench.and.screwdriver") }
            jujutsuTab
                .tabItem { Label("Jujutsu", systemImage: "arrow.triangle.branch") }
            AboutView(embedded: true, updater: updater)
                .tabItem { Label("About", systemImage: "info.circle") }
        }
        .frame(width: 480, height: 420)
    }

    // MARK: - Appearance

    private var appearanceTab: some View {
        Form {
            Picker("Theme", selection: Binding(
                get: { settings.appearanceMode },
                set: { settings.appearanceMode = $0 }
            )) {
                ForEach(AppSettings.AppearanceMode.allCases) { mode in
                    Text(mode.title).tag(mode)
                }
            }
            .pickerStyle(.segmented)

            Section("Font") {
                Picker("Family", selection: Binding(
                    get: { settings.fontFamily },
                    set: { settings.fontFamily = $0 }
                )) {
                    ForEach(AppSettings.MonoFont.allCases.filter(\.isInstalled)) { font in
                        Text(font.title).tag(font)
                    }
                }

                Stepper("Size: \(Int(settings.fontSize))pt", value: Binding(
                    get: { settings.fontSize },
                    set: { settings.fontSize = $0 }
                ), in: 9 ... 24, step: 1)
            }
        }
        .formStyle(.grouped)
    }

    // MARK: - Diff

    private var diffTab: some View {
        Form {
            Toggle("Side-by-side diff", isOn: Binding(
                get: { settings.sideBySideDiff },
                set: { settings.sideBySideDiff = $0 }
            ))
            Toggle("Ignore whitespace changes", isOn: Binding(
                get: { settings.ignoreWhitespace },
                set: { settings.ignoreWhitespace = $0 }
            ))
            Toggle("Tree view for files", isOn: Binding(
                get: { settings.treeFileList },
                set: { settings.treeFileList = $0 }
            ))
            Toggle("Skip abandon confirmation", isOn: Binding(
                get: { settings.skipAbandonConfirmation },
                set: { settings.skipAbandonConfirmation = $0 }
            ))
        }
        .formStyle(.grouped)
    }

    // MARK: - Tools

    private var toolsTab: some View {
        Form {
            Section {
                Picker("Editor", selection: Binding(
                    get: { settings.externalEditor },
                    set: { settings.externalEditor = $0 }
                )) {
                    ForEach(AppSettings.ExternalEditor.allCases) { editor in
                        Text(editor.title).tag(editor)
                    }
                }
                if settings.externalEditor == .custom {
                    TextField("Command", text: Binding(
                        get: { settings.customEditorCommand },
                        set: { settings.customEditorCommand = $0 }
                    ), prompt: Text("e.g. code, nvim"))
                }
                Picker("Terminal", selection: Binding(
                    get: { settings.terminal },
                    set: { settings.terminal = $0 }
                )) {
                    ForEach(AppSettings.Terminal.allCases) { term in
                        Text(term.title).tag(term)
                    }
                }
                if settings.terminal == .custom {
                    TextField("App name", text: Binding(
                        get: { settings.customTerminalCommand },
                        set: { settings.customTerminalCommand = $0 }
                    ), prompt: Text("e.g. Terminal"))
                }
            }

            Section("CLI") {
                HStack {
                    Text(CLIInstaller.installPath)
                        .font(.system(size: 11, design: .monospaced))
                        .foregroundStyle(.secondary)
                        .textSelection(.enabled)
                    Spacer()
                    if cliInstalled {
                        Image(systemName: "checkmark.circle.fill")
                            .foregroundStyle(.green)
                        Button("Uninstall") {
                            CLIInstaller.uninstall()
                            cliInstalled = CLIInstaller.isInstalled
                        }
                    } else {
                        Button("Install") {
                            CLIInstaller.install()
                            cliInstalled = CLIInstaller.isInstalled
                        }
                    }
                }
            }
        }
        .formStyle(.grouped)
    }

    // MARK: - Jujutsu

    private var jujutsuTab: some View {
        Form {
            Section {
                JJConfigView()
            }
        }
        .formStyle(.grouped)
    }
}
