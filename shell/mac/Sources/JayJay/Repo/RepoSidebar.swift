import JayJayCore
import SwiftUI

extension RepoContentView {
    var sidebar: some View {
        VStack(spacing: 0) {
            if showRevsetFilter {
                VStack(spacing: 6) {
                    HStack(spacing: 6) {
                        TextField("Revset expression", text: $revsetDraft)
                            .textFieldStyle(.roundedBorder).jayjayFont(12, design: .monospaced)
                            .onSubmit { applyRevset() }
                        Button { applyRevset() } label: {
                            Image(systemName: "arrow.right.circle.fill").foregroundStyle(.secondary)
                        }
                        .buttonStyle(.plain).disabled(revsetDraft == viewModel.revset)
                        Button {
                            revsetDraft = ""
                            applyRevset()
                        } label: {
                            Image(systemName: "xmark.circle.fill").foregroundStyle(.tertiary)
                        }
                        .buttonStyle(.plain)
                        .help("Reset to default")
                    }
                    ScrollView(.horizontal, showsIndicators: false) {
                        HStack(spacing: 6) {
                            revsetChip("All", revset: "all()")
                            revsetChip("Mine", revset: "mine()")
                            revsetChip("Bookmarks", revset: "bookmarks()")
                            revsetChip("Trunk", revset: "trunk()")
                            revsetChip("Conflicts", revset: "conflict()")
                            revsetChip("Heads", revset: "heads(all())")
                        }
                    }
                }
                .padding(.horizontal, 12).padding(.vertical, 8)
                Divider()
            }
            if let name = viewModel.pendingPushBookmark {
                pushFollowUpBanner(name)
                Divider()
            }
            DAGView(
                entries: viewModel.graphEntries,
                selectedId: viewModel.selectedChangeId,
                compareFromId: viewModel.compareFromId,
                actions: viewModel,
                onRequestRebase: { handleDAGRebase($0) },
                activePane: $activePane,
                revealRequest: dagRevealRequest,
                prHostName: viewModel.prHostName,
                onMoveBookmarkForward: { viewModel.moveBookmarkForward(name: $0) },
                onMoveBookmarkToRev: { viewModel.moveBookmark(name: $0, toRev: $1) },
                onMoveWorkingCopyToRev: { viewModel.edit(rev: $0) },
                onPushBookmark: { viewModel.gitPush(bookmark: $0) },
                onOpenPRForBookmark: { viewModel.openPR(bookmark: $0) },
                onDeleteBookmark: { viewModel.deleteBookmark(name: $0) },
                onAbandon: { requestAbandon($0) },
                onCreateBookmark: { rev in presentBookmarkCreate(rev: rev) },
                onCreateStackedPRs: { rev in presentStackedPr(rev: rev) },
                onLoadMore: viewModel.canLoadMore ? { viewModel.loadMore() } : nil
            )
            if shouldShowCommitBox {
                Divider()
                CommitBox(
                    description: viewModel.workingCopyDescription,
                    summary: $viewModel.commitSummaryDraft,
                    details: $viewModel.commitDescriptionDraft,
                    onSaveDescription: { viewModel.describeWorkingCopy(message: $0) },
                    onCommit: {
                        await viewModel.commit(message: $0, manageSubmodules: settings.enableGitSubmoduleSupport)
                    },
                    onGenerateMessage: { await viewModel.generateCommitMessage() },
                    aiProvider: viewModel.aiProvider
                )
            }
        }
    }

    func pushFollowUpBanner(_ name: String) -> some View {
        HStack(spacing: 6) {
            Image(systemName: "bookmark.fill").foregroundStyle(.green).jayjayFont(11)
            Text("Moved").jayjayFont(11).foregroundStyle(.secondary)
            Text(name).jayjayFont(11, weight: .medium, design: .monospaced).lineLimit(1)
            Spacer()
            Button("Push") { viewModel.confirmPendingPush() }
                .controlSize(.small)
            Button {
                viewModel.dismissPendingPush()
            } label: {
                Image(systemName: "xmark").jayjayFont(10)
            }
            .buttonStyle(.plain).foregroundStyle(.secondary)
            .help("Dismiss")
        }
        .padding(.horizontal, 12).padding(.vertical, 6)
        .glassEffect(in: RoundedRectangle(cornerRadius: 8))
    }

    func revsetChip(_ label: String, revset: String) -> some View {
        Button {
            revsetDraft = revset
            applyRevset()
        } label: {
            Text(label)
                .jayjayFont(11, weight: .medium)
                .padding(.horizontal, 10)
                .padding(.vertical, 4)
                .background(
                    viewModel.revset == revset
                        ? AnyShapeStyle(Color.accentColor.opacity(0.2))
                        : AnyShapeStyle(Color.primary.opacity(0.06)),
                    in: Capsule()
                )
        }
        .buttonStyle(.plain)
    }

    func applyRevset() {
        let t = revsetDraft.trimmingCharacters(in: .whitespacesAndNewlines)
        if t.isEmpty {
            let defaultRevset = RepoViewModel.buildDefaultRevset()
            revsetDraft = defaultRevset
            viewModel.applyRevset(defaultRevset)
        } else {
            revsetDraft = t
            viewModel.applyRevset(t)
        }
    }

    var statusBar: some View {
        StatusBarView(
            leadingItems: statusBarLeadingItems,
            trailingItems: statusBarTrailingItems
        )
    }

    private var statusBarLeadingItems: [StatusBarItem] {
        var items: [StatusBarItem] = []
        if viewModel.workspaces.count > 1,
           let current = viewModel.workspaces.first(where: \.isCurrent)
        {
            items.append(.picker(
                id: "workspace",
                icon: "square.on.square",
                label: current.name,
                options: workspacePickerOptions
            ))
        }
        items.append(.text(id: "path", icon: "folder", text: viewModel.repoPath))
        if let bookmark = activeBookmarkSyncItem {
            items.append(bookmark)
        }
        if let pr = viewModel.prInfo, let url = URL(string: pr.url) {
            items.append(.link(
                id: "pr",
                icon: pr.checksIcon,
                text: "#\(pr.number) \(pr.stateLabel)",
                url: url,
                tooltip: pr.title
            ))
        }
        return items
    }

    private var shouldShowCommitBox: Bool {
        viewModel.selectedChange?.info.isWorkingCopy == true
    }

    private var statusBarTrailingItems: [StatusBarItem] {
        var items: [StatusBarItem] = []
        if let wc = workingCopyStatItem { items.append(wc) }
        if let divergent = divergentItem { items.append(divergent) }
        let conflictedCount = viewModel.changes.filter(\.hasConflict).count
        if conflictedCount > 0 {
            items.append(.action(
                id: "conflicts",
                icon: "exclamationmark.triangle.fill",
                text: "\(conflictedCount) conflicted"
            ) {
                revsetDraft = "conflict()"
                applyRevset()
            })
        }
        if let lastOp = lastOpItem { items.append(lastOp) }
        items.append(.text(
            id: "changes",
            icon: "point.3.connected.trianglepath.dotted",
            text: "\(viewModel.changes.count) changes",
            tooltip: "Changes in view"
        ))
        return items
    }

    /// Working-copy edit summary (files + line counts), hidden when clean.
    private var workingCopyStatItem: StatusBarItem? {
        guard let stats = viewModel.workingCopyStats, stats.filesChanged > 0 else { return nil }
        var text = "\(stats.filesChanged) file\(stats.filesChanged == 1 ? "" : "s")"
        if stats.insertions > 0 || stats.deletions > 0 {
            text += " +\(stats.insertions) −\(stats.deletions)"
        }
        return .text(id: "wc-stat", icon: "pencil", text: text, tooltip: "Working-copy changes")
    }

    /// Count of divergent change-ids in view; clicking filters to `divergent()`.
    private var divergentItem: StatusBarItem? {
        let ids = Set(viewModel.changes.filter(\.isDivergent).map(\.changeId.id))
        guard !ids.isEmpty else { return nil }
        return .action(id: "divergent", icon: "arrow.triangle.branch", text: "\(ids.count) divergent") {
            revsetDraft = "divergent()"
            applyRevset()
        }
    }

    /// The repo's current operation; clicking opens the Operation Log.
    private var lastOpItem: StatusBarItem? {
        let desc = viewModel.currentOperationDescription
        guard !desc.isEmpty else { return nil }
        let short = desc.count > 40 ? String(desc.prefix(39)) + "…" : desc
        return .action(id: "last-op", icon: OperationIcon.symbol(for: desc), text: short) {
            showUndo()
        }
    }

    /// Sync state of the nearest tracked bookmark at or below `@`, VSCode-style: `name ✓` synced, `↑N` ahead, `↓N` behind, `↑N↓M` diverged.
    private var activeBookmarkSyncItem: StatusBarItem? {
        let changes = viewModel.changes
        guard let wcIndex = changes.firstIndex(where: { $0.isWorkingCopy }) else { return nil }
        for change in changes[wcIndex...] {
            for name in change.bookmarks {
                guard let bookmark = viewModel.bookmarks.first(where: { $0.name == name }),
                      let target = primaryRemoteTarget(bookmark)
                else { continue }
                let badge = switch target.status {
                    case .synced: "✓"
                    case .ahead: "↑\(target.ahead)"
                    case .behind: "↓\(target.behind)"
                    case .diverged: "↑\(target.ahead)↓\(target.behind)"
                    @unknown default: ""
                }
                return .text(
                    id: "bookmark-sync",
                    icon: "bookmark",
                    text: badge.isEmpty ? name : "\(name) \(badge)",
                    tooltip: "\(name) vs \(target.remote)"
                )
            }
        }
        return nil
    }

    private func primaryRemoteTarget(_ bookmark: BookmarkInfo) -> RemoteBookmarkTarget? {
        bookmark.remoteTargets.first(where: { $0.remote == "origin" }) ?? bookmark.remoteTargets.first
    }

    private var workspacePickerOptions: [StatusBarPickerOption] {
        viewModel.workspaces.map { ws in
            if ws.isCurrent {
                return StatusBarPickerOption(id: ws.name, label: ws.name, icon: "checkmark", disabled: true)
            }
            var children: [StatusBarPickerOption] = [
                StatusBarPickerOption(id: "\(ws.name)-open", label: "Open") {
                    windowManager.openRepo(ws.path)
                }
            ]
            if ws.name != "default" {
                children.append(StatusBarPickerOption(id: "\(ws.name)-forget", label: "Forget") {
                    viewModel.workspaceForget(name: ws.name)
                    settings.removeRecentRepo(ws.path)
                })
                children.append(StatusBarPickerOption(
                    id: "\(ws.name)-delete", label: "Forget & Delete from Disk",
                    destructive: true
                ) {
                    viewModel.workspaceForget(name: ws.name)
                    settings.removeRecentRepo(ws.path)
                    try? FileManager.default.removeItem(atPath: ws.path)
                })
            }
            return StatusBarPickerOption(id: ws.name, label: ws.name, children: children)
        }
    }
}

extension PrInfo {
    var stateLabel: String {
        switch state {
            case .open: "open"
            case .closed: "closed"
            case .merged: "merged"
        }
    }

    var checksIcon: String {
        switch checks {
            case .passing: "checkmark.circle.fill"
            case .failing: "xmark.circle.fill"
            case .pending: "clock.circle"
            case .none: "circle.dashed"
        }
    }
}
