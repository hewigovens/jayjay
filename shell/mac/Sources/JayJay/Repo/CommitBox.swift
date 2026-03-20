import SwiftUI

struct CommitBox: View {
    let description: String
    let onCommit: (String) -> Void
    let onGenerateMessage: () async -> String?
    let aiProvider: String

    @State private var draft = ""
    @State private var isGenerating = false

    private var trimmedDraft: String {
        draft.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Description")
                .jayjayFont(11, weight: .semibold)
                .foregroundStyle(.secondary)

            TextEditor(text: $draft)
                .jayjayFont(13, design: .monospaced)
                .scrollContentBackground(.hidden)
                .padding(6)
                .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 8, style: .continuous))
                .overlay(
                    RoundedRectangle(cornerRadius: 8, style: .continuous)
                        .stroke(Color.primary.opacity(0.1), lineWidth: 1)
                )
                .frame(minHeight: 60, maxHeight: 120)

            HStack(spacing: 8) {
                Spacer()
                Button {
                    isGenerating = true
                    Task {
                        if let msg = await onGenerateMessage() {
                            draft = msg
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

                Button {
                    if !trimmedDraft.isEmpty {
                        let msg = trimmedDraft
                        draft = ""
                        onCommit(msg)
                    }
                } label: {
                    Text("Commit")
                        .jayjayFont(12, weight: .semibold)
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.small)
                .disabled(trimmedDraft.isEmpty)
                .help("Describe + start new change (jj commit)")
            }
        }
        .padding(12)
        .onAppear { draft = description }
        .onChange(of: description) { _, newValue in draft = newValue }
    }
}
