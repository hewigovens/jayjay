import SwiftUI

struct DiffEditActionBar: View {
    let session: DiffEditSession

    var body: some View {
        VStack(spacing: 10) {
            Divider()
            HStack(spacing: 12) {
                Text(session.selectionSummary)
                    .jayjayFont(12, weight: .medium)
                Spacer()
                if !session.detail.info.isWorkingCopy {
                    TextField("New change description", text: Bindable(session).newChangeMessage)
                        .textFieldStyle(.roundedBorder)
                        .frame(width: 260)
                    Button("Create New Child Change") { session.apply(.newChild) }
                        .buttonStyle(.borderedProminent)
                    Button("Create Parallel Change") { session.apply(.newParallel) }
                        .buttonStyle(.bordered)
                    Button("Move to Working Copy") { session.apply(.moveToWorkingCopy) }
                        .buttonStyle(.bordered)
                }
                Button("Done") {
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
