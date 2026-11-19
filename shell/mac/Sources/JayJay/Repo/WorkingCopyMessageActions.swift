import JayJayCore
import SwiftUI

struct WorkingCopyMessageActions: View {
    let viewModel: RepoViewModel
    @Binding var summary: String
    @Binding var details: String
    @Binding var isGenerating: Bool
    /// The sidebar box also holds Generate and Describe and makes Commit its main action; a sheet has its own Save and puts Generate in its header.
    var isCommitBox = false
    /// The saved description a sheet edits, kept as it is when the fields were not touched.
    var original: String?
    var onCommitted: () -> Void = {}

    var body: some View {
        let message = original.map { updateCommitMessage(original: $0, summary: summary, body: details) }
            ?? joinCommitMessage(summary: summary, body: details)
        if isCommitBox {
            GenerateMessageButton(viewModel: viewModel, summary: $summary, details: $details, isGenerating: $isGenerating)
            Button(Localized.Repo.describe) { viewModel.describeWorkingCopy(message: message) }
                .buttonStyle(.bordered)
                .controlSize(.small)
                .disabled(message.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || viewModel.isCommitting)
                .help(Localized.Repo.saveDescriptionJjDescribe)
                .accessibilityIdentifier(AID.CommitBox.save)
        }
        let commit = Button {
            Task {
                if await viewModel.commit(message: message) {
                    onCommitted()
                }
            }
        } label: {
            if viewModel.isCommitting {
                ProgressView().controlSize(.small)
            } else {
                Text(Localized.Repo.commit)
            }
        }
        .disabled(summary.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || isGenerating || viewModel.isCommitting)
        .help(Localized.Repo.describeStartNewChangeJjCommit)
        .accessibilityIdentifier(AID.CommitBox.commit)
        if isCommitBox {
            commit
                .jayjayFont(12, weight: .semibold)
                .buttonStyle(.borderedProminent)
                .controlSize(.small)
        } else {
            commit
        }
    }
}
