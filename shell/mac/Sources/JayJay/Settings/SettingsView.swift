import SwiftUI

struct SettingsView: View {
    @Environment(AppSettings.self) private var settings
    @ObservedObject var updater: SparkleUpdater
    @State private var cliInstalled = CLIInstaller.isInstalled
    @State private var cliError: String?

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
            Section {
                Picker(selection: Binding(
                    get: { settings.appearanceMode },
                    set: { settings.appearanceMode = $0 }
                )) {
                    ForEach(AppSettings.AppearanceMode.allCases) { mode in
                        Text(mode.title).tag(mode)
                    }
                } label: {
                    settingsLabel("Theme", icon: "circle.lefthalf.filled")
                }
                .pickerStyle(.segmented)
            }

            Section("Font") {
                Picker(selection: Binding(
                    get: { settings.fontFamily },
                    set: { settings.fontFamily = $0 }
                )) {
                    ForEach(AppSettings.MonoFont.allCases.filter(\.isInstalled)) { font in
                        Text(font.title).tag(font)
                    }
                } label: {
                    settingsLabel("Family", icon: "textformat")
                }

                HStack {
                    settingsLabel("Size", icon: "textformat.size")
                    Spacer()
                    Text("\(Int(settings.fontSize))pt")
                        .foregroundStyle(.secondary)
                        .monospacedDigit()
                    Stepper("", value: Binding(
                        get: { settings.fontSize },
                        set: { settings.fontSize = $0 }
                    ), in: 9 ... 24, step: 1)
                        .labelsHidden()
                        .controlSize(.small)
                }
            }
        }
        .formStyle(.grouped)
    }

    // MARK: - Diff

    private var diffTab: some View {
        Form {
            Section {
                Toggle(isOn: Binding(
                    get: { settings.sideBySideDiff },
                    set: { settings.sideBySideDiff = $0 }
                )) {
                    settingsLabel("Side-by-side diff", icon: "rectangle.split.2x1")
                }
                Toggle(isOn: Binding(
                    get: { settings.ignoreWhitespace },
                    set: { settings.ignoreWhitespace = $0 }
                )) {
                    settingsLabel("Ignore whitespace changes", icon: "space")
                }
                Toggle(isOn: Binding(
                    get: { settings.treeFileList },
                    set: { settings.treeFileList = $0 }
                )) {
                    settingsLabel("Tree view for files", icon: "list.bullet.indent")
                }
            }

            Section {
                Toggle(isOn: Binding(
                    get: { settings.skipAbandonConfirmation },
                    set: { settings.skipAbandonConfirmation = $0 }
                )) {
                    settingsLabel("Skip abandon confirmation", icon: "trash")
                }
            }
        }
        .formStyle(.grouped)
    }

    // MARK: - Tools

    private var toolsTab: some View {
        Form {
            Section {
                Picker(selection: Binding(
                    get: { settings.externalEditor },
                    set: { settings.externalEditor = $0 }
                )) {
                    ForEach(AppSettings.ExternalEditor.allCases) { editor in
                        Text(editor.title).tag(editor)
                    }
                } label: {
                    settingsLabel("Editor", icon: "curlybraces")
                }
                if settings.externalEditor == .custom {
                    TextField("Command", text: Binding(
                        get: { settings.customEditorCommand },
                        set: { settings.customEditorCommand = $0 }
                    ), prompt: Text("e.g. code, nvim"))
                }
                Picker(selection: Binding(
                    get: { settings.terminal },
                    set: { settings.terminal = $0 }
                )) {
                    ForEach(AppSettings.Terminal.allCases) { term in
                        Text(term.title).tag(term)
                    }
                } label: {
                    settingsLabel("Terminal", icon: "terminal")
                }
                if settings.terminal == .custom {
                    TextField("App name", text: Binding(
                        get: { settings.customTerminalCommand },
                        set: { settings.customTerminalCommand = $0 }
                    ), prompt: Text("e.g. Terminal"))
                }
            }

            Section("AI Commit Message") {
                aiProviderRow("Codex CLI", icon: "chevron.left.forwardslash.chevron.right", command: "codex")
                aiProviderRow("Claude CLI", icon: "asterisk", command: "claude")
                aiProviderRow("Apple Intelligence", icon: "apple.logo", isAvailable: appleIntelligenceAvailable)
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
                            try? CLIInstaller.uninstall()
                            cliInstalled = CLIInstaller.isInstalled
                        }
                    } else {
                        Button("Install") {
                            do {
                                try CLIInstaller.install()
                                cliError = nil
                            } catch {
                                cliError = error.localizedDescription
                            }
                            cliInstalled = CLIInstaller.isInstalled
                        }
                    }
                }
                if let cliError {
                    Text(cliError)
                        .font(.system(size: 11))
                        .foregroundStyle(.red)
                }
            }
        }
        .formStyle(.grouped)
    }

    // MARK: - Icon helper

    private func settingsLabel(_ title: String, icon: String) -> some View {
        HStack(spacing: 6) {
            Image(systemName: icon)
                .frame(width: 16, alignment: .center)
                .foregroundStyle(.secondary)
            Text(title)
        }
    }

    // MARK: - AI helpers

    private func aiProviderRow(_ name: String, icon: String, command: String) -> some View {
        let found = AppSettings.ExternalEditor.findBinary(command) != nil
        return HStack {
            settingsLabel(name, icon: icon)
            Spacer()
            if found {
                Image(systemName: "checkmark.circle.fill").foregroundStyle(.green)
                Text("Installed").foregroundStyle(.secondary).font(.system(size: 11))
            } else {
                Image(systemName: "xmark.circle").foregroundStyle(.secondary)
                Text("Not found").foregroundStyle(.secondary).font(.system(size: 11))
            }
        }
    }

    private func aiProviderRow(_ name: String, icon: String, isAvailable: Bool) -> some View {
        HStack {
            settingsLabel(name, icon: icon)
            Spacer()
            if isAvailable {
                Image(systemName: "checkmark.circle.fill").foregroundStyle(.green)
                Text("Available").foregroundStyle(.secondary).font(.system(size: 11))
            } else {
                Image(systemName: "xmark.circle").foregroundStyle(.secondary)
                Text("Not available").foregroundStyle(.secondary).font(.system(size: 11))
            }
        }
    }

    private var appleIntelligenceAvailable: Bool {
        #if canImport(FoundationModels)
            if #available(macOS 26.0, *) { return true }
        #endif
        return false
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
