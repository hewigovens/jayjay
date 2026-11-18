import JayJayCore
import SwiftUI

struct CommitBox: View {
    @Bindable var viewModel: RepoViewModel
    @State private var isGenerating = false

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            CommitMessageEditor(
                summary: $viewModel.commitSummaryDraft, details: $viewModel.commitDescriptionDraft,
                participatesInPaneNavigation: true
            )
            HStack(spacing: 8) {
                Spacer()
                WorkingCopyMessageActions(
                    viewModel: viewModel, summary: $viewModel.commitSummaryDraft,
                    details: $viewModel.commitDescriptionDraft,
                    isGenerating: $isGenerating, isCommitBox: true
                )
            }
        }
        .padding(12)
        .onChange(of: viewModel.workingCopyDescription, initial: true) { _, message in
            guard viewModel.commitSummaryDraft.isEmpty, viewModel.commitDescriptionDraft.isEmpty else { return }
            viewModel.commitSummaryDraft = commitSummary(message: message)
            viewModel.commitDescriptionDraft = commitBody(message: message)
        }
    }
}
