import JayJayCore
import SwiftUI

extension SettingsView {
    var toolsTab: some View {
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
                    settingsLabel("jayjay", icon: "bird")
                    Spacer()
                    Text(CLIInstaller.installPath)
                        .font(.system(size: 11, design: .monospaced))
                        .foregroundStyle(.secondary)
                        .textSelection(.enabled)
                    if cliInstalled {
                        Button("Uninstall") {
                            try? CLIInstaller.uninstall()
                            cliInstalled = CLIInstaller.isInstalled
                        }
                        Spacer().frame(width: 8)
                        Image(systemName: "checkmark.circle.fill")
                            .foregroundStyle(.green)
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
                let jjStatus = checkJjEnvironment()
                HStack {
                    settingsLabel("jj", icon: "arrow.triangle.branch")
                    Spacer()
                    if jjStatus.isInstalled {
                        Text(jjStatus.path)
                            .font(.system(size: 11, design: .monospaced))
                            .foregroundStyle(.secondary)
                            .textSelection(.enabled)
                            .help("jj \(jjStatus.version)")
                        Spacer().frame(width: 8)
                        Image(systemName: "checkmark.circle.fill")
                            .foregroundStyle(.green)
                    } else {
                        Text("Not installed")
                            .font(.system(size: 11))
                            .foregroundStyle(.secondary)
                    }
                }
                let ghStatus = checkGhEnvironment()
                HStack {
                    settingsLabel("gh", icon: "arrow.triangle.pull")
                    Spacer()
                    if ghStatus.isInstalled {
                        Text(ghStatus.path)
                            .font(.system(size: 11, design: .monospaced))
                            .foregroundStyle(.secondary)
                            .textSelection(.enabled)
                            .help("gh \(ghStatus.version)")
                        Spacer().frame(width: 8)
                        Image(systemName: "checkmark.circle.fill")
                            .foregroundStyle(.green)
                    } else {
                        Text("Not installed")
                            .font(.system(size: 11))
                            .foregroundStyle(.secondary)
                    }
                }
                let glabStatus = checkGlabEnvironment()
                HStack {
                    settingsLabel("glab", icon: "arrow.triangle.merge")
                    Spacer()
                    if glabStatus.isInstalled {
                        Text(glabStatus.path)
                            .font(.system(size: 11, design: .monospaced))
                            .foregroundStyle(.secondary)
                            .textSelection(.enabled)
                            .help("glab \(glabStatus.version)")
                        Spacer().frame(width: 8)
                        Image(systemName: "checkmark.circle.fill")
                            .foregroundStyle(.green)
                    } else {
                        Text("Not installed")
                            .font(.system(size: 11))
                            .foregroundStyle(.secondary)
                    }
                }
            }
        }
        .formStyle(.grouped)
    }

    // MARK: - AI helpers

    private func aiProviderRow(_ name: String, icon: String, command: String) -> some View {
        let found = findBinary(name: command) != nil
        return HStack {
            settingsLabel(name, icon: icon)
            Spacer()
            if found {
                Text("Installed")
                    .foregroundStyle(.secondary)
                    .font(.system(size: 11))
                Spacer().frame(width: 8)
                Image(systemName: "checkmark.circle.fill")
                    .foregroundStyle(.green)
            } else {
                Text("Not found")
                    .foregroundStyle(.secondary)
                    .font(.system(size: 11))
                Spacer().frame(width: 8)
                Image(systemName: "xmark.circle")
                    .foregroundStyle(.secondary)
            }
        }
    }

    private func aiProviderRow(_ name: String, icon: String, isAvailable: Bool) -> some View {
        HStack {
            settingsLabel(name, icon: icon)
            Spacer()
            if isAvailable {
                Text("Available")
                    .foregroundStyle(.secondary)
                    .font(.system(size: 11))
                Spacer().frame(width: 8)
                Image(systemName: "checkmark.circle.fill")
                    .foregroundStyle(.green)
            } else {
                Text("Not available")
                    .foregroundStyle(.secondary)
                    .font(.system(size: 11))
                Spacer().frame(width: 8)
                Image(systemName: "xmark.circle")
                    .foregroundStyle(.secondary)
            }
        }
    }

    private var appleIntelligenceAvailable: Bool {
        #if canImport(FoundationModels)
            return true
        #else
            return false
        #endif
    }
}
