import JayJayCore
import JayJayDiffUI
import SwiftUI

extension ChangeDetailView {
    @ViewBuilder
    func fileContextMenu(for path: String) -> some View {
        let contextPaths = contextSelectionPaths(for: path)
        let contextHunks = contextPaths.compactMap { contextPath in
            detail.diff.first(where: { $0.path == contextPath })
        }
        let changedPaths = contextPaths.flatMap { contextPath in
            detail.diff.first(where: { $0.path == contextPath })?.changedPaths ?? [contextPath]
        }
        let includesSubmodulePlaceholder = contextHunks.contains { $0.isSubmodulePlaceholder }
        let includesUnreviewableFile = contextHunks.contains { $0.reviewIdentity.isEmpty }
        let isBatch = contextPaths.count > 1
        let reviewLabel = reviewActionLabel(for: contextPaths)

        if !isBatch {
            singleFileActions(path: path, isSubmodulePlaceholder: includesSubmodulePlaceholder)
        }

        if !includesSubmodulePlaceholder {
            if showsReviewControls, !includesUnreviewableFile {
                Button(reviewLabel) {
                    applyReviewMarks(
                        paths: contextPaths,
                        reviewed: !contextPaths.allSatisfy(reviewedPaths.contains)
                    )
                }
                .keyboardShortcut("r", modifiers: [])
                Divider()
            }

            Button(splitActionLabel(for: contextPaths)) {
                splitRequest = SplitSheetRequest(paths: changedPaths)
            }
            .keyboardShortcut("s", modifiers: [])
            if !detail.info.isWorkingCopy {
                Button(moveToWorkingCopyActionLabel(for: contextPaths)) {
                    actions?.moveToWorkingCopy(rev: detailRevision, paths: changedPaths)
                }
            }
            restoreActions(cardPaths: contextPaths, paths: changedPaths)
            if detail.info.isWorkingCopy {
                Button(deleteActionLabel(for: contextPaths), role: .destructive) {
                    actions?.deleteFiles(paths: contextPaths)
                }
            }
            Divider()
            Button(ignoreActionLabel(for: contextPaths)) {
                actions?.ignoreAndUntrack(paths: contextPaths)
            }
        }
    }

    @ViewBuilder
    private func singleFileActions(path: String, isSubmodulePlaceholder: Bool) -> some View {
        Button("Open in \(appSettings.externalEditor.title)") {
            appSettings.openInEditor(filePath: path, repoPath: repoPath)
        }
        .keyboardShortcut("o", modifiers: [])
        Button(Localized.Common.showIn(app: "Finder")) { showInFinder(path) }
            .keyboardShortcut("f", modifiers: [])
        Button("Copy Path") {
            NSPasteboard.general.clearContents()
            NSPasteboard.general.setString(path, forType: .string)
        }
        .keyboardShortcut("c", modifiers: [])
        if !isSubmodulePlaceholder {
            Divider()
            Button("Annotate (Blame)") { loadAnnotate(rev: detailRevision, path: path) }
                .keyboardShortcut("b", modifiers: [])
            Button("File History") {
                windowManager?.openFileHistory(FileHistoryRequest(repoPath: repoPath, path: path))
            }
            .keyboardShortcut("h", modifiers: [])
            Divider()
        }
    }

    @ViewBuilder
    private func restoreActions(cardPaths: [String], paths: [String]) -> some View {
        if detail.info.parents.count > 1 {
            Menu(restoreActionLabel(for: cardPaths)) {
                ForEach(Array(detail.info.parents.enumerated()), id: \.offset) { index, parentId in
                    Button("Parent \(index + 1): \(String(parentId.prefix(8)))") {
                        actions?.restoreFiles(rev: parentId, paths: paths)
                    }
                }
            }
        } else {
            Button(restoreActionLabel(for: cardPaths)) {
                actions?.restoreFiles(rev: detailRevision, paths: paths)
            }
        }
    }

    func canEditWorkingCopyFile(_ hunk: DiffHunk) -> Bool {
        Self.canEditWorkingCopyFile(
            info: detail.info,
            isCompareMode: isCompareMode,
            hunk: hunk,
            hasConflict: conflictedPaths.contains(hunk.path)
        )
    }

    static func canEditWorkingCopyFile(
        info: ChangeInfo,
        isCompareMode: Bool,
        hunk: DiffHunk,
        hasConflict: Bool
    ) -> Bool {
        info.isWorkingCopy
            && !isCompareMode
            && !hasConflict
            && hunk.hunkType != .removed
            && hunk.projection == nil
    }

    private func splitActionLabel(for paths: [String]) -> String {
        paths.count == 1 ? "Split to New Change" : "Split \(paths.count) Files to New Change"
    }

    private func moveToWorkingCopyActionLabel(for paths: [String]) -> String {
        paths.count == 1 ? "Move to Working Copy" : "Move \(paths.count) Files to Working Copy"
    }

    private func restoreActionLabel(for paths: [String]) -> String {
        paths.count == 1 ? "Restore to Parent" : "Restore \(paths.count) Files to Parent"
    }

    private func deleteActionLabel(for paths: [String]) -> String {
        paths.count == 1 ? "Delete from Disk" : "Delete \(paths.count) Files from Disk"
    }

    private func ignoreActionLabel(for paths: [String]) -> String {
        paths.count == 1 ? "Ignore & Untrack" : "Ignore & Untrack \(paths.count) Files"
    }

    private func reviewActionLabel(for paths: [String]) -> String {
        if paths.allSatisfy(reviewedPaths.contains) {
            return paths.count == 1 ? "Mark as Unreviewed" : "Mark \(paths.count) Files as Unreviewed"
        }
        return paths.count == 1 ? "Mark as Reviewed" : "Mark \(paths.count) Files as Reviewed"
    }

    /// Marks carry the fingerprints of a diff that has been viewed; a file marked straight from the list keeps a whole-file baseline.
    func applyReviewMarks(paths: [String], reviewed: Bool) {
        guard showsReviewControls else { return }
        let hunks = paths.compactMap { path in
            detail.diff.first(where: { $0.path == path && !$0.reviewIdentity.isEmpty })
        }
        commitReviewMarks(hunks, reviewed: reviewed)
        refreshReviewedPaths()
    }

    func commitReviewMarks(_ hunks: [DiffHunk], reviewed: Bool) {
        for hunk in hunks {
            if reviewed {
                reviewStore.markReviewed(
                    changeId: reviewChangeId,
                    path: hunk.path,
                    identity: hunk.reviewIdentity,
                    snapshot: reviewSnapshots[hunk.reviewSnapshotKey]
                )
            } else {
                reviewStore.markUnreviewed(changeId: reviewChangeId, path: hunk.path)
            }
        }
    }

    func showInFinder(_ path: String) {
        RepositoryActions.showInFinder(repoPath: repoPath, path: path)
        selectSingleFile(path)
    }

    func loadAnnotate(rev: String, path: String) {
        guard let repo else { return }
        paneMode = .annotate(lines: [], path: path)
        Task.detached {
            let lines = try? repo.annotateFile(rev: rev, path: path)
            await MainActor.run {
                guard case let .annotate(_, currentPath) = paneMode, currentPath == path else { return }
                paneMode = .annotate(lines: lines ?? [], path: path)
            }
        }
    }

    func loadConflictedPaths() {
        guard let repo, detail.info.hasConflict else {
            conflictedPaths = []
            return
        }
        let rev = detailRevision
        Task.detached {
            let paths = (try? repo.resolveList(rev: rev)) ?? []
            await MainActor.run {
                conflictedPaths = Set(paths)
            }
        }
    }
}
