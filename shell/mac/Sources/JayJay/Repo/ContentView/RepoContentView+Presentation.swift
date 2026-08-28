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
            modal = .confirmChange(.abandon(rev: rev))
        }
    }

    func requestAbandonSelection(_ revisions: [String]) {
        modal = .confirmChange(.abandonSelection(revisions: revisions))
    }

    func requestSquashSelection(_ revisions: [String]) {
        modal = .confirmChange(.squashSelection(revisions: revisions))
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
            modal = .confirmChange(.rebase(request: request))
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

    func runDAGRebase(_ request: DAGRebaseRequest) {
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
}
