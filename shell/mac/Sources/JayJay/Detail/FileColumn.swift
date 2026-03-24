import JayJayCore
import SwiftUI

extension ChangeDetailView {
    var fileColumn: some View {
        VStack(spacing: 0) {
            HStack(spacing: 6) {
                if fileFilter.isEmpty {
                    Text("\(detail.diff.count) files")
                        .jayjayFont(11, weight: .medium)
                        .foregroundStyle(.secondary)
                } else {
                    Text("\(filteredDiff.count) of \(detail.diff.count) files")
                        .jayjayFont(11, weight: .medium)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                if detail.info.isWorkingCopy, !reviewedPaths.isEmpty {
                    Text("\(reviewedPaths.count)/\(detail.diff.count)")
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
        .focused($fileColumnFocused)
        .onKeyPress(.space) {
            guard detail.info.isWorkingCopy, !selectedPaths.isEmpty else { return .ignored }
            for path in selectedPaths.sorted() {
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
            return .handled
        }
        .onKeyPress(.upArrow) {
            guard let cur = selectedPath, let i = filteredDiff.firstIndex(where: { $0.path == cur }),
                  i > 0 else { return .ignored }
            selectedPath = filteredDiff[i - 1].path
            selectedPaths = [filteredDiff[i - 1].path]
            fileSelectionAnchorPath = filteredDiff[i - 1].path
            return .handled
        }
        .onKeyPress(.downArrow) {
            guard let cur = selectedPath, let i = filteredDiff.firstIndex(where: { $0.path == cur }),
                  i < filteredDiff.count - 1 else { return .ignored }
            selectedPath = filteredDiff[i + 1].path
            selectedPaths = [filteredDiff[i + 1].path]
            fileSelectionAnchorPath = filteredDiff[i + 1].path
            return .handled
        }
    }

    private var flatFileList: some View {
        List(filteredDiff, id: \.path) { hunk in
            fileRowView(hunk: hunk)
                .listRowInsets(EdgeInsets(top: 0, leading: 4, bottom: 0, trailing: 4))
        }
        .listStyle(.plain)
        .id(detail.info.commitId)
    }

    private var treeFileList: some View {
        let treeEntries = buildFileTree(paths: detail.diff.map(\.path))
        return List {
            ForEach(treeEntries, id: \.path) { entry in
                if let hunkIdx = entry.hunkIndex, Int(hunkIdx) < detail.diff.count {
                    let hunk = detail.diff[Int(hunkIdx)]
                    fileRowView(hunk: hunk)
                        .padding(.leading, CGFloat(entry.depth) * 12)
                        .tag(hunk.path)
                } else {
                    HStack(spacing: 4) {
                        Image(systemName: "folder").foregroundStyle(.secondary).jayjayFont(11)
                        Text(entry.name).jayjayFont(12, weight: .medium)
                    }
                    .padding(.leading, CGFloat(entry.depth) * 12)
                }
            }
        }
        .listStyle(.plain)
        .id(detail.info.commitId)
    }

    func fileRowView(hunk: DiffHunk) -> some View {
        FileRow(
            hunk: hunk,
            isSelected: selectedPaths.contains(hunk.path),
            showReview: detail.info.isWorkingCopy,
            isReviewed: reviewedPaths.contains(hunk.path),
            hasConflict: conflictedPaths.contains(hunk.path),
            onToggleReview: { toggleReview(hunk.path) }
        )
        .contentShape(Rectangle())
        .onTapGesture {
            handleFileSelection(hunk.path)
            fileColumnFocused = true
        }
        .contextMenu {
            fileContextMenu(for: hunk.path)
        }
    }

    @ViewBuilder
    private func fileContextMenu(for path: String) -> some View {
        let contextPaths = contextSelectionPaths(for: path)
        let isBatch = contextPaths.count > 1
        let reviewLabel = reviewActionLabel(for: contextPaths)

        if !isBatch {
            Button("Open in \(appSettings.externalEditor.title)") {
                appSettings.openInEditor(filePath: path, repoPath: repoPath)
            }
            Button("Show in Finder") { showInFinder(path) }
            Button("Copy Path") {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(path, forType: .string)
            }
            Divider()
            Button("Annotate (Blame)") { loadAnnotate(rev: detail.info.changeId, path: path) }
            Button("File History") { loadFileHistory(path: path) }
            Divider()
        }

        if detail.info.isWorkingCopy {
            Button(reviewLabel) {
                setReviewState(for: contextPaths, reviewed: !contextPaths.allSatisfy(reviewedPaths.contains))
            }
            Divider()
        }

        Button(splitActionLabel(for: contextPaths)) {
            splitPaths = contextPaths
            showSplitSheet = true
        }
        if !detail.info.isWorkingCopy {
            Button(moveToWorkingCopyActionLabel(for: contextPaths)) {
                actions?.moveToWorkingCopy(rev: detail.info.changeId, paths: contextPaths)
            }
        }
        if detail.info.parents.count > 1 {
            Menu(restoreActionLabel(for: contextPaths)) {
                ForEach(Array(detail.info.parents.enumerated()), id: \.offset) { idx, parentId in
                    Button("Parent \(idx + 1): \(String(parentId.prefix(8)))") {
                        actions?.restoreFiles(rev: parentId, paths: contextPaths)
                    }
                }
            }
        } else {
            Button(restoreActionLabel(for: contextPaths)) {
                actions?.restoreFiles(rev: detail.info.changeId, paths: contextPaths)
            }
        }
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

    func toggleReview(_ path: String) {
        reviewStore.toggleReviewed(changeId: detail.info.changeId, path: path)
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

    private func contextSelectionPaths(for clickedPath: String) -> [String] {
        let activeSelection: Set<String> =
            if selectedPaths.contains(clickedPath), selectedPaths.count > 1 {
                selectedPaths
            } else {
                [clickedPath]
            }
        return visibleSelectablePaths.filter(activeSelection.contains)
    }

    private func selectionTitle(
        count: Int,
        singular: String,
        plural: String? = nil
    ) -> String {
        if count == 1 {
            return singular
        }
        return "\(singular.split(separator: " ").first ?? "") \(count) \(plural ?? "Files")"
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

    private func setReviewState(for paths: [String], reviewed: Bool) {
        for path in paths {
            if reviewed {
                reviewStore.markReviewed(changeId: detail.info.changeId, path: path)
            } else {
                reviewStore.markUnreviewed(changeId: detail.info.changeId, path: path)
            }
        }
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

    func showInFinder(_ path: String) {
        NSWorkspace.shared.activateFileViewerSelecting([URL(fileURLWithPath: repoPath).appendingPathComponent(path)])
    }

    func loadAnnotate(rev: String, path: String) {
        guard let repo else { return }
        annotatePath = path
        annotateLines = nil
        Task.detached {
            let lines = try? repo.annotateFile(rev: rev, path: path)
            await MainActor.run {
                annotateLines = lines ?? []
            }
        }
    }

    func loadFileHistory(path: String) {
        guard let repo else { return }
        fileHistoryPath = path
        fileHistory = nil
        Task.detached {
            let history = try? repo.fileHistory(path: path)
            await MainActor.run {
                fileHistory = history ?? []
            }
        }
    }

    func loadConflictedPaths() {
        guard let repo, detail.info.hasConflict else {
            conflictedPaths = []
            return
        }
        let rev = detail.info.changeId
        Task.detached {
            let paths = (try? repo.resolveList(rev: rev)) ?? []
            let pathSet = Set(paths)
            await MainActor.run {
                conflictedPaths = pathSet
            }
        }
    }
}
