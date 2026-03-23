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
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 6)

            HStack(spacing: 4) {
                Image(systemName: "magnifyingglass").foregroundStyle(.tertiary).jayjayFont(10)
                TextField("Filter files", text: $fileFilter)
                    .textFieldStyle(.plain).jayjayFont(11)
            }
            .padding(.horizontal, 10)
            .padding(.bottom, 6)

            Divider()

            if appSettings.treeFileList {
                treeFileList
            } else {
                flatFileList
            }
        }
        .focused($fileColumnFocused)
        .onKeyPress(.space) {
            guard detail.info.isWorkingCopy, let path = selectedPath else { return .ignored }
            toggleReview(path)
            if reviewedPaths.contains(path),
               let next = filteredDiff.first(where: { !reviewedPaths.contains($0.path) })
            {
                selectedPath = next.path
            }
            return .handled
        }
        .onKeyPress(.upArrow) {
            guard let cur = selectedPath, let i = filteredDiff.firstIndex(where: { $0.path == cur }),
                  i > 0 else { return .ignored }
            selectedPath = filteredDiff[i - 1].path
            return .handled
        }
        .onKeyPress(.downArrow) {
            guard let cur = selectedPath, let i = filteredDiff.firstIndex(where: { $0.path == cur }),
                  i < filteredDiff.count - 1 else { return .ignored }
            selectedPath = filteredDiff[i + 1].path
            return .handled
        }
    }

    private var flatFileList: some View {
        List(filteredDiff, id: \.path, selection: $selectedPath) { hunk in
            fileRowView(hunk: hunk)
        }
        .listStyle(.plain)
    }

    private var treeFileList: some View {
        let treeEntries = buildFileTree(paths: detail.diff.map(\.path))
        return List(selection: $selectedPath) {
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
    }

    func fileRowView(hunk: DiffHunk) -> some View {
        FileRow(
            hunk: hunk,
            isSelected: selectedHunk?.path == hunk.path,
            showReview: detail.info.isWorkingCopy,
            isReviewed: reviewedPaths.contains(hunk.path),
            onToggleReview: { toggleReview(hunk.path) }
        )
        .contentShape(Rectangle())
        .onTapGesture {
            selectedPath = hunk.path
            fileColumnFocused = true
        }
        .contextMenu {
            Button("Open in \(appSettings.externalEditor.title)") {
                appSettings.openInEditor(filePath: hunk.path, repoPath: repoPath)
            }
            Button("Show in Finder") { showInFinder(hunk.path) }
            Button("Copy Path") {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(hunk.path, forType: .string)
            }
            Divider()
            Button("Annotate (Blame)") { loadAnnotate(rev: detail.info.changeId, path: hunk.path) }
            Button("File History") { loadFileHistory(path: hunk.path) }
            Divider()
            if detail.info.isWorkingCopy {
                Button(reviewedPaths.contains(hunk.path) ? "Mark as Unreviewed" : "Mark as Reviewed") {
                    toggleReview(hunk.path)
                }
                Divider()
            }
            Button("Split to New Change") {
                splitPaths = [hunk.path]
                showSplitSheet = true
            }
            if !detail.info.isWorkingCopy {
                Button("Move to Working Copy") {
                    actions?.moveToWorkingCopy(rev: detail.info.changeId, paths: [hunk.path])
                }
            }
            Button("Restore to Parent") { actions?.restoreFiles(rev: detail.info.changeId, paths: [hunk.path]) }
            if detail.info.isWorkingCopy {
                Button("Delete from Disk", role: .destructive) { actions?.deleteFiles(paths: [hunk.path]) }
            }
            Divider()
            Button("Ignore & Untrack") { actions?.ignoreAndUntrack(paths: [hunk.path]) }
        }
    }

    func toggleReview(_ path: String) {
        reviewStore.toggleReviewed(changeId: detail.info.changeId, path: path)
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
}
