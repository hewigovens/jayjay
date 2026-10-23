import JayJayCore
import SwiftUI

extension ChangeDetailView {
    private var detailHeader: some View {
        VStack(alignment: .leading, spacing: descriptionExpanded.wrappedValue ? 12 : 0) {
            descriptionSection()
            metadataSection
                .frame(minHeight: descriptionExpanded.wrappedValue ? nil : PaneLayout.fileRowHeight(scale: typography.interfaceScale))
                .animation(reduceMotion ? nil : .smooth(duration: 0.35, extraBounce: 0), value: descriptionExpanded.wrappedValue)
        }
        .padding(.horizontal, PaneLayout.detailInset)
        .padding(.bottom, descriptionExpanded.wrappedValue ? 18 : 0)
        .zIndex(1)
    }

    // MARK: - Preview column

    var previewColumn: some View {
        VStack(alignment: .leading, spacing: 0) {
            if isCompareMode {
                compareBanner
            } else {
                detailHeader
            }

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
            } else if let hunk = selectedHunk {
                VStack(spacing: 0) {
                    if conflictedPaths.contains(hunk.path) {
                        conflictBar(hunk: hunk)
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
                        onOpenDiffEdit: canEnterDiffEdit ? { paneMode = .diffEdit } : nil,
                        onEditFile: canEditWorkingCopyFile(hunk)
                            ? { prepareFileEditor(path: hunk.path, line: $0) }
                            : nil,
                        onOpenInExternalEditor: canEditWorkingCopyFile(hunk)
                            ? { appSettings.openInEditor(filePath: hunk.path, repoPath: repoPath, line: $0) }
                            : nil,
                        onReviewStateChanged: { refreshReviewState() },
                        onReviewSnapshotLoaded: { loadedHunk, snapshot in
                            rememberLoadedReviewSnapshot(for: loadedHunk, snapshot: snapshot)
                        },
                        compareFromRev: compareFromId,
                        lineStats: fileStats[hunk.path]
                    )
                    // Rebuild DiffSection on commit-id change so Abandon-Selected-Lines refreshes @State fileDiff.
                    .id("\(detail.info.commitId)|\(hunk.path)")
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                }
            } else if visibleDiff.isEmpty, hiddenDiffCount > 0 {
                ContentUnavailableView(
                    hiddenOnlyStateTitle,
                    systemImage: hiddenOnlyStateIcon,
                    description: Text(hiddenOnlyStateDescription)
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if visibleDiff.isEmpty {
                ContentUnavailableView(
                    Localized.Repo.noFilesChanged,
                    systemImage: "doc.badge.minus",
                    description: Text(Localized.Repo.thisRevisionDoesNotModifyAny)
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                Spacer()
                ContentUnavailableView(
                    Localized.Repo.selectAFile,
                    systemImage: "doc.text.magnifyingglass",
                    description: Text(Localized.Repo.chooseAFileToInspect)
                )
                .frame(maxWidth: .infinity)
                Spacer()
            }
        }
        .frame(maxHeight: .infinity)
        .background(appSettings.readingBackground(colorScheme))
        .overlay {
            if conflictEditorPreparation != nil {
                LoadingHUD(accessibilityIdentifier: AID.Conflict.editorPreparing)
            } else if fileEditorPreparation != nil {
                LoadingHUD(accessibilityIdentifier: AID.FileEditor.preparing)
            }
        }
    }

    func prepareFileEditor(path: String, line: UInt32?) {
        guard fileEditorPreparation == nil, conflictEditorPreparation == nil, let repo else { return }
        let request = EditorPreparationRequest(revision: detailRevision)
        fileEditorPreparation = request
        let session = WorkingCopyFileEditorSession(repo: repo, path: path, line: line)
        Task {
            await session.load()
            guard fileEditorPreparation == request else { return }
            fileEditorPreparation = nil
            guard selectedHunk?.path == path else { return }
            fileEditor = session
        }
    }

    func prepareConflictEditor(path: String) {
        guard conflictEditorPreparation == nil,
              fileEditorPreparation == nil,
              let repo,
              let hunk = selectedHunk,
              hunk.path == path,
              Self.canEnterConflictEditor(info: detail.info, hunk: hunk, isCompareMode: isCompareMode)
        else { return }
        let request = EditorPreparationRequest(revision: detailRevision)
        conflictEditorPreparation = request
        let target = ConflictEditorTarget(repo: repo, rev: detailRevision, path: path)
        let session = ConflictEditorSession(target: target)
        Task {
            await session.load()
            guard conflictEditorPreparation == request else { return }
            conflictEditorPreparation = nil
            guard selectedHunk?.path == path else { return }
            conflictEditor = session
        }
    }

    private var hiddenOnlyStateTitle: String {
        if hiddenGitLfsCount > 0, hiddenSubmoduleCount > 0 {
            return Localized.Repo.onlyGitManagedChangesHidden
        }
        if hiddenGitLfsCount > 0 {
            return Localized.Repo.onlyGitLfsBackedFilesHidden
        }
        return Localized.Repo.onlyGitSubmoduleChangesHidden
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
            return Localized.Repo.thisRevisionOnlyChangesGitLfs
        }
        if hiddenGitLfsCount > 0 {
            return Localized.Repo.thisRevisionOnlyChangesGitLfsBacked
        }
        return Localized.Repo.thisRevisionOnlyChangesGitSubmodules
    }
}
