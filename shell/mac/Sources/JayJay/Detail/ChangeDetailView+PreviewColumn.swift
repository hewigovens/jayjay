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

    var changeHeader: some View {
        VStack(alignment: .leading, spacing: 0) {
            if isCompareMode {
                compareBanner
            } else {
                detailHeader
            }

            Divider()

            if !isCompareMode, detail.info.isDivergent, let onResolveDivergent {
                divergentBar(onResolve: onResolveDivergent)
                Divider()
            }

            if !staleOrOrphanedReviewNotes.isEmpty {
                staleReviewNotesSection
                Divider()
            }
        }
        .background(appSettings.readingBackground(colorScheme))
    }

    // MARK: - Preview column

    private func diffSection(for hunk: DiffHunk, stacked: Bool = false) -> some View {
        VStack(spacing: 0) {
            if conflictedPaths.contains(hunk.path) {
                conflictBar(hunk: hunk)
            }
            DiffSection(
                hunk: hunk,
                rev: detailRevision,
                commitId: detail.info.commitId.id,
                reviewChangeId: showsReviewControls ? reviewChangeId : nil,
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
                fileReview: .chrome(
                    showReview: showsReviewMarks && hunk.isReviewable,
                    rollup: fileRollups[hunk.path] ?? .unreviewed
                ),
                onToggleFileReview: { toggleReview([hunk.path]) },
                compareFromRev: comparison?.fromId,
                lineStats: fileStats?.value?[hunk.path],
                fitsContent: stacked,
                showsSharedActions: false,
                onShowAlone: stacked ? { selectSingleFile(hunk.path) } : nil,
                stackedHeights: stacked ? stackedHeights : nil
            )
            // Rebuild DiffSection on commit-id change so Abandon-Selected-Lines refreshes @State fileDiff.
            .id("\(detail.info.commitId.id)|\(hunk.oldPath ?? "")|\(hunk.path)")
        }
    }

    /// The preview column shows the selected files, or every file while none is selected and all diffs are shown together.
    var shownHunks: [DiffHunk] {
        let selected = fileSelection.paths
        let showsAll = selected.isEmpty && appSettings.showAllFileDiffs
        guard showsAll || selected.count > 1 else { return selectedHunk.map { [$0] } ?? [] }
        let hunks = Dictionary(filteredDiff.map { ($0.path, $0) }, uniquingKeysWith: { first, _ in first })
        return showsAll
            ? displayOrderedPaths.compactMap { hunks[$0] }
            : visibleSelectablePaths.compactMap { selected.contains($0) ? hunks[$0] : nil }
    }

    var isStacked: Bool {
        shownHunks.count > 1
    }

    var previewColumn: some View {
        let shown = shownHunks
        return VStack(alignment: .leading, spacing: 0) {
            if case let .annotate(lines, path, error) = paneMode {
                AnnotateView(
                    lines: lines, path: path, error: error,
                    onSelectChange: { changeId in
                        paneMode = .files
                        if let onRevealChangeInDag {
                            onRevealChangeInDag(changeId)
                        } else {
                            actions?.select(changeId: changeId)
                        }
                    },
                    onRetry: { loadAnnotate(rev: detailRevision, path: path) },
                    onDismiss: { paneMode = .files }
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else if shown.count > 1 {
                ScrollViewReader { proxy in
                    ScrollView {
                        LazyVStack(alignment: .leading, spacing: 0) {
                            ForEach(shown, id: \.path) { hunk in
                                diffSection(for: hunk, stacked: true)
                                Divider()
                            }
                        }
                        .background(EnclosingScrollView(handle: stackScroll))
                    }
                    // The stack only estimates the height of files it has not shown, so the jump repeats until the files it lands on stop resizing it.
                    .task(id: stackScrollEdge) {
                        guard let edge = stackScrollEdge, let target = edge == .bottom ? shown.last : shown.first else { return }
                        var height: CGFloat?
                        var settledPasses = 0
                        for _ in 0 ..< 40 where !Task.isCancelled && settledPasses < 5 {
                            proxy.scrollTo(target.path, anchor: edge == .bottom ? .bottom : .top)
                            try? await Task.sleep(for: .milliseconds(120))
                            let newHeight = stackScroll.scrollView?.documentView?.frame.height
                            settledPasses = newHeight == height ? settledPasses + 1 : 0
                            height = newHeight
                        }
                        if !Task.isCancelled {
                            stackScrollEdge = nil
                        }
                    }
                }
            } else if let hunk = shown.first {
                diffSection(for: hunk)
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            } else if visibleDiff.isEmpty, hiddenDiffCount > 0 {
                emptyState(
                    hiddenOnlyStateTitle,
                    icon: hiddenOnlyStateIcon,
                    description: hiddenOnlyStateDescription,
                    action: hiddenGitLfsCount > 0 ? (Localized.Repo.showGitLfsFiles, { appSettings.hideGitLfsDiffs = false }) : nil
                )
            } else if visibleDiff.isEmpty {
                emptyState(Localized.Repo.noFilesChanged, icon: "doc", description: Localized.Repo.thisRevisionDoesNotModifyAny)
                    .accessibilityIdentifier(AID.Detail.noFilesChanged)
            } else {
                unlistedFilesState
            }
        }
        .frame(maxHeight: .infinity)
        .background(appSettings.readingBackground(colorScheme))
        .overlay {
            if conflictEditor?.isPreparing == true {
                LoadingHUD(accessibilityIdentifier: AID.Conflict.editorPreparing)
            } else if fileEditor?.isPreparing == true {
                LoadingHUD(accessibilityIdentifier: AID.FileEditor.preparing)
            }
        }
    }

    func prepareFileEditor(path: String, line: UInt32?) {
        guard fileEditor == nil, conflictEditor == nil, let repo else { return }
        let presentation = EditorPresentation<WorkingCopyFileEditorSession>(revision: detailRevision)
        fileEditor = presentation
        let session = WorkingCopyFileEditorSession(repo: repo, path: path, line: line)
        Task {
            await session.load()
            guard fileEditor?.id == presentation.id else { return }
            if shownHunks.contains(where: { $0.path == path }) {
                fileEditor?.session = session
            } else {
                fileEditor = nil
            }
        }
    }

    func openPendingConflictEditor() {
        guard let path = actions?.takePendingConflictEditor(commitId: detail.info.commitId.id),
              visibleDiff.contains(where: { $0.path == path })
        else { return }
        selectFile(path)
        prepareConflictEditor(path: path)
    }

    func showRequestedFile() {
        guard let request = actions?.fileRevealRequest, request.revision == detailRevision else { return }
        if !paneMode.isDiffEdit, visibleDiff.contains(where: { $0.path == request.path }) {
            selectFile(request.path)
        }
        actions?.finishFileReveal(shown: !paneMode.isDiffEdit && selectedHunk?.path == request.path)
    }

    private func selectFile(_ path: String) {
        if !filteredDiff.contains(where: { $0.path == path }) {
            fileFilter = ""
            showNotedFilesOnly = false
        }
        fileSelection.primary = path
        fileSelection.paths = [path]
    }

    func prepareConflictEditor(path: String) {
        guard fileEditor == nil,
              conflictEditor == nil,
              let repo,
              let hunk = shownHunks.first(where: { $0.path == path }),
              Self.canEnterConflictEditor(info: detail.info, hunk: hunk, isCompareMode: isCompareMode)
        else { return }
        let presentation = EditorPresentation<ConflictEditorSession>(revision: detailRevision)
        conflictEditor = presentation
        let target = ConflictEditorTarget(repo: repo, rev: detailRevision, path: path)
        let session = ConflictEditorSession(target: target)
        Task {
            await session.load()
            guard conflictEditor?.id == presentation.id else { return }
            if shownHunks.contains(where: { $0.path == path }) {
                conflictEditor?.session = session
            } else {
                conflictEditor = nil
            }
        }
    }

    /// The list is empty while the change has files: name the first filter that emptied it and offer to undo it.
    @ViewBuilder private var unlistedFilesState: some View {
        let matches = fileFilterMatches
        let noted = showNotedFilesOnly && showsReviewControls
            ? matches.filter { activeNoteCountsByPath[$0.path] != nil }
            : matches
        if matches.isEmpty {
            emptyState(
                Localized.Repo.noMatchingFilesTitle,
                icon: "magnifyingglass",
                description: Localized.Repo.noFilePathContains(query: fileFilter),
                action: (Localized.Repo.clearFilter, dismissFileFilter)
            )
        } else if noted.isEmpty {
            emptyState(
                Localized.Repo.noFilesWithReviewNotes,
                icon: "text.bubble",
                description: fileFilter.isEmpty
                    ? Localized.Repo.noFileHasAnOpenReviewNote
                    : Localized.Repo.noMatchingFileHasAnOpenReviewNote(query: fileFilter),
                action: (Localized.Repo.showAllFiles, { showNotedFilesOnly = false })
            )
        } else {
            emptyState(
                Localized.Repo.allFilesReviewed,
                icon: "checkmark.circle",
                description: Localized.Repo.reviewedFilesHidden(count: noted.count),
                action: (Localized.Repo.showReviewedFiles, { appSettings.hideReviewedFiles = false })
            )
        }
    }

    private func emptyState(_ title: String, icon: String, description: String, action: (String, () -> Void)? = nil) -> some View {
        ContentUnavailableView {
            Label(title, systemImage: icon)
        } description: {
            if case let (label, perform)? = action {
                let button = Button(label, action: perform)
                    .buttonStyle(InlineTextButtonStyle())
                    .keyboardFocusStop(.emptyStateAction, action: perform)
                    .accessibilityIdentifier(AID.Detail.emptyStateAction)
                ViewThatFits(in: .horizontal) {
                    HStack(alignment: .firstTextBaseline, spacing: 4) {
                        Text(description)
                        button
                    }
                    VStack(spacing: 4) {
                        Text(description)
                        button
                    }
                }
            } else {
                Text(description)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
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
        hiddenGitLfsCount > 0 ? "externaldrive.badge.questionmark" : "square.stack.3d.up.slash"
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
