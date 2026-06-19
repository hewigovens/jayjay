import JayJayCore
import SwiftUI

extension ChangeDetailView {
    var headerSection: some View {
        VStack(alignment: .leading, spacing: 6) {
            CopyableRow(
                "Change",
                value: detail.info.changeId,
                emphasizedPrefix: Int(detail.info.changeIdShortLen)
            )
            CopyableRow(
                "Commit",
                value: String(detail.info.commitId.prefix(12)),
                copyValue: detail.info.commitId,
                emphasizedPrefix: Int(detail.info.commitIdShortLen)
            )
            HStack(spacing: 6) {
                Text("Author").jayjayFont(11).foregroundStyle(.secondary).frame(width: 70, alignment: .trailing)
                CommitAvatar(email: detail.info.author.email, size: 18)
                Text("\(detail.info.author.name) <\(detail.info.author.email)>")
                    .jayjayFont(11, design: .monospaced)
                    .textSelection(.enabled)
            }
            LabeledRow("Date", value: formatTimestamp(detail.info.author.timestampMillis))
            if !detail.info.parents.isEmpty {
                LabeledRow("Parents", value: detail.info.parents.map { String($0.prefix(12)) }.joined(separator: ", "))
            }
            if !detail.info.bookmarks.isEmpty {
                HStack(spacing: 4) {
                    Text("Bookmarks").jayjayFont(11).foregroundStyle(.secondary).frame(width: 70, alignment: .trailing)
                    ForEach(detail.info.bookmarks, id: \.self) { name in
                        HStack(spacing: 4) {
                            Text(name).jayjayFont(11, design: .monospaced)
                                .padding(.horizontal, 6).padding(.vertical, 2)
                                .background(.tint.opacity(0.15), in: .capsule)
                            CopyIconButton(value: name, help: "Copy bookmark name")
                        }
                    }
                }
            }
            if let stats = diffStats, stats.insertions > 0 || stats.deletions > 0 {
                HStack(spacing: 4) {
                    Text("Changes").jayjayFont(11).foregroundStyle(.secondary).frame(width: 70, alignment: .trailing)
                    if stats.insertions > 0 {
                        Text("+\(stats.insertions)")
                            .jayjayFont(11, weight: .semibold, design: .monospaced)
                            .foregroundStyle(.green)
                    }
                    if stats.deletions > 0 {
                        Text("-\(stats.deletions)")
                            .jayjayFont(11, weight: .semibold, design: .monospaced)
                            .foregroundStyle(.red)
                    }
                }
                .accessibilityElement(children: .ignore)
                .accessibilityIdentifier(AID.Detail.diffStats(insertions: stats.insertions, deletions: stats.deletions))
            }
        }
    }

    var compareBanner: some View {
        HStack(spacing: 8) {
            Button {
                onReverseCompare?()
            } label: {
                Image(systemName: "arrow.left.arrow.right")
                    .foregroundStyle(.orange)
            }
            .buttonStyle(.plain)
            .help("Reverse compare direction")
            Text(compareDisplay?.title ?? "Comparing")
                .jayjayFont(12, weight: .medium)
            compareLabel(compareDisplay?.from ?? String(compareFromId?.prefix(8) ?? ""))
            Image(systemName: "arrow.right")
                .jayjayFont(10)
                .foregroundStyle(.secondary)
            compareLabel(compareDisplay?.to ?? String(detailRevision.prefix(8)))
            Spacer()
            Text("\(detail.diff.count) files changed")
                .jayjayFont(11)
                .foregroundStyle(.secondary)
            Button {
                onClearCompare?()
            } label: {
                Image(systemName: "xmark.circle.fill")
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)
            .help("Exit compare mode")
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .background(.orange.opacity(0.08))
        .accessibilityIdentifier(AID.Compare.banner)
    }

    private func compareLabel(_ text: String) -> some View {
        Text(text)
            .jayjayFont(12, weight: .semibold, design: .monospaced)
            .lineLimit(1)
            .truncationMode(.middle)
    }

    func formatTimestamp(_ millis: Int64) -> String {
        Date(timeIntervalSince1970: Double(millis) / 1000.0).formatted(.dateTime.year().month().day().hour().minute())
    }

    func resetState(preservingFileContext: Bool = false) {
        let previousSelectedPath = selectedPath
        let previousSelectedPaths = selectedPaths
        let previousAnchorPath = fileSelectionAnchorPath
        let previousShowFileFilter = showFileFilter
        let previousFileFilter = fileFilter

        descriptionText = detail.info.description
        editingDescription = false
        trackedGitLfsPaths = []
        restoreFileSelection(
            preserving: preservingFileContext,
            previousSelectedPath: previousSelectedPath,
            previousSelectedPaths: previousSelectedPaths,
            previousAnchorPath: previousAnchorPath
        )
        if preservingFileContext {
            showFileFilter = previousShowFileFilter
            fileFilter = previousFileFilter
        } else {
            showFileFilter = false
            fileFilter = ""
        }
        paneMode = .files
        loadConflictedPaths()
        loadTrackedGitLfsPaths()
        loadDiffStats()
        refreshReviewedPaths()
        // No clear(): content-addressed by commit id, so prior changes stay warm and never go stale.
        diffStore.preload(
            hunks: detail.diff,
            rev: detailRevision,
            commitId: detail.info.commitId,
            repo: repo,
            compareFromRev: compareFromId,
            ignoreWhitespace: appSettings.ignoreWhitespace
        )
    }

    func refreshReviewedPaths() {
        reviewedPaths = reviewStore.reviewedPaths(
            changeId: detailRevision,
            files: visibleDiff.map { (path: $0.path, identity: $0.reviewIdentity) }
        )
    }

    private func restoreFileSelection(
        preserving: Bool,
        previousSelectedPath: String?,
        previousSelectedPaths: Set<String>,
        previousAnchorPath: String?
    ) {
        let availablePaths = Set(detail.diff.map(\.path))
        let fallbackPath = detail.diff.first?.path

        if preserving,
           let previousSelectedPath,
           availablePaths.contains(previousSelectedPath)
        {
            selectedPath = previousSelectedPath

            let preservedPaths = previousSelectedPaths.intersection(availablePaths)
            selectedPaths = preservedPaths.isEmpty ? [previousSelectedPath] : preservedPaths

            if let previousAnchorPath, availablePaths.contains(previousAnchorPath) {
                fileSelectionAnchorPath = previousAnchorPath
            } else {
                fileSelectionAnchorPath = previousSelectedPath
            }
            return
        }

        selectedPath = fallbackPath
        selectedPaths = fallbackPath.map { [$0] } ?? []
        fileSelectionAnchorPath = fallbackPath
    }

    func loadDiffStats() {
        let rev = detailRevision
        // Key on commitId, not the (stable) changeId, so amends to a mutable change reload.
        let commitId = detail.info.commitId
        guard diffStatsCommitId != commitId else { return }
        diffStatsCommitId = commitId
        diffStats = nil
        guard let repo else { return }
        Task.detached {
            let stats = try? repo.diffStats(rev: rev)
            await MainActor.run {
                guard diffStatsCommitId == commitId else { return }
                diffStats = stats
            }
        }
    }

    func loadTrackedGitLfsPaths() {
        guard let repo, detail.info.isWorkingCopy else {
            trackedGitLfsPaths = []
            return
        }
        let paths = detail.diff.map(\.path)
        Task.detached {
            let paths = (try? repo.gitLfsPaths(paths: paths)) ?? []
            await MainActor.run {
                trackedGitLfsPaths = Set(paths)
                if appSettings.hideGitLfsDiffs,
                   let selectedPath,
                   trackedGitLfsPaths.contains(selectedPath)
                {
                    let nextVisible = detail.diff.first { !trackedGitLfsPaths.contains($0.path) }
                    self.selectedPath = nextVisible?.path
                    self.selectedPaths = nextVisible.map { [$0.path] } ?? []
                    self.fileSelectionAnchorPath = nextVisible?.path
                }
            }
        }
    }

    func conflictBar(path: String) -> some View {
        HStack(spacing: 10) {
            Image(systemName: "exclamationmark.triangle.fill")
                .foregroundStyle(.red)
            Text("Conflict")
                .jayjayFont(12, weight: .semibold)
            Spacer()
            Button("Use Ours") {
                actions?.resolveUseOurs(rev: detailRevision, path: path)
            }
            .buttonStyle(.bordered)
            .accessibilityIdentifier(AID.Conflict.useOurs(path))
            Button("Use Theirs") {
                actions?.resolveUseTheirs(rev: detailRevision, path: path)
            }
            .buttonStyle(.bordered)
            .accessibilityIdentifier(AID.Conflict.useTheirs(path))
            if let tool = appSettings.externalEditor.jjMergeTool {
                Button("Resolve in \(appSettings.externalEditor.title)") {
                    actions?.resolveInEditor(rev: detailRevision, path: path, tool: tool)
                }
                .buttonStyle(.borderedProminent)
            } else {
                Button("Open in \(appSettings.externalEditor.title)") {
                    appSettings.openInEditor(filePath: path, repoPath: repoPath)
                }
                .buttonStyle(.borderedProminent)
            }
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .background(.red.opacity(0.08))
    }
}
