import Foundation
import JayJayCore

extension RepoViewModel {
    func restoreFiles(rev: String, paths: [String]) {
        perform(selecting: rev) { try $0.restoreFiles(rev: rev, paths: paths) }
    }

    func deleteFiles(paths: [String]) {
        perform { try $0.deleteFiles(paths: paths) }
    }

    func ignoreAndUntrack(paths: [String]) {
        perform(selecting: nil) { try $0.ignoreAndUntrack(paths: paths) }
    }

    func split(rev: String, paths: [String], message: String = "", parallel: Bool = false) {
        // The remainder keeps @'s content but gets a fresh change id; the typed draft still describes it.
        perform(beforeRefresh: { viewModel in
            viewModel.keepCommitDraftOnNextWorkingCopyChange = true
        }, {
            try $0.split(rev: rev, paths: paths, message: message, parallel: parallel)
        })
    }

    func moveToWorkingCopy(rev: String, paths: [String]) {
        perform { try $0.moveToWorkingCopy(rev: rev, paths: paths) }
    }

    func applyDiffSelection(
        rev: String,
        destination: DiffEditDestination,
        selections: [DiffEditFileSelection],
        message: String,
        ignoreWhitespace: Bool
    ) {
        // Abandoning lines from a leaf @ rewrites only that commit, so the cheap in-place row patch is safe; any other rev (or @ mid-stack) rebases descendants and needs the full refresh.
        if destination == .removeFromSource, Self.canPatchWorkingCopyRowInPlace(rev: rev, changes: changes) {
            abandonWorkingCopySelection(rev: rev, selections: selections, ignoreWhitespace: ignoreWhitespace)
            return
        }
        perform(selecting: rev) {
            try $0.applyDiffSelection(
                rev: rev,
                destination: destination,
                selections: selections,
                message: message,
                ignoreWhitespace: ignoreWhitespace
            )
        }
    }

    /// True when rev is the working copy and @ has no children in the loaded graph, so a removeFromSource rewrite cannot move any other row.
    static func canPatchWorkingCopyRowInPlace(rev: String, changes: [ChangeInfo]) -> Bool {
        guard let workingCopy = changes.first(where: \.isWorkingCopy) else { return false }
        let revIsWorkingCopy = rev == "@" || rev == workingCopy.changeId.id || rev == workingCopy.commitId.id
        let isLeaf = !changes.contains { $0.parents.contains(workingCopy.commitId.id) }
        return revIsWorkingCopy && isLeaf
    }

    private func abandonWorkingCopySelection(
        rev: String,
        selections: [DiffEditFileSelection],
        ignoreWhitespace: Bool
    ) {
        lastInternalMutationAt = Date()
        let includeSubmoduleStatuses = includeSubmoduleStatuses
        load {
            try $0.applyDiffSelection(
                rev: rev,
                destination: .removeFromSource,
                selections: selections,
                message: "",
                ignoreWhitespace: ignoreWhitespace
            )
            // The mutation already updated @; reload only this change's detail.
            let detail = try Self.loadSummaryWithConflicts(
                repo: $0,
                rev: rev,
                includeSubmoduleStatuses: includeSubmoduleStatuses
            )
            return (detail, StatusBarSnapshot.load(from: $0))
        } onSuccess: { viewModel, result in
            let (detail, statusBar) = result
            viewModel.successActionSignal += 1
            viewModel.selectedChange = detail
            viewModel.selectedChangeId = detail.info.selectionRevision
            viewModel.apply(statusBar)
            // Patch the @ row in place (no descendants → edges unchanged) instead of a full log rebuild.
            if let index = viewModel.graphEntries.firstIndex(where: { $0.change.isWorkingCopy }) {
                viewModel.graphEntries[index] = GraphEntry(
                    change: detail.info,
                    edges: viewModel.graphEntries[index].edges
                )
            }
        }
    }

    func resolveUseOurs(rev: String, path: String) {
        perform(selecting: rev) { try $0.resolveUseOurs(rev: rev, path: path) }
    }

    func resolveInEditor(rev: String, path: String, tool: String) {
        perform(selecting: rev) { try $0.resolveWithTool(rev: rev, path: path, tool: tool) }
    }

    func resolveUseTheirs(rev: String, path: String) {
        perform(selecting: rev) { try $0.resolveUseTheirs(rev: rev, path: path) }
    }
}
