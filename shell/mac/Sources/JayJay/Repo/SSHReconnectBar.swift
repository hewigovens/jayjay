import JayJayCore
import SwiftUI

struct SSHReconnectBar: View {
    @Bindable var viewModel: RepoViewModel

    var body: some View {
        if !viewModel.sshConnected {
            let message = viewModel.sshErrorMessage ?? Localized.Ssh.errorDisconnected
            NoticeBar(systemImage: "network.slash", tint: .red, title: sshMachine(location: viewModel.repoPath) ?? "") {
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
        }
    }
}
