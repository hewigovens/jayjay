import AppKit
import JayJayCore

@MainActor
extension ScriptRepository {
    @objc func handleRefreshCommand(_ command: NSScriptCommand) -> Any? {
        guard !command.targetsSeveralObjects else {
            ScriptError.severalRepositories.report(to: command)
            return nil
        }
        command.complete {
            let viewModel = try self.requireLiveViewModel()
            viewModel.refresh()
            try await Self.settle(viewModel)
            return nil
        }
        return nil
    }

    static func settle(_ viewModel: RepoViewModel) async throws {
        await viewModel.waitForRefresh()
        try requireLoaded(viewModel)
        guard viewModel.sshConnected else {
            throw ScriptError(number: -10000, message: "The repository is disconnected.")
        }
    }

    static func requireLoaded(_ viewModel: RepoViewModel, reportingError: Bool = true) throws {
        guard !viewModel.isShuttingDown else { throw ScriptError.repositoryClosed }
        if viewModel.workspaceVanished {
            throw ScriptError(number: -10000, message: Localized.Repo.theWorkspaceNoLongerExists)
        }
        if reportingError, let error = viewModel.error {
            throw ScriptError(number: -10000, message: error)
        }
    }

    func showFile(_ path: String) async throws {
        let viewModel = try requireLiveViewModel()
        guard viewModel.comparison == nil,
              let detail = viewModel.selectedChange,
              detail.diff.contains(where: { $0.path == path })
        else {
            throw ScriptError(number: -1728, message: "\(path) is not a file of the selected change.")
        }
        guard await viewModel.revealFile(path, in: detail.info.selectionRevision) else {
            guard !viewModel.isShuttingDown else { throw ScriptError.repositoryClosed }
            throw ScriptError(number: -10000, message: "The file list could not show \(path).")
        }
    }

    func selectChange(_ expression: String) async throws {
        let viewModel = try requireLiveViewModel()
        let resolved: [ChangeInfo]
        do {
            resolved = try await viewModel.awaitRepoTask { try $0.log(revset: "latest(\(expression), 2)") }
        } catch {
            guard !viewModel.isShuttingDown else { throw ScriptError.repositoryClosed }
            throw ScriptError(number: -1728, message: "No change matches \(expression).")
        }
        guard !viewModel.isShuttingDown else { throw ScriptError.repositoryClosed }
        guard let match = resolved.first else {
            throw ScriptError(number: -1728, message: "No change matches \(expression).")
        }
        guard resolved.count == 1 else {
            throw ScriptError(number: -1728, message: "\(expression) matches more than one change.")
        }
        guard let change = viewModel.changes.first(where: { $0.commitId.id == match.commitId.id }) else {
            throw ScriptError(number: -1728, message: "\(expression) is not shown by the current revset.")
        }
        viewModel.revealInGraph(change.selectionRevision)
        await viewModel.selectionLoadTask?.value
        guard !viewModel.isShuttingDown else { throw ScriptError.repositoryClosed }
        guard viewModel.selectedChangeId == change.selectionRevision, viewModel.selectedChange != nil else {
            if let error = viewModel.error {
                throw ScriptError(number: -10000, message: error)
            }
            throw ScriptError(number: -10000, message: "The selection changed before the change loaded.")
        }
    }
}
