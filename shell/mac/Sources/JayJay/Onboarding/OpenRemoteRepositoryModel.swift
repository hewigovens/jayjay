import Foundation
import JayJayCore

@MainActor
@Observable
final class OpenRemoteRepositoryModel {
    var machine = "" {
        didSet {
            if machine != oldValue {
                disconnect()
            }
        }
    }

    var path = ""
    var machines: [String] = []
    var directory: RemoteDirectory?
    var selectedFolder: String?
    var isBusy = false
    var error: String?
    private var generation = 0

    var host: String {
        machine.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    var selectedPath: String {
        selectedFolder ?? path
    }

    var canOpen: Bool {
        !isBusy && directory != nil && !selectedPath.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }

    func loadMachines() async {
        let (machines, last) = await Task.detached { (sshMachines(), sshLastMachine()) }.value
        self.machines = machines
        if machine.isEmpty, let last {
            machine = last
        }
    }

    func invalidate() {
        generation += 1
        isBusy = false
    }

    private func disconnect() {
        guard directory != nil || error != nil else { return }
        directory = nil
        selectedFolder = nil
        path = ""
        error = nil
    }

    func browse(_ folder: String? = nil) async {
        guard !isBusy, !host.isEmpty else { return }
        let host = host
        let target = folder ?? (path.isEmpty ? "~" : path)
        generation += 1
        let request = generation
        isBusy = true
        error = nil
        let result = await Task.detached { Result { try sshBrowse(machine: host, path: target) } }.value
        guard request == generation else { return }
        isBusy = false
        switch result {
            case let .success(directory):
                self.directory = directory
                path = directory.path
                selectedFolder = nil
            case let .failure(error): self.error = error.friendlyDescription
        }
    }

    func location() async -> String? {
        guard canOpen else { return nil }
        error = nil
        generation += 1
        let request = generation
        let host = host
        let path = selectedPath
        isBusy = true
        let result = await Task.detached {
            Result {
                let location = try sshLocation(machine: host, path: path)
                do {
                    return try JayJayRepo.open(path: location).path()
                } catch JayError.RepoNotFound {
                    // The repository window offers `jj git init` for a folder that is not a repository yet.
                    return location
                }
            }
        }.value
        guard request == generation else { return nil }
        isBusy = false
        switch result {
            case let .success(location): return location
            case let .failure(error): self.error = error.friendlyDescription
                return nil
        }
    }
}
