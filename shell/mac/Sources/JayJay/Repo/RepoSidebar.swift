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
                        Button("Save") {
                            revsetSaveName = suggestedRevsetName
                            modal = .saveRevset
                        }
                        .controlSize(.small)
                        .disabled(revsetDraft.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
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
                            ForEach(SavedRevset.builtIns, id: \.id) { revset in
                                revsetChip(revset)
                            }
                            ForEach(settings.savedRevsets, id: \.id) { revset in
                                revsetChip(revset, saved: true)
                            }
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

    var revsetSaveSheet: some View {
        SheetContainer(
            title: "Save Revset",
            cancelLabel: "Cancel",
            confirmLabel: "Save",
            confirmDisabled: revsetSaveName.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
            onCancel: { modal = nil },
            onConfirm: { saveCurrentRevset() },
            content: {
                TextField("Name", text: $revsetSaveName)
                    .textFieldStyle(.roundedBorder)
                    .frame(width: 220)
                    .onSubmit { saveCurrentRevset() }
                Text(revsetDraft.trimmingCharacters(in: .whitespacesAndNewlines))
                    .jayjayFont(11, design: .monospaced)
                    .foregroundStyle(.secondary)
                    .lineLimit(3)
                    .frame(width: 220, alignment: .leading)
            }
        )
    }

    private var suggestedRevsetName: String {
        let expression = revsetDraft.trimmingCharacters(in: .whitespacesAndNewlines)
        if let existing = SavedRevset.builtIns.first(where: { $0.expression == expression }) {
            return existing.name
        }
        if let existing = settings.savedRevsets.first(where: { $0.expression == expression }) {
            return existing.name
        }
        return "Custom Revset"
    }

    func revsetChip(_ item: SavedRevset, saved: Bool = false) -> some View {
        Button {
            revsetDraft = item.expression
            applyRevset()
        } label: {
            Text(item.name)
                .jayjayFont(11, weight: .medium)
                .padding(.horizontal, 10)
                .padding(.vertical, 4)
                .background(
                    viewModel.revset == item.expression
                        ? AnyShapeStyle(Color.accentColor.opacity(0.2))
                        : AnyShapeStyle(Color.primary.opacity(0.06)),
                    in: Capsule()
                )
        }
        .buttonStyle(.plain)
        .help(item.expression)
        .contextMenu {
            if saved {
                Button(role: .destructive) {
                    settings.removeSavedRevset(id: item.id)
                } label: {
                    Label("Delete Saved Revset", systemImage: "trash")
                }
            } else {
                Text(item.expression)
            }
        }
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

    private func saveCurrentRevset() {
        guard !revsetSaveName.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
              !revsetDraft.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
        else { return }
        settings.saveRevset(name: revsetSaveName, expression: revsetDraft)
        modal = nil
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
                revsetDraft = "conflicts()"
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
