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
                if fileFilter.isEmpty {
                    Text(fileCountLabel)
                        .jayjayFont(11, weight: .medium)
                        .foregroundStyle(.secondary)
                } else {
                    Text(filteredFileCountLabel)
                        .jayjayFont(11, weight: .medium)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                if showsReviewControls, activeReviewNoteCount > 0 {
                    Button {
                        showNotedFilesOnly.toggle()
                    } label: {
                        HStack(spacing: 3) {
                            Image(systemName: "text.bubble.fill")
                                .jayjayFont(9)
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
                            ? "Showing only files with review notes"
                            : "Show only files with review notes"
                    )
                    .accessibilityIdentifier(AID.ReviewNote.activeCount(activeReviewNoteCount))
                }
                if showsReviewControls, !reviewableDiff.isEmpty, !reviewedPaths.isEmpty {
                    Text("\(reviewedPaths.count)/\(reviewableDiff.count)")
                        .jayjayFont(10, weight: .medium)
                        .foregroundStyle(.secondary)
                        .help("\(reviewedPaths.count) of \(reviewableDiff.count) files reviewed")
                    Button {
                        splitRequest = SplitSheetRequest(paths: Array(reviewedPaths))
                    } label: {
                        Image(systemName: "arrow.branch")
                            .foregroundStyle(.secondary)
                            .jayjayFont(11)
                    }
                    .buttonStyle(.plain)
                    .help("Split \(reviewedPaths.count) checked files to a new change")
                    .accessibilityIdentifier(AID.SplitSheet.openButton)
                }
                if showsReviewControls, !reviewedPaths.isEmpty {
                    Button {
                        hideReviewedFiles.toggle()
                    } label: {
                        Image(systemName: hideReviewedFiles ? "eye.slash.fill" : "eye.slash")
                            .foregroundStyle(hideReviewedFiles ? Color.accentColor : .secondary)
                            .jayjayFont(11)
                    }
                    .buttonStyle(.plain)
                    .help(hideReviewedFiles ? "Showing only unreviewed files" : "Hide reviewed files")
                }
                Button {
                    appSettings.treeFileList.toggle()
                } label: {
                    Image(systemName: appSettings.treeFileList ? "list.bullet.indent" : "list.bullet")
                        .foregroundStyle(appSettings.treeFileList ? Color.accentColor : .secondary)
                        .jayjayFont(11)
                }
                .buttonStyle(.plain)
                .help(appSettings.treeFileList ? "Showing files as a tree" : "Showing files as a flat list")
                Button {
                    showFileFilter.toggle()
                    if !showFileFilter {
                        fileFilter = ""
                    }
                } label: {
                    Image(systemName: "magnifyingglass")
                        .foregroundStyle(showFileFilter ? Color.accentColor : .secondary)
                        .jayjayFont(11)
                }
                .buttonStyle(.plain)
                .help("Filter files")
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 6)
            .frame(height: 40)

            if showFileFilter {
                HStack(spacing: 4) {
                    TextField("Filter files", text: $fileFilter)
                        .textFieldStyle(.roundedBorder).jayjayFont(11)
                        .focused($fileFilterFocused)
                        .onAppear { fileFilterFocused = true }
                    Button {
                        fileFilter = ""
                        showFileFilter = false
                    } label: {
                        Image(systemName: "xmark.circle.fill").foregroundStyle(.tertiary)
                    }
                    .buttonStyle(.plain)
                }
                .padding(.horizontal, 10)
                .padding(.bottom, 6)
            }

            Divider()

            if appSettings.treeFileList {
                treeFileList
            } else {
                flatFileList
            }
        }
        .background(
            KeyDownMonitor(
                isActive: { activePane == .fileColumn },
                onKeyDown: { event in handleFileColumnKey(event) }
            )
            .frame(width: 0, height: 0)
            .allowsHitTesting(false)
        )
    }

    private var flatFileList: some View {
        List(filteredDiff, id: \.path) { hunk in
            fileRowView(hunk: hunk)
                .listRowInsets(EdgeInsets(top: 0, leading: 4, bottom: 0, trailing: 4))
        }
        .listStyle(.plain)
        .scrollIndicators(.never)
        .id(detail.info.commitId)
    }

    private var treeFileList: some View {
        TreeFileList(
            filteredDiff: filteredDiff,
            commitId: detail.info.commitId.id,
            fileRowView: fileRowView
        )
    }

    func fileRowView(hunk: DiffHunk) -> some View {
        let noteCount = activeNoteCountsByPath[hunk.path] ?? 0
        return FileRow(
            hunk: hunk,
            isSelected: selectedPaths.contains(hunk.path),
            showReview: showsReviewControls && !hunk.isSubmodulePlaceholder && !hunk.reviewIdentity.isEmpty,
            reviewRollup: fileRollups[hunk.path] ?? .unreviewed,
            noteCount: noteCount,
            hasConflict: conflictedPaths.contains(hunk.path),
            onToggleReview: { toggleReview(hunk.path) }
        )
        .contentShape(Rectangle())
        .accessibilityIdentifier(AID.FileList.row(hunk.path))
        .accessibilityValue(fileRowAccessibilityValue(noteCount: noteCount))
        .onTapGesture {
            activePane = .fileColumn
            NSApp.keyWindow?.makeFirstResponder(nil)
            handleFileSelection(hunk.path)
        }
        .contextMenu {
            fileContextMenu(for: hunk.path)
        }
    }

    private func fileRowAccessibilityValue(noteCount: Int) -> String {
        noteCount > 0 ? noteCount.reviewNoteCountLabel : ""
    }

    func toggleReview(_ path: String) {
        guard let hunk = detail.diff.first(where: { $0.path == path }), !hunk.reviewIdentity.isEmpty else { return }
        reviewStore.toggleReviewed(
            changeId: reviewChangeId,
            path: path,
            identity: hunk.reviewIdentity,
            snapshot: reviewSnapshot(for: hunk)
        )
        refreshReviewedPaths()
    }

    func reviewSnapshot(for hunk: DiffHunk) -> ReviewFileSnapshot? {
        let snapshot = reviewSnapshotFromDiffHunk(hunk: hunk)
        if !snapshot.fingerprints.isEmpty {
            return snapshot
        }
        guard let repo else { return nil }
        return try? repo.reviewFileSnapshot(
            rev: detailRevision,
            path: hunk.path,
            oldPath: hunk.oldPath
        )
    }

    private var fileCountLabel: String {
        var parts = ["\(filteredDiff.count) files"]
        if hiddenGitLfsCount > 0 {
            parts.append("\(hiddenGitLfsCount) LFS hidden")
        }
        if hiddenSubmoduleCount > 0 {
            parts.append("\(hiddenSubmoduleCount) submodule hidden")
        }
        return parts.joined(separator: ", ")
    }

    private var filteredFileCountLabel: String {
        var parts = ["\(filteredDiff.count) of \(visibleDiff.count) files"]
        if hiddenGitLfsCount > 0 {
            parts.append("\(hiddenGitLfsCount) LFS hidden")
        }
        if hiddenSubmoduleCount > 0 {
            parts.append("\(hiddenSubmoduleCount) submodule hidden")
        }
        return parts.joined(separator: ", ")
    }
}
