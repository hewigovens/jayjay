import JayJayCore
import SwiftUI

struct SSHConnectionView: View {
    @Bindable var viewModel: RepoViewModel
    let onDismiss: () -> Void

    var body: some View {
        SheetContainer(
            title: Localized.Ssh.connection,
            cancelLabel: Localized.Common.close,
            confirmLabel: viewModel.sshConnected ? Localized.Ssh.disconnect : Localized.Ssh.reconnect,
            confirmDisabled: viewModel.sshConnectionBusy,
            onCancel: onDismiss,
            onConfirm: {
                if viewModel.sshConnected {
                    viewModel.disconnectSSH()
                } else {
                    viewModel.reconnectSSH()
                }
            },
            content: {
                Grid(alignment: .leadingFirstTextBaseline, horizontalSpacing: 10, verticalSpacing: 8) {
                    GridRow {
                        Text(Localized.Ssh.machine).foregroundStyle(.secondary)
                        Text(sshMachine(location: viewModel.repoPath) ?? "").textSelection(.enabled)
                    }
                    GridRow {
                        Text(Localized.Ssh.folder).foregroundStyle(.secondary)
                        HStack(spacing: 6) {
                            Text(sshPath(location: viewModel.repoPath))
                                .jayjayFont(12, design: .monospaced)
                                .textSelection(.enabled)
                                .lineLimit(1)
                                .truncationMode(.middle)
                            Button(Localized.Ssh.copyPath, systemImage: "doc.on.doc") { viewModel.copySSHPath() }
                                .labelStyle(.iconOnly)
                                .buttonStyle(.borderless)
                                .help(Localized.Ssh.copyPath)
                                .focusable(false)
                        }
                    }
                }
                .jayjayFont(12)
                Label(viewModel.sshStatusLabel, systemImage: viewModel.sshStatusIcon)
                    .jayjayFont(12)
                if let message = viewModel.sshErrorMessage {
                    Text(message).jayjayFont(12).foregroundStyle(.secondary).textSelection(.enabled)
                }
            }
        )
        .frame(width: 480)
    }
}
