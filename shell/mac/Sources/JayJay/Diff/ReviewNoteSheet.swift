import JayJayCore
import SwiftUI

extension ChangeDetailView {
    func saveReviewNote(editor: ReviewNoteEditorState, body: String) {
        let trimmed = body.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        if let noteId = editor.noteId {
            reviewStore.updateNote(id: noteId, body: trimmed)
        } else if let anchor = editor.anchor {
            reviewStore.addNote(
                anchor: NoteAnchor(
                    changeId: reviewChangeId,
                    path: editor.path,
                    identity: editor.identity,
                    side: NoteSide(anchor.side),
                    line: anchor.line,
                    anchorExcerpt: anchor.excerpt,
                    anchorContext: anchor.context,
                    ignoreWhitespace: appSettings.ignoreWhitespace
                ),
                body: trimmed
            )
        }
        noteEditor = nil
        refreshReviewState()
    }
}

struct ReviewNoteSheet: View {
    let editor: ReviewNoteEditorState
    let onCancel: () -> Void
    let onSave: (String) -> Void

    @State private var bodyText: String

    init(
        editor: ReviewNoteEditorState,
        onCancel: @escaping () -> Void,
        onSave: @escaping (String) -> Void
    ) {
        self.editor = editor
        self.onCancel = onCancel
        self.onSave = onSave
        _bodyText = State(initialValue: editor.body)
    }

    var body: some View {
        SheetContainer(
            title: editor.noteId == nil ? "Add Review Note" : "Edit Review Note",
            subtitle: nil,
            cancelLabel: "Cancel",
            confirmLabel: editor.noteId == nil ? "Add Note" : "Save",
            confirmDisabled: bodyText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty,
            onCancel: onCancel,
            onConfirm: { onSave(bodyText) },
            content: {
                if !editor.context.isEmpty {
                    contextPreview
                }
                TextEditor(text: $bodyText)
                    .font(.system(size: 13, design: .monospaced))
                    .frame(minHeight: 130)
                    .scrollContentBackground(.hidden)
                    .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 6))
                    .accessibilityIdentifier(AID.ReviewNote.body)
                HStack {
                    Spacer()
                    Text("⌘↩ to save")
                        .jayjayFont(10)
                        .foregroundStyle(.tertiary)
                }
                // The editor consumes plain Return for newlines, so the container's default-action shortcut never fires while typing; ⌘↩ still travels the key-equivalent chain.
                .background(
                    Button("") { save() }
                        .keyboardShortcut(.return, modifiers: .command)
                        .hidden()
                )
            }
        )
    }

    private var contextPreview: some View {
        VStack(alignment: .leading, spacing: 0) {
            ForEach(Array(editor.context.enumerated()), id: \.offset) { _, line in
                HStack(spacing: 0) {
                    Text(marker(for: line))
                        .frame(width: 16, alignment: .leading)
                        .foregroundStyle(.secondary)
                    Text(line.text.isEmpty ? " " : line.text)
                        .lineLimit(1)
                        .truncationMode(.tail)
                        .foregroundStyle(line.isAnchor ? .primary : .secondary)
                    Spacer(minLength: 0)
                }
                .font(.system(size: 11, design: .monospaced))
                .padding(.horizontal, 8)
                .padding(.vertical, 2)
                .background(background(for: line))
            }
        }
        .clipShape(RoundedRectangle(cornerRadius: 6))
        .overlay(
            RoundedRectangle(cornerRadius: 6)
                .stroke(Color.primary.opacity(0.08), lineWidth: 1)
        )
    }

    private func marker(for line: ReviewNoteContextLine) -> String {
        switch line.style {
            case .added: "+"
            case .removed: "-"
            default: " "
        }
    }

    private func background(for line: ReviewNoteContextLine) -> Color {
        if line.isAnchor {
            return .orange.opacity(0.16)
        }
        return switch line.style {
            case .added: .green.opacity(0.10)
            case .removed: .red.opacity(0.10)
            default: .primary.opacity(0.02)
        }
    }

    private func save() {
        guard !bodyText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }
        onSave(bodyText)
    }
}
