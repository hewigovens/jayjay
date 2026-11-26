import JayJayCore
import SwiftUI

struct GenerateMessageButton: View {
    @Environment(AppSettings.self) private var settings
    let viewModel: RepoViewModel
    @Binding var summary: String
    @Binding var details: String
    @Binding var isGenerating: Bool

    var body: some View {
        Button {
            isGenerating = true
        } label: {
            if isGenerating {
                ProgressView().controlSize(.mini)
            } else {
                Image(systemName: "sparkles")
            }
        }
        .iconButtonStyle()
        .foregroundStyle(.secondary)
        .help(viewModel.aiProvider.isEmpty ? Localized.Repo.noAiAvailable : Localized.Repo.generateWith(provider: viewModel.aiProvider))
        .disabled(isGenerating || viewModel.isCommitting || viewModel.aiProvider.isEmpty)
        .task(id: isGenerating) {
            guard isGenerating else { return }
            defer { isGenerating = false }
            let changeId = viewModel.workingCopyChangeId
            if let message = await viewModel.generateCommitMessage(using: settings.aiProviderOrder),
               !Task.isCancelled, viewModel.workingCopyChangeId == changeId
            {
                summary = commitSummary(message: message)
                details = commitBody(message: message)
            }
        }
        .task(id: settings.aiProviderOrder) {
            let label = await settings.aiProviderOrder.firstReadyLabel()
            if !Task.isCancelled {
                viewModel.aiProvider = label
            }
        }
    }
}
