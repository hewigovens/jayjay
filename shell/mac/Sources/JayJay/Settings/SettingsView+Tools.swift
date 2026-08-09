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
        }
        .formStyle(.grouped)
    }

    // MARK: - AI helpers

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
