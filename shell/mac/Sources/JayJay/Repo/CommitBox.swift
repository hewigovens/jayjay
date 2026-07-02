import JayJayCore
import SwiftUI

/// Commit box: Summary line + optional Description, combined into jj's `summary\n\nbody` description.
struct CommitBox: View {
    let description: String
    @Binding var summary: String
    @Binding var details: String
    let onSaveDescription: (String) -> Void
    let onCommit: (String) async -> Bool
    let onGenerateMessage: () async -> String?
    let aiProvider: String

    @State private var isGenerating = false
    @State private var isCommitting = false

    private var trimmedSummary: String {
        summary.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private var draftMessage: String {
        joinCommitMessage(summary: summary, body: details)
    }

    private var trimmedDraft: String {
        draftMessage.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            TextField("Summary (required)", text: $summary)
                .textFieldStyle(.plain)
                .jayjayFont(13, design: .monospaced)
                .padding(6)
                .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 8, style: .continuous))
                .overlay(
                    RoundedRectangle(cornerRadius: 8, style: .continuous)
                        .stroke(Color.primary.opacity(0.1), lineWidth: 1)
                )
                .accessibilityIdentifier(AID.CommitBox.summary)

            // TextEditor has no native placeholder; overlay one while empty.
            TextEditor(text: $details)
                .jayjayFont(13, design: .monospaced)
                .scrollContentBackground(.hidden)
                .accessibilityIdentifier(AID.CommitBox.draft)
                .overlay(alignment: .topLeading) {
                    if details.isEmpty {
                        Text("Description")
                            .jayjayFont(13, design: .monospaced)
                            .foregroundStyle(.tertiary)
                            // Match the TextEditor's text origin so the placeholder aligns with the cursor.
                            .padding(.leading, 5)
                            .padding(.top, 0)
                            .allowsHitTesting(false)
                            .accessibilityHidden(true)
                    }
                }
                .padding(6)
                .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 8, style: .continuous))
                .overlay(
                    RoundedRectangle(cornerRadius: 8, style: .continuous)
                        .stroke(Color.primary.opacity(0.1), lineWidth: 1)
                )
                .frame(minHeight: 50, maxHeight: 100)

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

                Button("Describe", action: saveDescription)
                    .buttonStyle(.bordered)
                    .controlSize(.small)
                    .disabled(trimmedDraft.isEmpty || isCommitting)
                    .help("Save description (jj describe)")
                    .accessibilityIdentifier(AID.CommitBox.save)

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
        let message = draftMessage
        isCommitting = true
        Task {
            _ = await onCommit(message)
            isCommitting = false
        }
    }

    private func saveDescription() {
        guard !trimmedDraft.isEmpty, !isCommitting else { return }
        onSaveDescription(draftMessage)
    }

    /// Seed the fields from an existing description, unless the user has started typing.
    private func prefillIfEmpty(_ message: String) {
        guard summary.isEmpty, details.isEmpty else { return }
        split(message)
    }

    private func split(_ message: String) {
        summary = commitSummary(message: message)
        details = commitBody(message: message)
    }
}
