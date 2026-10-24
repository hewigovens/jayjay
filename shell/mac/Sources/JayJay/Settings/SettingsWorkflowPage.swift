import JayJayCore
import SwiftUI

struct SettingsWorkflowPage: View {
    @Environment(AppSettings.self) private var settings

    var body: some View {
        Form {
            Section(Localized.Settings.confirmations) {
                Toggle(isOn: Binding(
                    get: { !settings.skipAbandonConfirmation },
                    set: { settings.skipAbandonConfirmation = !$0 }
                )) {
                    SettingsLabel(Localized.Settings.confirmAbandon, icon: "trash")
                }
                .accessibilityIdentifier(AID.Settings.confirmAbandon)
                Toggle(isOn: Binding(
                    get: { !settings.skipWorkspaceDeleteConfirmation },
                    set: { settings.skipWorkspaceDeleteConfirmation = !$0 }
                )) {
                    SettingsLabel(Localized.Settings.confirmWorkspaceDelete, icon: "folder.badge.minus")
                }
                .accessibilityIdentifier(AID.Settings.confirmWorkspaceDelete)
                Toggle(isOn: Binding(
                    get: { settings.confirmDragRebase },
                    set: { settings.confirmDragRebase = $0 }
                )) {
                    SettingsLabel(Localized.Settings.confirmDragRebase, icon: "arrow.up.forward.app")
                }
            }

            Section(Localized.Settings.repositoryBehavior) {
                Toggle(isOn: Binding(
                    get: { settings.enableGitSubmoduleSupport },
                    set: { settings.enableGitSubmoduleSupport = $0 }
                )) {
                    SettingsLabel(Localized.Settings.submoduleSupport, icon: "square.stack.3d.up")
                }
            }
        }
        .formStyle(.grouped)
    }
}
