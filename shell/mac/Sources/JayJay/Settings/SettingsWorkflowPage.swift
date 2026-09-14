import SwiftUI

struct SettingsWorkflowPage: View {
    @Environment(AppSettings.self) private var settings

    var body: some View {
        Form {
            Section("Confirmations") {
                Toggle(isOn: Binding(
                    get: { !settings.skipAbandonConfirmation },
                    set: { settings.skipAbandonConfirmation = !$0 }
                )) {
                    SettingsLabel("Confirm before abandoning changes", icon: "trash")
                }
                .accessibilityIdentifier(AID.Settings.confirmAbandon)
                Toggle(isOn: Binding(
                    get: { !settings.skipWorkspaceDeleteConfirmation },
                    set: { settings.skipWorkspaceDeleteConfirmation = !$0 }
                )) {
                    SettingsLabel("Confirm before deleting a workspace", icon: "folder.badge.minus")
                }
                .accessibilityIdentifier(AID.Settings.confirmWorkspaceDelete)
                Toggle(isOn: Binding(
                    get: { settings.confirmDragRebase },
                    set: { settings.confirmDragRebase = $0 }
                )) {
                    SettingsLabel("Confirm drag-to-rebase", icon: "arrow.up.forward.app")
                }
            }

            Section("Repository Behavior") {
                Toggle(isOn: Binding(
                    get: { settings.enableGitSubmoduleSupport },
                    set: { settings.enableGitSubmoduleSupport = $0 }
                )) {
                    SettingsLabel("Enable Git submodule support", icon: "square.stack.3d.up")
                }
            }
        }
        .formStyle(.grouped)
    }
}
