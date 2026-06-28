import JayJayCore
import SwiftUI

extension ChangeDetailView {
    // MARK: - Empty state

    var emptyState: some View {
        VStack(alignment: .leading, spacing: 16) {
            headerSection
            descriptionSection
            Divider()
            if visibleDiff.isEmpty, hiddenDiffCount > 0 {
                ContentUnavailableView(
                    hiddenOnlyStateTitle,
                    systemImage: hiddenOnlyStateIcon,
                    description: Text(hiddenOnlyStateDescription)
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ContentUnavailableView(
                    "No Files Changed",
                    systemImage: "doc.badge.minus",
                    description: Text("This revision does not modify any tracked files.")
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .padding(18)
    }

    // MARK: - Preview column

    var previewColumn: some View {
        VStack(alignment: .leading, spacing: 0) {
            if isCompareMode {
                compareBanner
            }
            VStack(alignment: .leading, spacing: 12) {
                if !isCompareMode {
                    headerSection
                    descriptionSection
                }
            }
            .padding(.horizontal, 18)
            .padding(.top, isCompareMode ? 4 : 14)
            .padding(.bottom, 8)
            .zIndex(1)

            Divider()

            if !staleOrOrphanedReviewNotes.isEmpty {
                staleReviewNotesSection
                Divider()
            }

            if case let .annotate(lines, path) = paneMode {
                AnnotateView(
                    lines: lines, path: path,
                    onSelectChange: { changeId in
                        paneMode = .files
                        if let onRevealChangeInDag {
                            onRevealChangeInDag(changeId)
                        } else {
                            actions?.select(changeId: changeId)
                        }
                    },
                    onDismiss: { paneMode = .files }
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if case let .fileHistory(history, path) = paneMode {
                FileHistoryView(
                    history: history, path: path,
                    onSelectChange: { changeId in
                        paneMode = .files
                        if let onRevealChangeInDag {
                            onRevealChangeInDag(changeId)
                        } else {
                            actions?.select(changeId: changeId)
                        }
                    },
                    onDismiss: { paneMode = .files }
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if let hunk = selectedHunk {
                VStack(spacing: 0) {
                    if conflictedPaths.contains(hunk.path) {
                        conflictBar(path: hunk.path)
                    }
                    DiffSection(
                        hunk: hunk,
                        rev: detailRevision,
                        commitId: detail.info.commitId.id,
                        reviewChangeId: reviewChangeId,
                        repo: repo,
                        actions: actions,
                        isWorkingCopy: detail.info.isWorkingCopy,
                        diffStore: diffStore,
                        reviewStore: reviewStore,
                        staleNoteIds: staleReviewNoteIds,
                        noteEditor: $noteEditor,
                        onOpenDiffEdit: {
                            paneMode = .diffEdit
                        },
                        onReviewStateChanged: { refreshReviewState() },
                        compareFromRev: compareFromId
                    )
                    // Rebuild DiffSection on commit-id change so Abandon-Selected-Lines refreshes @State fileDiff.
                    .id("\(detail.info.commitId)|\(hunk.path)")
                    .padding(.horizontal, 18)
                    .padding(.top, 10)
                    .padding(.bottom, 6)
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                }
            } else {
                Spacer()
                ContentUnavailableView(
                    "Select a File",
                    systemImage: "doc.text.magnifyingglass",
                    description: Text("Choose a file to inspect.")
                )
                .frame(maxWidth: .infinity)
                Spacer()
            }
        }
        .frame(maxHeight: .infinity)
    }

    private var hiddenOnlyStateTitle: String {
        if hiddenGitLfsCount > 0, hiddenSubmoduleCount > 0 {
            return "Only Git-managed Changes Hidden"
        }
        if hiddenGitLfsCount > 0 {
            return "Only Git LFS-backed Files Hidden"
        }
        return "Only Git Submodule Changes Hidden"
    }

    private var hiddenOnlyStateIcon: String {
        if hiddenGitLfsCount > 0, hiddenSubmoduleCount > 0 {
            return "externaldrive.badge.questionmark"
        }
        if hiddenGitLfsCount > 0 {
            return "externaldrive.badge.questionmark"
        }
        return "square.stack.3d.up.slash"
    }

    private var hiddenOnlyStateDescription: String {
        if hiddenGitLfsCount > 0, hiddenSubmoduleCount > 0 {
            return "This revision only changes Git LFS-backed files and submodules, and the current diff settings hide them."
        }
        if hiddenGitLfsCount > 0 {
            return "This revision only changes Git LFS-backed files, and the current diff setting hides them."
        }
        return "This revision only changes Git submodules, and the current settings hide them."
    }
}
