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
                HStack {
                    settingsLabel("jj tool configuration", icon: "doc.on.doc")
                    Spacer()
                    Text("diff, edit & merge")
                        .font(.system(size: 11))
                        .foregroundStyle(.secondary)
                    CopyIconButton(
                        value: jjToolConfig(),
                        help: "Copy jj tool configuration",
                        label: "Copy Config"
                    )
                    .accessibilityIdentifier(AID.Settings.copyJJToolConfig)
                }
                cliStatusRow("jj", icon: "arrow.triangle.branch", status: checkJjEnvironment())
                cliStatusRow("gh", icon: "arrow.triangle.pull", status: checkGhEnvironment())
                cliStatusRow("glab", icon: "arrow.triangle.merge", status: checkGlabEnvironment())
            }
        }
        .formStyle(.grouped)
    }

    // MARK: - AI helpers

    private func cliStatusRow(_ name: String, icon: String, status: CliStatus) -> some View {
        HStack {
            settingsLabel(name, icon: icon)
            Spacer()
            if status.isInstalled {
                Text(status.path)
                    .font(.system(size: 11, design: .monospaced))
                    .foregroundStyle(.secondary)
                    .textSelection(.enabled)
                    .help("\(name) \(status.version)")
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

    private func aiProviderRow(_ name: String, icon: String, command: String) -> some View {
        availabilityRow(
            name,
            icon: icon,
            isAvailable: findBinary(name: command) != nil,
            availableLabel: "Installed",
            unavailableLabel: "Not found"
        )
    }

    private func aiProviderRow(_ name: String, icon: String, isAvailable: Bool) -> some View {
        availabilityRow(
            name,
            icon: icon,
            isAvailable: isAvailable,
            availableLabel: "Available",
            unavailableLabel: "Not available"
        )
    }

    private func availabilityRow(
        _ name: String,
        icon: String,
        isAvailable: Bool,
        availableLabel: String,
        unavailableLabel: String
    ) -> some View {
        HStack {
            settingsLabel(name, icon: icon)
            Spacer()
            if isAvailable {
                Text(availableLabel)
                    .foregroundStyle(.secondary)
                    .font(.system(size: 11))
                Spacer().frame(width: 8)
                Image(systemName: "checkmark.circle.fill")
                    .foregroundStyle(.green)
            } else {
                Text(unavailableLabel)
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
