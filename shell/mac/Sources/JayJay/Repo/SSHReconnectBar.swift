import JayJayCore
import SwiftUI

struct SSHReconnectBar: View {
    @Bindable var viewModel: RepoViewModel

    var body: some View {
        if !viewModel.sshConnected {
            let message = viewModel.sshErrorMessage ?? Localized.Ssh.errorDisconnected
            HStack(spacing: 10) {
                Image(systemName: "exclamationmark.triangle.fill")
                    .foregroundStyle(.orange)
                Text(sshMachine(location: viewModel.repoPath) ?? "")
                    .jayjayFont(12, weight: .semibold)
                Text(message)
                    .jayjayFont(12)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                    .truncationMode(.tail)
                    .help(message)
                Spacer(minLength: 8)
                Button(viewModel.sshConnectionBusy ? Localized.Ssh.reconnecting : Localized.Ssh.reconnect) {
                    viewModel.reconnectSSH()
                }
                .buttonStyle(.bordered)
                .disabled(viewModel.sshConnectionBusy)
            }
            .controlSize(.small)
            .padding(.horizontal, PaneLayout.detailInset)
            .padding(.vertical, 8)
            .background(.orange.opacity(0.08))
            Divider()
        }
    }
}
