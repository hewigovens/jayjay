import Foundation
import JayJayCore

extension RepoViewModel {
    func describe(rev: String, message: String) {
        perform(selecting: rev) { try $0.describe(rev: rev, message: message) }
    }

    func describeWorkingCopy(message: String) {
        perform(beforeRefresh: { viewModel in
            // The fields become what was saved, so stray whitespace does not read as an unsaved draft.
            viewModel.commitSummaryDraft = commitSummary(message: message)
            viewModel.commitDescriptionDraft = commitBody(message: message)
        }, {
            try $0.describe(rev: "@", message: message)
        })
    }

    @MainActor
    @discardableResult
    func commit(message: String) async -> Bool {
        guard !isCommitting else { return false }
        isCommitting = true
        defer { isCommitting = false }
        // Capture before the commit rewrites the graph; the committed change keeps this id and only its marks should clear.
        let committedChangeId = changes.first(where: \.isWorkingCopy)?.changeId.id
        // Text typed in the commit box survives a commit made with another message, such as the description sheet's.
        let commitsDraft = !showsWorkingCopyDraft || joinCommitMessage(summary: commitSummaryDraft, body: commitDescriptionDraft) == message
        return await performAwaiting(beforeRefresh: { viewModel in
            if let committedChangeId {
                viewModel.reviewStore.clearChange(changeId: committedChangeId)
            }
            if commitsDraft {
                viewModel.commitSummaryDraft = ""
                viewModel.commitDescriptionDraft = ""
            }
        }, {
            try $0.jjCommit(message: message)
        })
    }

    func newChange(parent: String, message: String = "") {
        performClearingDrafts { try $0.newChange(parent: parent, message: message) }
    }

    func insertChange(rev: String, position: InsertPosition) {
        performClearingDrafts { try $0.newChangeInserted(rev: rev, position: position, message: "") }
    }

    private func performClearingDrafts(_ action: @escaping RepoOperation<Void>) {
        perform(beforeRefresh: { viewModel in
            viewModel.commitSummaryDraft = ""
            viewModel.commitDescriptionDraft = ""
        }, action)
    }

    func abandon(rev: String) {
        perform(selecting: survivingParent(ofAbandoned: [rev]), undoable: Localized.Repo.abandonedChanges(count: 1), beforeRefresh: { viewModel in
            viewModel.applySingleSelectedChange(nil)
        }, {
            try $0.abandon(rev: rev)
        })
    }

    func abandon(revs: [String]) {
        perform(selecting: survivingParent(ofAbandoned: revs), undoable: Localized.Repo.abandonedChanges(count: revs.count), beforeRefresh: { viewModel in
            viewModel.applySingleSelectedChange(nil)
        }, {
            try $0.abandonMany(revs: revs)
        })
    }

    private func survivingParent(ofAbandoned revs: [String]) -> String {
        let abandoned = revs.compactMap { change(for: $0) }
        let abandonedIds = Set(abandoned.map(\.commitId.id))
        let parent = abandoned.lazy.flatMap(\.parents).first { !abandonedIds.contains($0) }
        return parent.flatMap { change(for: $0)?.selectionRevision } ?? "@"
    }

    func squash(rev: String) {
        let destination = change(for: rev)?.parents.first.flatMap { change(for: $0)?.selectionRevision }
        perform(selecting: destination ?? "@", undoable: Localized.Repo.squashedChanges(count: 1)) { try $0.squash(rev: rev, intoRev: nil) }
    }

    func squash(rev: String, into destination: String) {
        perform(selecting: destination, undoable: Localized.Repo.squashedChanges(count: 1)) {
            try $0.squash(rev: rev, intoRev: destination)
        }
    }

    func parallelize(revs: [String]) {
        performResult(
            selecting: revs.first,
            undoable: { outcome in
                switch outcome {
                    case .changed: Localized.Repo.parallelizedChanges(count: revs.count)
                    case .unchanged: Localized.Menu.nothingToParallelizeTheSelectionWould
                }
            },
            { try $0.parallelize(revs: revs) }
        )
    }

    func fix(revs: [String]) {
        performResult(
            selecting: revs.first,
            afterRefresh: { viewModel, summary in
                let message = fixSummaryMessage(summary: summary)
                if summary.failures.isEmpty {
                    viewModel.info = message
                } else {
                    viewModel.error = message
                }
            },
            { try $0.fix(revs: revs) }
        )
    }

    func squash(revs: [String]) {
        performResult(
            selectingResult: { $0 },
            undoable: { _ in Localized.Repo.squashedChanges(count: revs.count) },
            { try $0.squashMany(revs: revs) }
        )
    }

    func edit(rev: String) {
        perform(selecting: rev, beforeRefresh: { viewModel in
            guard viewModel.change(for: rev)?.isWorkingCopy != true else { return }
            // Explicit Edit changes which revision @ points to, so discard any draft for the old working copy before the new description is loaded.
            viewModel.commitSummaryDraft = commitSummary(message: viewModel.workingCopyDescription)
            viewModel.commitDescriptionDraft = commitBody(message: viewModel.workingCopyDescription)
        }, {
            try $0.edit(rev: rev)
        })
    }

    func absorb(rev: String) {
        performResult(
            undoable: { outcome in
                switch outcome {
                    case .changed: Localized.Repo.absorbedIntoAncestors
                    case .unchanged: Localized.Menu.nothingToAbsorbUseSquashInto
                }
            },
            { try $0.absorb(rev: rev) }
        )
    }

    func revertChange(rev: String) {
        performResult(selectingResult: { $0 }, undoable: { _ in Localized.Repo.revertedChange }, { try $0.revertChange(rev: rev) })
    }

    /// The merge becomes the working copy.
    func merge(parents: [String]) {
        perform(undoable: Localized.Repo.createdMergeChange) { try $0.merge(parentRevs: parents) }
    }

    func duplicate(rev: String) {
        performResult(selectingResult: { $0 }, undoable: { _ in Localized.Repo.duplicatedChange }, { try $0.duplicate(rev: rev) })
    }

    func opRestore(opId: String) {
        perform(undoable: Localized.Oplog.restoredOperation) { try $0.opRestore(opId: opId) }
    }
}
