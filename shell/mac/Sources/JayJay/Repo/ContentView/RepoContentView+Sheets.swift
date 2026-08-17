import JayJayCore
import SwiftUI

extension RepoContentView {
    @ViewBuilder
    func modalView(for modal: RepoModalState) -> some View {
        switch modal {
            case let .createBookmark(rev):
                bookmarkCreateSheet(rev: rev)
            case let .stackedPr(rev):
                StackedPrPanel(viewModel: viewModel, tipRev: rev, onDismiss: { self.modal = nil })
            case let .confirmAbandon(rev):
                abandonSheet(rev: rev)
            case let .confirmRebase(request):
                rebaseConfirmationSheet(request: request)
            case .submoduleAttention:
                submoduleAttentionSheet
            case .undoLog:
                UndoView(
                    entries: viewModel.opLogEntries,
                    onRestore: { opId in viewModel.opRestore(opId: opId) },
                    onDismiss: { self.modal = nil }
                )
            case .bookmarkManager:
                BookmarkManagerView(
                    bookmarks: viewModel.bookmarks,
                    actions: viewModel,
                    repo: viewModel.repo,
                    prHostName: viewModel.prHostName,
                    onFilter: { bookmarkName in
                        self.modal = nil
                        revsetDraft = "ancestors(\(bookmarkName), 20) | trunk()"
                        applyRevset()
                    },
                    onDiffBookmark: { request in
                        self.modal = nil
                        viewModel.diffBookmark(request)
                    },
                    onDismiss: { self.modal = nil }
                )
            case .workspaceCreate:
                workspaceCreateSheet
            case let .confirmWorkspaceDelete(name, path, operationId):
                workspaceDeleteSheet(name: name, path: path, operationId: operationId)
            case .sponsorPrompt:
                SponsorPromptView(
                    onDismiss: { self.modal = nil },
                    onDontShowAgain: {
                        settings.sponsorDismissed = true
                        self.modal = nil
                    }
                )
        }
    }

    private func bookmarkCreateSheet(rev: String) -> some View {
        SheetContainer(
            title: "Create Bookmark",
            subtitle: "On change: \(String(rev.prefix(12)))",
            cancelLabel: "Cancel",
            confirmLabel: "Create",
            confirmDisabled: bookmarkCreateName.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
            onCancel: { modal = nil },
            onConfirm: { submitBookmarkCreate(rev: rev) },
            content: {
                TextField("Bookmark name", text: $bookmarkCreateName)
                    .textFieldStyle(.roundedBorder)
                    .jayjayFont(13, design: .monospaced)
                    .onSubmit { submitBookmarkCreate(rev: rev) }
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

    private func rebaseConfirmationSheet(request: DAGRebaseRequest) -> some View {
        SheetContainer(
            title: "Rebase Change?",
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

    private func workspaceDeleteSheet(name: String, path: String, operationId: String) -> some View {
        DestructiveConfirmSheet(
            title: "Delete Workspace \(name)?",
            message: "This closes its window, forgets the workspace, and deletes its directory from disk:\n\(path)",
            confirmLabel: "Delete",
            width: 400,
            onCancel: { modal = nil },
            onConfirm: {
                modal = nil
                let settings = settings
                let viewModel = viewModel
                let windowManager = windowManager
                Task { @MainActor in
                    // Wait out the target window's in-flight repo tasks before touching its checkout, or the move below could race a snapshot or mutation still running there.
                    await windowManager.closeRepoWindowForWorkspaceRemoval(at: path)
                    defer { windowManager.finishWorkspaceRemoval(at: path) }
                    // Capture the directory identity before core validation, then verify that exact object after the move so replacement at any point fails closed.
                    let quarantined: QuarantinedDirectory
                    let warning: String?
                    do {
                        let identity = try await Task.detached {
                            try QuarantinedDirectory.identity(path: path)
                        }.value
                        let operation = try await viewModel.workspaceRemovalGuard(
                            name: name,
                            expectedRoot: path,
                            expectedOperation: operationId
                        )
                        quarantined = try await Task.detached {
                            try QuarantinedDirectory.capture(
                                path: path,
                                expectedIdentity: identity
                            )
                        }.value
                        do {
                            warning = try await viewModel.workspaceForget(
                                name: name,
                                expectedRoot: path,
                                expectedOperation: operation
                            )
                        } catch {
                            let forgetError = error
                            do {
                                try await Task.detached { try quarantined.restore() }.value
                                viewModel.present(error: forgetError)
                            } catch {
                                viewModel.error = "The forget failed and the workspace directory could not be moved back. It is preserved at:\n\(quarantined.quarantineURL.path)"
                            }
                            return
                        }
                    } catch {
                        viewModel.present(error: error)
                        return
                    }
                    settings.removeRecentRepo(path)
                    do {
                        try await Task.detached { try quarantined.delete() }.value
                    } catch {
                        viewModel.error = "The workspace was forgotten, but its quarantined directory could not be deleted. It is preserved at:\n\(quarantined.quarantineURL.path)"
                        return
                    }
                    if let warning {
                        viewModel.error = warning
                    }
                }
            }
        )
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

    private func submitBookmarkCreate(rev: String) {
        let name = bookmarkCreateName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !name.isEmpty else { return }
        viewModel.createBookmark(name: name, rev: rev)
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
