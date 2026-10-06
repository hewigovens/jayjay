import JayJayCore
import SwiftUI

extension RepoContentView {
    @ViewBuilder
    func modalView(for modal: RepoModalState) -> some View {
        switch modal {
            case let .editDescription(rev, description):
                DescriptionEditorSheet(
                    revision: rev,
                    description: description,
                    onCancel: { self.modal = nil },
                    onSave: { message in
                        viewModel.describe(rev: rev, message: message)
                        self.modal = nil
                    }
                )
            case let .createBookmark(rev):
                refCreateSheet(title: "Create Bookmark", placeholder: "Bookmark name", rev: rev) { name in
                    viewModel.createBookmark(name: name, rev: rev)
                }
            case let .createTag(rev):
                refCreateSheet(
                    title: "Create Tag",
                    placeholder: "Tag name",
                    note: "A tagged change becomes immutable.",
                    rev: rev
                ) { name in
                    viewModel.createTag(name: name, rev: rev)
                }
            case let .stackedPr(rev):
                StackedPrPanel(viewModel: viewModel, tipRev: rev, onDismiss: { self.modal = nil })
            case let .confirmChange(confirmation):
                changeConfirmationSheet(confirmation)
            case .submoduleAttention:
                submoduleAttentionSheet
            case .undoLog:
                UndoView(
                    entries: viewModel.opLogEntries,
                    onRestore: { opId in viewModel.opRestore(opId: opId) },
                    onDismiss: { self.modal = nil }
                )
            case .bookmarkManager:
                bookmarkManagerSheet
            case .workspaceCreate, .pullRequestImport, .confirmWorkspaceDelete, .confirmTagDeleteOnRemote:
                secondarySheet(for: modal)
            case .ratingPrompt:
                RatingPromptView(
                    onDismiss: { self.modal = nil },
                    onDontShowAgain: {
                        settings.ratingPromptDismissed = true
                        self.modal = nil
                    }
                )
        }
    }

    private var bookmarkManagerSheet: some View {
        BookmarkManagerView(
            bookmarks: viewModel.bookmarks,
            actions: viewModel,
            repo: viewModel.repo,
            prHostName: viewModel.prHostName,
            onFilter: { target in
                self.modal = nil
                viewModel.filterByBookmark(target)
            },
            onDiffBookmark: { request in
                self.modal = nil
                viewModel.diffBookmark(request)
            },
            onDismiss: { self.modal = nil }
        )
    }

    @ViewBuilder
    /// Split from the main switch to keep it under swiftlint's complexity limit.
    private func secondarySheet(for modal: RepoModalState) -> some View {
        switch modal {
            case .workspaceCreate:
                workspaceCreateSheet
            case .pullRequestImport:
                PullRequestImportSheet(
                    viewModel: viewModel,
                    onDismiss: { self.modal = nil },
                    onOpen: { dest in windowManager.openRepo(dest) }
                )
            case let .confirmWorkspaceDelete(workspace):
                workspaceDeleteSheet(workspace: workspace)
            case let .confirmTagDeleteOnRemote(name):
                tagDeleteOnRemoteSheet(name: name)
            case .editDescription, .createBookmark, .createTag, .stackedPr, .confirmChange, .submoduleAttention, .undoLog, .bookmarkManager, .ratingPrompt:
                EmptyView()
        }
    }

    @ViewBuilder
    private func changeConfirmationSheet(_ confirmation: RepoChangeConfirmation) -> some View {
        switch confirmation {
            case let .abandon(rev):
                abandonSheet(rev: rev)
            case let .abandonSelection(revisions):
                abandonSelectionSheet(revisions: revisions)
            case let .squashSelection(revisions):
                squashSelectionSheet(revisions: revisions)
            case let .rebase(request):
                rebaseConfirmationSheet(request: request)
        }
    }

    private func refCreateSheet(
        title: String,
        placeholder: String,
        note: String? = nil,
        rev: String,
        onCreate: @escaping (String) -> Void
    ) -> some View {
        let submit = { submitRefCreate(onCreate) }
        return SheetContainer(
            title: title,
            subtitle: "On change: \(String(rev.prefix(12)))",
            cancelLabel: "Cancel",
            confirmLabel: "Create",
            confirmDisabled: refCreateName.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
            onCancel: { modal = nil },
            onConfirm: submit,
            content: {
                TextField(placeholder, text: $refCreateName)
                    .textFieldStyle(.roundedBorder)
                    .jayjayFont(13, design: .monospaced)
                    .onSubmit(submit)
                if let note {
                    Text(note)
                        .jayjayFont(11)
                        .foregroundStyle(.secondary)
                }
            }
        )
    }

    private func tagDeleteOnRemoteSheet(name: String) -> some View {
        let remotes = viewModel.tags.first { $0.name == name }?.trackedRemotes.joined(separator: ", ") ?? "the remote"
        return DestructiveConfirmSheet(
            title: "Delete Tag \(name) on \(remotes)?",
            message: "This removes the tag locally and pushes the deletion to \(remotes). A GitHub release on this tag turns into a draft and its downloads stop working until the tag exists again.",
            confirmLabel: "Delete on Remote",
            width: 400,
            onCancel: { modal = nil },
            onConfirm: {
                modal = nil
                viewModel.deleteTagAndPush(name: name)
            }
        )
    }

    private func abandonSheet(rev: String) -> some View {
        DestructiveConfirmSheet(
            title: "Abandon Change?",
            message: "This will remove the change and reparent its children.\nYou can undo this with jj op restore.",
            confirmLabel: "Abandon",
            dontAskAgain: Binding(
                get: { settings.skipAbandonConfirmation },
                set: { settings.skipAbandonConfirmation = $0 }
            ),
            onCancel: { modal = nil },
            onConfirm: {
                viewModel.abandon(rev: rev)
                modal = nil
            }
        )
    }

    private func abandonSelectionSheet(revisions: [String]) -> some View {
        DestructiveConfirmSheet(
            title: "Abandon \(revisions.count) Changes?",
            message: "This will remove the selected changes and reparent their descendants.\nYou can undo this with jj op restore.",
            confirmLabel: "Abandon \(revisions.count)",
            onCancel: { modal = nil },
            onConfirm: {
                viewModel.abandon(revs: revisions)
                modal = nil
            }
        )
    }

    private func squashSelectionSheet(revisions: [String]) -> some View {
        let destination = revisions.last ?? ""
        return SheetContainer(
            title: "Squash \(revisions.count) Changes?",
            subtitle: "Into \(String(destination.prefix(12)))",
            cancelLabel: "Cancel",
            confirmLabel: "Squash",
            onCancel: { modal = nil },
            onConfirm: {
                viewModel.squash(revs: revisions)
                modal = nil
            },
            content: {
                Text(
                    "This combines the selected linear range into its oldest change and abandons the other selected changes. You can undo it with jj op restore."
                )
                .jayjayFont(12)
                .foregroundStyle(.secondary)
                .fixedSize(horizontal: false, vertical: true)
            }
        )
        .frame(width: 380)
    }

    private func rebaseConfirmationSheet(request: DAGRebaseRequest) -> some View {
        SheetContainer(
            title: request.selectionCommitIds.isEmpty ? "Rebase Change?" : "Rebase \(request.selectionCommitIds.count) Changes?",
            subtitle: "\(String(request.sourceCommitId.prefix(12))) -> \(String(request.destCommitId.prefix(12)))",
            cancelLabel: "Cancel",
            confirmLabel: "Rebase",
            onCancel: { modal = nil },
            onConfirm: {
                modal = nil
                runDAGRebase(request)
            },
            content: {
                VStack(alignment: .leading, spacing: 12) {
                    rebaseSummaryRow(
                        title: "Change",
                        value: request.sourceLabel,
                        detail: request.sourceChangeId
                    )
                    Label("Will become a child of", systemImage: "arrow.down")
                        .jayjayFont(11)
                        .foregroundStyle(.secondary)
                    rebaseSummaryRow(
                        title: "New parent",
                        value: request.destLabel,
                        detail: request.destChangeId
                    )
                    Toggle(isOn: Binding(
                        get: { settings.confirmDragRebase },
                        set: { settings.confirmDragRebase = $0 }
                    )) {
                        Text("Confirm before drag-to-rebase")
                            .jayjayFont(12)
                    }
                    Text("Any conflicts will appear inline after the rebase.")
                        .jayjayFont(11)
                        .foregroundStyle(.secondary)
                }
            }
        )
        .frame(width: 360)
    }

    private var workspaceCreateSheet: some View {
        SheetContainer(
            title: "New Workspace",
            subtitle: "Creates a new working copy in a sibling directory",
            cancelLabel: "Cancel",
            cancelDisabled: workspaceCreating,
            confirmLabel: workspaceCreating ? "Creating…" : "Create",
            confirmDisabled: workspaceCreating
                || workspaceName.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
            onCancel: {
                modal = nil
                workspaceName = ""
                workspaceNameError = nil
            },
            onConfirm: {
                let name = workspaceName.trimmingCharacters(in: .whitespacesAndNewlines)
                guard !name.isEmpty else { return }
                guard isValidWorkspaceName(name: name) else {
                    workspaceNameError = "Invalid workspace name: \(name)"
                    return
                }
                workspaceNameError = nil
                workspaceCreating = true
                let parent = URL(fileURLWithPath: viewModel.repoPath).deletingLastPathComponent()
                let dest = parent.appendingPathComponent(name).path
                let windowManager = windowManager
                viewModel.workspaceAdd(dest: dest, name: name, onSuccess: {
                    workspaceCreating = false
                    if case .workspaceCreate = modal {
                        modal = nil
                        workspaceName = ""
                    }
                    windowManager.openRepo(dest)
                }, onFailure: {
                    workspaceCreating = false
                })
            },
            content: {
                TextField("Workspace name", text: $workspaceName)
                    .textFieldStyle(.roundedBorder)
                    .jayjayFont(13, design: .monospaced)
                if let workspaceNameError {
                    Text(workspaceNameError)
                        .jayjayFont(11)
                        .foregroundStyle(.red)
                }
            }
        )
    }

    func requestWorkspaceDelete(_ workspace: WorkspaceInfo) {
        if settings.skipWorkspaceDeleteConfirmation {
            removeWorkspace(workspace, deleteFromDisk: true)
        } else {
            modal = .confirmWorkspaceDelete(workspace: workspace)
        }
    }

    private func workspaceDeleteSheet(workspace: WorkspaceInfo) -> some View {
        DestructiveConfirmSheet(
            title: "Delete Workspace \(workspace.name)?",
            message: "This closes its window, forgets the workspace, and deletes its directory from disk:\n\(workspace.path)",
            confirmLabel: "Delete",
            width: 400,
            dontAskAgain: Binding(
                get: { settings.skipWorkspaceDeleteConfirmation },
                set: { settings.skipWorkspaceDeleteConfirmation = $0 }
            ),
            onCancel: { modal = nil },
            onConfirm: {
                modal = nil
                removeWorkspace(workspace, deleteFromDisk: true)
            }
        )
    }

    func removeWorkspace(_ workspace: WorkspaceInfo, deleteFromDisk: Bool) {
        let viewModel = viewModel
        let windowManager = windowManager
        Task { @MainActor in
            await windowManager.withWorkspaceRemoval(workspace, repositoryStorePath: viewModel.repo.repositoryStorePath()) {
                await viewModel.forgetWorkspace(workspace, deleteFromDisk: deleteFromDisk)
            }
        }
    }

    private var submoduleAttentionSheet: some View {
        SubmoduleAttentionSheet(
            repoPath: viewModel.repoPath,
            submoduleStatuses: viewModel.submoduleAttentionItems,
            onClose: {
                viewModel.submoduleAttentionItems = []
                viewModel.pendingCommitMessage = nil
                modal = nil
            },
            onAutoCommit: { await viewModel.commitWithSafeSubmoduleUpdates() }
        )
    }

    private func submitRefCreate(_ onCreate: (String) -> Void) {
        let name = refCreateName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !name.isEmpty else { return }
        onCreate(name)
        modal = nil
    }

    private func rebaseSummaryRow(title: String, value: String, detail: String) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title)
                .jayjayFont(11, weight: .semibold)
                .foregroundStyle(.secondary)
            Text(value)
                .jayjayFont(13, weight: .medium)
                .lineLimit(1)
            Text(String(detail.prefix(12)))
                .jayjayFont(10, design: .monospaced)
                .foregroundStyle(.tertiary)
        }
    }
}
