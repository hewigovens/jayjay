import JayJayCore
import SwiftUI

/// GitHub-Desktop-style commit box: a single-line Summary + an optional
/// Description. They combine into jj's one change description (`summary\n\nbody`),
/// so the first line becomes the PR title and the body becomes the PR body.
struct CommitBox: View {
    let description: String
    @Binding var summary: String
    @Binding var details: String
    let onCommit: (String) async -> Bool
    let onGenerateMessage: () async -> String?
    let aiProvider: String

    @State private var isGenerating = false
    @State private var isCommitting = false

    private var trimmedSummary: String {
        summary.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Summary")
                .jayjayFont(11, weight: .semibold)
                .foregroundStyle(.secondary)

            TextField("Summary", text: $summary)
                .textFieldStyle(.plain)
                .jayjayFont(13, design: .monospaced)
                .padding(6)
                .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 8, style: .continuous))
                .overlay(
                    RoundedRectangle(cornerRadius: 8, style: .continuous)
                        .stroke(Color.primary.opacity(0.1), lineWidth: 1)
                )
                .accessibilityIdentifier(AID.CommitBox.summary)

            Text("Description")
                .jayjayFont(11, weight: .semibold)
                .foregroundStyle(.secondary)

            TextEditor(text: $details)
                .jayjayFont(13, design: .monospaced)
                .scrollContentBackground(.hidden)
                .padding(6)
                .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 8, style: .continuous))
                .overlay(
                    RoundedRectangle(cornerRadius: 8, style: .continuous)
                        .stroke(Color.primary.opacity(0.1), lineWidth: 1)
                )
                .frame(minHeight: 50, maxHeight: 100)
                .accessibilityIdentifier(AID.CommitBox.draft)

            HStack(spacing: 8) {
                Spacer()
                Button {
                    isGenerating = true
                    Task {
                        if let msg = await onGenerateMessage() {
                            split(msg)
                        }
                        isGenerating = false
                    }
                } label: {
                    if isGenerating {
                        ProgressView()
                            .controlSize(.mini)
                    } else {
                        Image(systemName: "sparkles")
                    }
                }
                .buttonStyle(.plain)
                .foregroundStyle(.secondary)
                .help(aiProvider.isEmpty ? "No AI available" : "Generate with \(aiProvider)")
                .disabled(isGenerating || aiProvider.isEmpty)

                Button(action: commit) {
                    if isCommitting {
                        ProgressView()
                            .controlSize(.small)
                    } else {
                        Text("Commit")
                            .jayjayFont(12, weight: .semibold)
                    }
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.small)
                .disabled(trimmedSummary.isEmpty || isCommitting)
                .help("Describe + start new change (jj commit)")
                .accessibilityIdentifier(AID.CommitBox.commit)
            }
        }
        .padding(12)
        .onAppear { prefillIfEmpty(description) }
        .onChange(of: description) { _, newValue in prefillIfEmpty(newValue) }
    }

    private func commit() {
        guard !trimmedSummary.isEmpty, !isCommitting else { return }
        let message = joinCommitMessage(summary: summary, body: details)
        isCommitting = true
        Task {
            _ = await onCommit(message)
            isCommitting = false
        }
    }

    /// Seed the fields from an existing description (first line → summary, rest →
    /// details), but only when the user hasn't started typing.
    private func prefillIfEmpty(_ message: String) {
        guard summary.isEmpty, details.isEmpty else { return }
        split(message)
    }

    private func split(_ message: String) {
        summary = commitSummary(message: message)
        details = commitBody(message: message)
    }
}
