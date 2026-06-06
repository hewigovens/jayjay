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
                onPushBookmark: { viewModel.gitPush(bookmark: $0) },
                onOpenPRForBookmark: { viewModel.openPR(bookmark: $0) },
                onAbandon: { requestAbandon($0) },
                onCreateBookmark: { rev in presentBookmarkCreate(rev: rev) },
                onLoadMore: viewModel.canLoadMore ? { viewModel.loadMore() } : nil
            )
            if shouldShowCommitBox {
                Divider()
                CommitBox(
                    description: viewModel.workingCopyDescription,
                    draft: $viewModel.commitDraft,
                    onCommit: {
                        await viewModel.commit(message: $0, manageSubmodules: settings.enableGitSubmoduleSupport)
                    },
                    onGenerateMessage: { await viewModel.generateCommitMessage() },
                    aiProvider: viewModel.aiProvider
                )
            }
        }
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
        items.append(.text(id: "path", text: viewModel.repoPath))
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
        items.append(.text(id: "changes", text: "\(viewModel.changes.count) changes"))
        return items
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
