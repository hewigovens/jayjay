import AppKit
import JayJayCore

extension RepoViewModel {
    var sshConnected: Bool {
        sshState == .connected
    }

    var sshConnectionBusy: Bool {
        isReconnectingSSH || sshState == .connecting
    }

    var sshStatusLabel: String {
        if sshConnectionBusy {
            return Localized.Ssh.reconnecting
        }
        return sshConnected ? Localized.Ssh.connected : Localized.Ssh.disconnected
    }

    var sshStatusIcon: String {
        sshConnected ? "network" : "exclamationmark.triangle"
    }

    var sshErrorMessage: String? {
        sshConnectionError ?? repo.sshError()
    }
}

@MainActor
extension RepoViewModel {
    func pollSSH() async {
        guard repo.isRemote(), !isShuttingDown else { return }
        sshState = repo.sshState()
        guard sshConnected, !isBackgroundRefreshSuspended, !isReconnectingSSH else { return }
        let changed = await Task.detached { [repo] in (try? repo.pollRemoteChanges()) ?? true }.value
        if changed {
            requestBackgroundReload()
        }
    }

    func copySSHPath() {
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(sshPath(location: repoPath), forType: .string)
    }

    func disconnectSSH() {
        do {
            try repo.disconnectSsh()
            sshState = .disconnected
        } catch {
            info = error.friendlyDescription
        }
    }

    func reconnectSSH() {
        guard !sshConnectionBusy else { return }
        isReconnectingSSH = true
        sshConnectionError = nil
        startRepoTask { [weak self, repo] in
            let result = Result { try repo.reconnectSsh() }
            await self?.finishSSHConnection(result)
        }
    }

    private func finishSSHConnection(_ result: Result<Void, any Error>) {
        isReconnectingSSH = false
        sshState = repo.sshState()
        switch result {
            case .success: refresh(isAutoTriggered: true)
            case let .failure(error):
                sshConnectionError = error.friendlyDescription
                info = error.friendlyDescription
        }
    }
}
