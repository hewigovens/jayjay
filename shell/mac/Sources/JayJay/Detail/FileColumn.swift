import AppKit
import JayJayCore
import SwiftUI

extension ChangeDetailView {
    var reviewableDiff: [DiffHunk] {
        visibleDiff.filter { !$0.isSubmodulePlaceholder }
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
                if showsReviewControls, !reviewableDiff.isEmpty, !reviewedPaths.isEmpty {
                    Text("\(reviewedPaths.count)/\(reviewableDiff.count)")
                        .jayjayFont(10, weight: .medium)
                        .foregroundStyle(.secondary)
                    Button {
                        splitPaths = Array(reviewedPaths)
                        showSplitSheet = true
                    } label: {
                        Label("Split \(reviewedPaths.count)", systemImage: "arrow.branch")
                            .jayjayFont(10, weight: .medium)
                    }
                    .help("Split \(reviewedPaths.count) checked files to a new change")
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
                    showFileFilter.toggle()
                    if !showFileFilter { fileFilter = "" }
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

    func handleFileColumnKey(_ event: NSEvent) -> Bool {
        if event.keyCode == 49 { // Space
            return toggleReviewOnSelection()
        }
        switch event.keyCode {
            case 125: return moveFileSelection(by: 1) // Down arrow
            case 126: return moveFileSelection(by: -1) // Up arrow
            default: break
        }
        let isCtrl = event.modifierFlags.intersection(.deviceIndependentFlagsMask) == .control
        switch event.charactersIgnoringModifiers {
            case "j": return moveFileSelection(by: 1)
            case "k": return moveFileSelection(by: -1)
            case "n" where isCtrl: return moveFileSelection(by: 1)
            case "p" where isCtrl: return moveFileSelection(by: -1)
            default: return false
        }
    }

    private func toggleReviewOnSelection() -> Bool {
        let selectedReviewablePaths = selectedPaths
            .filter { path in reviewableDiff.contains(where: { $0.path == path }) }
            .sorted()
        guard showsReviewControls, !selectedReviewablePaths.isEmpty else { return false }
        for path in selectedReviewablePaths {
            toggleReview(path)
        }
        if let primaryPath = selectedPath,
           reviewedPaths.contains(primaryPath),
           let next = filteredDiff.first(where: { !reviewedPaths.contains($0.path) })
        {
            selectedPath = next.path
            selectedPaths = [next.path]
            fileSelectionAnchorPath = next.path
        }
        return true
    }

    @discardableResult
    func moveFileSelection(by delta: Int) -> Bool {
        guard !filteredDiff.isEmpty else { return false }
        let currentIdx: Int = if let cur = selectedPath, let idx = filteredDiff.firstIndex(where: { $0.path == cur }) {
            idx
        } else {
            delta > 0 ? -1 : filteredDiff.count
        }
        let newIdx = max(0, min(filteredDiff.count - 1, currentIdx + delta))
        guard newIdx != currentIdx else { return false }
        let nextPath = filteredDiff[newIdx].path
        selectedPath = nextPath
        selectedPaths = [nextPath]
        fileSelectionAnchorPath = nextPath
        return true
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
        FileRow(
            hunk: hunk,
            isSelected: selectedPaths.contains(hunk.path),
            showReview: showsReviewControls && !hunk.isSubmodulePlaceholder,
            isReviewed: reviewedPaths.contains(hunk.path),
            hasConflict: conflictedPaths.contains(hunk.path),
            onToggleReview: { toggleReview(hunk.path) }
        )
        .contentShape(Rectangle())
        .accessibilityIdentifier(AID.FileList.row(hunk.path))
        .onTapGesture {
            activePane = .fileColumn
            NSApp.keyWindow?.makeFirstResponder(nil)
            handleFileSelection(hunk.path)
        }
        .contextMenu {
            fileContextMenu(for: hunk.path)
        }
    }

    func toggleReview(_ path: String) {
        guard let hunk = detail.diff.first(where: { $0.path == path }) else { return }
        reviewStore.toggleReviewed(
            changeId: detailRevision,
            path: path,
            identity: hunk.reviewIdentity
        )
        if reviewedPaths.contains(path) {
            reviewedPaths.remove(path)
        } else {
            reviewedPaths.insert(path)
        }
    }

    private var visibleSelectablePaths: [String] {
        if appSettings.treeFileList {
            let visibleHunks = filteredDiff
            let entries = buildFileTree(paths: visibleHunks.map(\.path))
            return entries.compactMap { entry in
                guard let hunkIndex = entry.hunkIndex, Int(hunkIndex) < visibleHunks.count else { return nil }
                return visibleHunks[Int(hunkIndex)].path
            }
        }
        return filteredDiff.map(\.path)
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

    func contextSelectionPaths(for clickedPath: String) -> [String] {
        let activeSelection: Set<String> =
            if selectedPaths.contains(clickedPath), selectedPaths.count > 1 {
                selectedPaths
            } else {
                [clickedPath]
            }
        return visibleSelectablePaths.filter(activeSelection.contains)
    }

    func handleFileSelection(_ path: String) {
        let modifiers = NSEvent.modifierFlags.intersection(.deviceIndependentFlagsMask)
        let orderedPaths = visibleSelectablePaths

        if modifiers.contains(.shift),
           let anchor = fileSelectionAnchorPath,
           let anchorIndex = orderedPaths.firstIndex(of: anchor),
           let currentIndex = orderedPaths.firstIndex(of: path)
        {
            let lower = min(anchorIndex, currentIndex)
            let upper = max(anchorIndex, currentIndex)
            selectedPaths = Set(orderedPaths[lower ... upper])
        } else {
            selectedPaths = [path]
            fileSelectionAnchorPath = path
        }

        selectedPath = path
    }

    func selectSingleFile(_ path: String) {
        selectedPath = path
        selectedPaths = [path]
        fileSelectionAnchorPath = path
    }
}
