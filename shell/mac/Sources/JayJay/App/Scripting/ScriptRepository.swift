import AppKit
import JayJayCore

@objc(ScriptRepository)
final class ScriptRepository: NSObject {
    private weak var viewModel: RepoViewModel?
    @objc let path: String

    init(viewModel: RepoViewModel) {
        self.viewModel = viewModel
        path = normalizedRepositoryPath(path: viewModel.repoPath)
    }

    func requireLiveViewModel() throws -> RepoViewModel {
        guard let viewModel, !viewModel.isShuttingDown, viewModel.switchTarget == nil else {
            throw ScriptError.repositoryClosed
        }
        return viewModel
    }

    private func read<Value>(_ value: (RepoViewModel) throws -> Value?) -> Value? {
        do {
            return try value(requireLiveViewModel())
        } catch {
            (error as? ScriptError)?.report(to: NSScriptCommand.current())
            return nil
        }
    }

    func exportedSelectedChangeIDs() throws -> [String] {
        let viewModel = try requireLiveViewModel()
        var seen = Set<String>()
        return viewModel.selectedChangeIds.compactMap { revision in
            let detail = viewModel.selectedChange?.info
            guard let change = viewModel.change(for: revision)
                ?? detail.flatMap({ $0.matchesRevision(revision) ? $0 : nil }) else { return nil }
            let id = change.changeId.id
            return seen.insert(id).inserted ? id : nil
        }
    }

    @objc var selectedChangeIDs: [String] {
        read { _ in try exportedSelectedChangeIDs() } ?? []
    }

    @objc var revset: String {
        get {
            read(\.revset) ?? ""
        }
        set {
            guard let command = NSScriptCommand.current() else { return }
            guard !command.targetsSeveralObjects else {
                ScriptError.severalRepositories.report(to: command)
                return
            }
            guard let viewModel = try? requireLiveViewModel() else {
                ScriptError.repositoryClosed.report(to: command)
                return
            }
            let isEmpty = newValue.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
            if !isEmpty, let message = viewModel.revsetError(newValue) {
                ScriptError(number: -10000, message: message).report(to: command)
                return
            }
            command.complete {
                viewModel.applyFilter(newValue)
                try await Self.settle(viewModel)
                return nil
            }
        }
    }

    @objc var errorMessage: String? {
        read(\.error)
    }

    @objc var notice: String? {
        read { $0.actionNotice?.message ?? $0.info }
    }

    @objc var windowID: NSNumber? {
        read { _ in
            MainActor.assumeIsolated {
                ScriptingHost.windowManager?.repoWindow(at: path).map { NSNumber(value: $0.windowNumber) }
            }
        }
    }

    override var objectSpecifier: NSScriptObjectSpecifier? {
        guard let description = NSClassDescription(for: NSApplication.self) as? NSScriptClassDescription else { return nil }
        return NSUniqueIDSpecifier(
            containerClassDescription: description,
            containerSpecifier: nil,
            key: "scriptRepositories",
            uniqueID: path
        )
    }
}
