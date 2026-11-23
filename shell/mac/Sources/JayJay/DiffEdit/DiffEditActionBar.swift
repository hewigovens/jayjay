import JayJayCore
import SwiftUI

struct DiffEditActionBar: View {
    let session: DiffEditViewModel
    @State private var isEditingDescription = false

    var body: some View {
        VStack(spacing: 10) {
            Divider()
            HStack(spacing: 12) {
                Text(session.selectionSummary)
                    .jayjayFont(12, weight: .medium)
                Spacer()
                descriptionButton
                if session.detail.info.isWorkingCopy {
                    Button(Localized.Diff.editCommitSelected) { session.apply(.newParent) }
                        .buttonStyle(.borderedProminent)
                        .help(Localized.Diff.editCommitSelectedHelp)
                } else {
                    Button(Localized.Repo.createNewChildChange) { session.apply(.newChild) }
                        .buttonStyle(.borderedProminent)
                    Button(Localized.Repo.createParallelChange) { session.apply(.newParallel) }
                        .buttonStyle(.bordered)
                    Button(Localized.Menu.moveToWorkingCopy) { session.apply(.moveToWorkingCopy) }
                        .buttonStyle(.bordered)
                }
                if session.discardedLines > 0 {
                    Text(Localized.Diff.editDiscards(count: Int(session.discardedLines)))
                        .jayjayFont(11, weight: .medium)
                        .foregroundStyle(.orange)
                }
                Button(Localized.Diff.editKeepSelected) {
                    session.apply(.removeFromSource)
                }
                .buttonStyle(.bordered)
                .help(Localized.Diff.editKeepSelectedHelp)
                .confirmationDialog(
                    Localized.Diff.editDiscardTitle(count: Int(session.pendingDiscard ?? 0)),
                    isPresented: Binding(
                        get: { session.pendingDiscard != nil },
                        set: {
                            if !$0 {
                                session.pendingDiscard = nil
                            }
                        }
                    )
                ) {
                    Button(Localized.Diff.editDiscardConfirm, role: .destructive) {
                        session.finishApply(.removeFromSource)
                    }
                    Button(Localized.Common.cancel, role: .cancel) {}
                } message: {
                    Text(Localized.Diff.editDiscardMessage)
                }
            }
            .disabled(session.isPreparingRemoval || session.isSelectingAll)
            .padding(.horizontal, 18)
            .padding(.bottom, 12)
        }
        .background(.bar)
        .sheet(isPresented: $isEditingDescription) {
            DescriptionEditorSheet(
                revision: String(session.detailRevision.prefix(12)),
                description: session.newChangeMessage,
                workingCopy: nil,
                onDismiss: { isEditingDescription = false },
                onSave: {
                    session.newChangeMessage = $0
                    isEditingDescription = false
                }
            )
        }
    }

    private var descriptionButton: some View {
        let summary = commitSummary(message: session.newChangeMessage)
        return Button {
            isEditingDescription = true
        } label: {
            Label(summary.isEmpty ? Localized.Repo.addDescriptionEllipsis : summary, systemImage: "pencil")
                .lineLimit(1)
                .truncationMode(.tail)
                .foregroundStyle(summary.isEmpty ? .secondary : .primary)
                .frame(width: 240, alignment: .leading)
        }
        .buttonStyle(.bordered)
        .help(Localized.Repo.editDescription)
        .accessibilityIdentifier(AID.DiffEdit.description)
    }
}
