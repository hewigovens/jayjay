import JayJayCore
import SwiftUI

struct SettingsAgentSkillsSection: View {
    @State private var installation: AgentSkillInstallation?
    @State private var busy = false
    @State private var error: String?

    var body: some View {
        Section {
            Toggle(isOn: Binding(
                get: { installation?.installed ?? false },
                set: { setInstalled($0) }
            )) {
                VStack(alignment: .leading, spacing: 4) {
                    SettingsLabel(Localized.Settings.agentSkill, icon: "sparkles")
                    Text(Localized.Settings.agentSkillDetail)
                        .font(.system(size: 11))
                        .foregroundStyle(.secondary)
                }
            }
            .toggleStyle(.switch)
            .disabled(installation == nil || busy)
            .accessibilityLabel(Localized.Settings.agentSkill)
            .accessibilityHint(Localized.Settings.agentSkillDetail)
            .accessibilityIdentifier(AID.Settings.agentSkill)
            if let installation, installation.installed, !installation.complete {
                Button(Localized.Settings.agentSkillRepair) { setInstalled(true) }
                    .disabled(busy)
            }
            if let error {
                Text(error)
                    .font(.system(size: 11))
                    .foregroundStyle(.red)
                    .textSelection(.enabled)
            }
        } header: {
            Text(Localized.Settings.codingAgents)
        } footer: {
            VStack(alignment: .leading, spacing: 4) {
                Text(Localized.Settings.agentSkillAgents)
                if !CLIInstaller.isInstalled {
                    Text(Localized.Settings.agentSkillCli)
                }
            }
        }
        .task {
            guard installation == nil, !busy else { return }
            let result = await Task.detached(priority: .utility) {
                Result { try agentSkillInstallation() }
            }.value
            guard !Task.isCancelled else { return }
            apply(result)
        }
    }

    private func setInstalled(_ installed: Bool) {
        busy = true
        error = nil
        Task {
            let (result, status) = await Task.detached(priority: .userInitiated) {
                (Result { try setAgentSkillInstalled(installed: installed) }, Result { try agentSkillInstallation() })
            }.value
            apply(status)
            if case let .failure(failure) = result {
                error = failure.friendlyDescription
            }
            busy = false
        }
    }

    private func apply(_ result: Result<AgentSkillInstallation, Error>) {
        switch result {
            case let .success(status): installation = status
            case let .failure(failure): error = failure.friendlyDescription
        }
    }
}
