import JayJayCore
import SwiftUI

private let sponsorPromptInterval = 20

extension RepoContentView {
    var overlayState: RepoOverlayState? {
        if viewModel.isLoading {
            return .loading
        }
        if let toast {
            return .toast(toast)
        }
        return nil
    }

    var presentationOverlay: some View {
        Group {
            switch overlayState {
                case .loading:
                    ProgressView()
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                        .background(.ultraThinMaterial)
                case let .toast(toast):
                    RepoToastView(
                        toast: toast,
                        dismiss: dismissToast,
                        colorScheme: colorScheme
                    )
                    .contentShape(Rectangle())
                    .onTapGesture { dismissToast() }
                    .transition(.scale(scale: 0.9).combined(with: .opacity))
                case nil:
                    EmptyView()
            }
        }
    }

    var alertState: RepoAlertState? {
        if let error = viewModel.error {
            return .error(error)
        }
        if let warning = viewModel.configWarning {
            return .configWarning(warning)
        }
        return nil
    }

    var isAlertPresented: Binding<Bool> {
        .init(
            get: { alertState != nil },
            set: { isPresented in
                guard !isPresented else { return }
                viewModel.error = nil
                viewModel.configWarning = nil
            }
        )
    }

    var alertTitle: String {
        switch alertState {
            case .error:
                "Error"
            case .configWarning:
                "jj Configuration Incomplete"
            case nil:
                ""
        }
    }

    @ViewBuilder
    func alertActions(for alert: RepoAlertState) -> some View {
        switch alert {
            case .error:
                Button("OK") { viewModel.error = nil }
            case .configWarning:
                Button("Open Settings") { openSettings() }
                Button("Dismiss", role: .cancel) { viewModel.configWarning = nil }
        }
    }

    func alertMessage(for alert: RepoAlertState) -> String {
        switch alert {
            case let .error(message), let .configWarning(message):
                message
        }
    }

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
                    onCleanUp: { viewModel.forgetStaleBookmarks() },
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

    func handleSuccessActionSignalChange() {
        settings.sponsorActionCount += 1
        if settings.sponsorActionCount >= settings.sponsorNextPromptCount,
           !settings.sponsorDismissed,
           modal == nil
        {
            settings.sponsorNextPromptCount = settings.sponsorActionCount + sponsorPromptInterval
            modal = .sponsorPrompt
        }
    }

    func handleSubmoduleAttentionChange() {
        if viewModel.submoduleAttentionItems.isEmpty {
            if case .submoduleAttention = modal {
                modal = nil
            }
        } else if modal == nil {
            modal = .submoduleAttention
        }
    }

    func showUndo() {
        viewModel.opLog()
        modal = .undoLog
    }

    func requestAbandon(_ rev: String) {
        if settings.skipAbandonConfirmation {
            viewModel.abandon(rev: rev)
        } else {
            modal = .confirmAbandon(rev: rev)
        }
    }

    func presentBookmarkCreate(rev: String) {
        bookmarkCreateName = ""
        modal = .createBookmark(rev: rev)
    }

    func presentStackedPr(rev: String) {
        modal = .stackedPr(rev: rev)
    }

    func handleDAGRebase(_ request: DAGRebaseRequest) {
        if settings.confirmDragRebase {
            modal = .confirmRebase(request: request)
        } else {
            runDAGRebase(request)
        }
    }

    func showToast(_ message: String) {
        showToast(message, action: nil)
    }

    func showToast(_ message: String, action: RepoToastAction?) {
        toastDismissTask?.cancel()
        toast = RepoToastState(message: message, action: action)
        let words = message.split(whereSeparator: \.isWhitespace).count
        let seconds = min(max(Double(words) / 3.0, 2), 8)
        toastDismissTask = Task {
            try? await Task.sleep(for: .seconds(seconds))
            guard !Task.isCancelled else { return }
            toast = nil
        }
    }

    private func dismissToast() {
        toastDismissTask?.cancel()
        toast = nil
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
        VStack(spacing: 16) {
            Image(systemName: "trash.circle.fill")
                .font(.system(size: 36))
                .foregroundStyle(.red)
            Text("Abandon Change?")
                .jayjayFont(16, weight: .semibold)
            Text("This will remove the change and reparent its children.\nYou can undo this with jj op restore.")
                .jayjayFont(13)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)

            Toggle("Don't ask again", isOn: Binding(
                get: { settings.skipAbandonConfirmation },
                set: { settings.skipAbandonConfirmation = $0 }
            ))
            .jayjayFont(12)

            HStack(spacing: 12) {
                Button("Cancel") { modal = nil }
                    .keyboardShortcut(.cancelAction)
                Button("Abandon") {
                    viewModel.abandon(rev: rev)
                    modal = nil
                }
                .keyboardShortcut(.defaultAction)
                .buttonStyle(.borderedProminent)
                .tint(.red)
            }
        }
        .padding(24)
        .frame(width: 340)
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
            confirmLabel: "Create",
            confirmDisabled: workspaceName.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
            onCancel: { modal = nil },
            onConfirm: {
                let name = workspaceName.trimmingCharacters(in: .whitespacesAndNewlines)
                guard !name.isEmpty else { return }
                let parent = URL(fileURLWithPath: viewModel.repoPath).deletingLastPathComponent()
                let dest = parent.appendingPathComponent(name).path
                viewModel.workspaceAdd(dest: dest, name: name)
                modal = nil
                workspaceName = ""
                windowManager.openRepo(dest)
            },
            content: {
                TextField("Workspace name", text: $workspaceName)
                    .textFieldStyle(.roundedBorder)
                    .jayjayFont(13, design: .monospaced)
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

    private func runDAGRebase(_ request: DAGRebaseRequest) {
        viewModel.rebase(
            request: request,
            onSuccess: { repoViewModel, feedback in
                let action = feedback.undoOperationId.map { operationId in
                    RepoToastAction(title: "Undo") {
                        repoViewModel.opRestore(opId: operationId)
                    }
                }
                showToast(feedback.message, action: action)
            },
            onFailure: { _, message in
                showToast(message)
            }
        )
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
