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
                Button(Localized.Repo.done) {
                    session.apply(.removeFromSource)
                }
                .buttonStyle(.bordered)
            }
            .disabled(session.isPreparingRemoval)
            .padding(.horizontal, 18)
            .padding(.bottom, 12)
        }
        .background(.bar)
    }
}
