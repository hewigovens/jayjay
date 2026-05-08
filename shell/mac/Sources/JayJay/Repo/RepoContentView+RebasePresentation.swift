import SwiftUI

extension RepoContentView {
    func rebaseConfirmationSheet(request: DAGRebaseRequest) -> some View {
        SheetContainer(
            title: "\(request.placement.confirmationLabel) Change?",
            subtitle: "\(String(request.sourceCommitId.prefix(12))) -> \(String(request.destCommitId.prefix(12)))",
            cancelLabel: "Cancel",
            confirmLabel: request.placement.confirmationLabel,
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
                    Label(
                        rebasePlacementSummary(request.placement),
                        systemImage: rebasePlacementIcon(request.placement)
                    )
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
                    rebaseSummaryRow(
                        title: request.placement.targetRole,
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

    private func rebasePlacementSummary(_ placement: DAGRebasePlacement) -> String {
        switch placement {
            case .onto:
                "Will become a child of"
            case .after:
                "Will be inserted after; descendants move after it"
            case .before:
                "Will be inserted before; target and descendants move after it"
        }
    }

    private func rebasePlacementIcon(_ placement: DAGRebasePlacement) -> String {
        switch placement {
            case .onto: "arrow.down"
            case .after: "arrow.down.to.line"
            case .before: "arrow.up.to.line"
        }
    }
}
