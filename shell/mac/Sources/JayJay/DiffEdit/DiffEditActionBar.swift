import JayJayCore
import SwiftUI

struct DiffEditActionBar: View {
    let session: DiffEditViewModel

    var body: some View {
        VStack(spacing: 10) {
            Divider()
            HStack(spacing: 12) {
                Text(session.selectionSummary)
                    .jayjayFont(12, weight: .medium)
                Spacer()
                if !session.detail.info.isWorkingCopy {
                    TextField(Localized.Diff.newChangeDescription, text: Bindable(session).newChangeMessage)
                        .textFieldStyle(.roundedBorder)
                        .frame(width: 260)
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
            .disabled(session.isPreparingRemoval)
            .padding(.horizontal, 18)
            .padding(.bottom, 12)
        }
        .background(.bar)
    }
}
