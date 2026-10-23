import AppKit
import JayJayCore
import SwiftUI

extension ChangeDetailView {
    var reviewableDiff: [DiffHunk] {
        visibleDiff.filter { !$0.isSubmodulePlaceholder && !$0.reviewIdentity.isEmpty }
    }

    var fileColumn: some View {
        VStack(spacing: 0) {
            HStack(spacing: 6) {
                if showsReviewControls, activeReviewNoteCount > 0 {
                    Button {
                        showNotedFilesOnly.toggle()
                    } label: {
                        HStack(spacing: 3) {
                            Image(systemName: "text.bubble.fill")
                                .jayjayFont(14)
                            Text("\(activeReviewNoteCount)")
                                .jayjayFont(10, weight: .medium)
                        }
                        .foregroundStyle(.orange)
                        .padding(.horizontal, 5)
                        .padding(.vertical, 2)
                        .background(
                            showNotedFilesOnly
                                ? AnyShapeStyle(Color.orange.opacity(0.16))
                                : AnyShapeStyle(Color.clear),
                            in: RoundedRectangle(cornerRadius: 4, style: .continuous)
                        )
                    }
                    .buttonStyle(.plain)
                    .help(
                        showNotedFilesOnly
                            ? Localized.Repo.showingOnlyFilesWithReviewNotes
                            : Localized.Repo.showOnlyFilesWithReviewNotes
                    )
                    .accessibilityIdentifier(AID.ReviewNote.activeCount(activeReviewNoteCount))
                }
                if showsReviewedCount {
                    Button {
                        splitRequest = SplitSheetRequest(
                            paths: reviewedPaths.flatMap { path in
                                detail.diff.first(where: { $0.path == path })?.changedPaths ?? [path]
                            }
                        )
                    } label: {
                        Image(systemName: "arrow.branch")
                            .foregroundStyle(.secondary)
                            .jayjayFont(14)
                    }
                    .buttonStyle(.plain)
                    .help(Localized.Repo.splitCheckedFiles(count: reviewedPaths.count))
                    .accessibilityIdentifier(AID.SplitSheet.openButton)
                }
                if showsReviewControls, !reviewedPaths.isEmpty || appSettings.hideReviewedFiles {
                    Button {
                        appSettings.hideReviewedFiles.toggle()
                    } label: {
                        Image(systemName: appSettings.hideReviewedFiles ? "eye.slash.fill" : "eye.slash")
                            .foregroundStyle(appSettings.hideReviewedFiles ? Color.accentColor : .secondary)
                            .jayjayFont(14)
                    }
                    .buttonStyle(.plain)
                    .help(appSettings.hideReviewedFiles ? Localized.Repo.showingOnlyUnreviewedFiles : Localized.Settings.hideReviewed)
                }
                Button {
                    appSettings.treeFileList.toggle()
                } label: {
                    Image(systemName: appSettings.treeFileList ? "list.bullet.indent" : "list.bullet")
                        .foregroundStyle(appSettings.treeFileList ? Color.accentColor : .secondary)
                        .jayjayFont(14)
                }
                .buttonStyle(.plain)
                .keyboardFocusStop(.treeToggle) { appSettings.treeFileList.toggle() }
                .help(appSettings.treeFileList ? Localized.Repo.showingFilesAsATree : Localized.Repo.showingFilesAsAFlatList)
                Button {
                    toggleFileFilter()
                } label: {
                    Image(systemName: "magnifyingglass")
                        .foregroundStyle(showFileFilter ? Color.accentColor : .secondary)
                        .jayjayFont(14)
                }
                .buttonStyle(.plain)
                .keyboardFocusStop(.filterToggle, action: toggleFileFilter)
                .help(Localized.Repo.filterFiles)
                .accessibilityIdentifier(AID.FileList.filterToggle)
                Spacer(minLength: 8)
                Text(fileCountLabel)
                    .jayjayFont(12)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                    .help(showsReviewedCount ? reviewedCountHelp : "")
            }
            .padding(.horizontal, 14)
            .padding(.vertical, 6)
            .frame(minHeight: PaneLayout.headerHeight)

            if showFileFilter {
                HStack(spacing: 4) {
                    FilterField(text: $fileFilter, onSubmit: focusFileList, onCancel: dismissFileFilter)
                    Button {
                        dismissFileFilter()
                    } label: {
                        Image(systemName: "xmark.circle.fill").foregroundStyle(.tertiary)
                    }
                    .buttonStyle(.plain)
                }
                .padding(.horizontal, 14)
                .padding(.bottom, 6)
            }

            Divider()

            if filteredDiff.isEmpty {
                Text(emptyFileListLabel)
                    .jayjayFont(12)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                    .accessibilityIdentifier(AID.FileList.empty)
            } else if appSettings.treeFileList {
                treeFileList
            } else {
                flatFileList
            }
        }
        .scrollContentBackground(.hidden)
        .background(appSettings.readingBackground(colorScheme))
        .keyboardFocusStop(.fileList, action: focusFileList)
        .background(
            KeyDownMonitor(
                isActive: { activePane == .fileColumn && keyboardFocus?.control == nil },
                swallowsUnhandledKeys: true,
                onKeyDown: { event in handleFileColumnKey(event) }
            )
            .frame(width: 0, height: 0)
            .allowsHitTesting(false)
        )
    }

    private var flatFileList: some View {
        ScrollViewReader { proxy in
            List(filteredDiff, id: \.path) { hunk in
                fileRowView(hunk: hunk)
                    .listRowInsets(FileRow.listInsets)
            }
            .listStyle(.plain)
            .scrollIndicators(.never)
            .revealing(selectedPath, again: contextMenuRequest?.id, with: proxy)
            .id(detail.info.commitId)
        }
    }

    private var treeFileList: some View {
        TreeFileList(
            filteredDiff: filteredDiff,
            treeEntries: fileTreeCache.entries(for: filteredDiff.map(\.path)),
            commitId: detail.info.commitId.id,
            selectedPath: selectedPath,
            revealTrigger: contextMenuRequest?.id,
            collapsedDirs: $collapsedDirs,
            fileRowView: fileRowView
        )
    }

    func fileRowView(hunk: DiffHunk) -> some View {
        let noteCount = activeNoteCountsByPath[hunk.path] ?? 0
        return FileRow(
            hunk: hunk,
            isSelected: selectedPaths.contains(hunk.path),
            isPaneActive: activePane == .fileColumn,
            showReview: showsReviewControls && !hunk.isSubmodulePlaceholder && !hunk.reviewIdentity.isEmpty,
            reviewRollup: fileRollups[hunk.path] ?? .unreviewed,
            agentMarked: agentMarkedPaths.contains(hunk.path),
            noteCount: noteCount,
            hasConflict: conflictedPaths.contains(hunk.path),
            onToggleReview: { toggleReview(hunk.path) }
        )
        .contentShape(Rectangle())
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier(AID.FileList.row(hunk.path))
        .accessibilityAddTraits(selectedPaths.contains(hunk.path) ? .isSelected : [])
        .onTapGesture {
            activePane = .fileColumn
            NSApp.keyWindow?.makeFirstResponder(nil)
            handleFileSelection(hunk.path)
        }
        .selectionContextMenu(for: hunk.path, request: $contextMenuRequest) {
            fileContextMenu(for: hunk.path)
        }
    }

    var reviewedCountHelp: String {
        let agentMarked = agentMarkedPaths.filter { path in
            fileRollups[path].map { $0 != .unreviewed } ?? false
        }.count
        let base = Localized.Repo.filesReviewedProgress(reviewed: reviewedPaths.count, total: reviewableDiff.count)
        return agentMarked > 0 ? "\(base), \(agentMarked) include agent marks" : base
    }

    func toggleReview(_ path: String) {
        applyReviewMarks(paths: [path], reviewed: fileRollups[path] != .reviewed)
    }

    private var showsReviewedCount: Bool {
        showsReviewControls && !reviewableDiff.isEmpty && !reviewedPaths.isEmpty
    }

    private var emptyFileListLabel: String {
        if detail.diff.isEmpty {
            return Localized.Repo.noFiles
        }
        if !fileFilter.isEmpty {
            return Localized.Repo.noMatchingFiles
        }
        return Localized.Repo.allFilesHidden
    }

    private var fileCountLabel: String {
        var parts = if !fileFilter.isEmpty {
            [Localized.Repo.filesFiltered(shown: filteredDiff.count, total: visibleDiff.count)]
        } else if showsReviewedCount {
            ["\(reviewedPaths.count)/\(reviewableDiff.count) reviewed"]
        } else {
            [Localized.Repo.fileCount(count: filteredDiff.count)]
        }
        if hiddenGitLfsCount > 0 {
            parts.append("\(hiddenGitLfsCount) LFS hidden")
        }
        if hiddenSubmoduleCount > 0 {
            parts.append("\(hiddenSubmoduleCount) submodule hidden")
        }
        return parts.joined(separator: ", ")
    }
}
