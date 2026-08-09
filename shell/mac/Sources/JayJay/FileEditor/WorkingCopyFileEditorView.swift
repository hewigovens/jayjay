import JayJayCore
import SwiftUI

struct WorkingCopyFileEditorView: View {
    typealias Save = (FileEditorData, String, @escaping @MainActor (Bool) -> Void) -> Void

    @State private var session: WorkingCopyFileEditorSession
    let onSave: Save
    let onDone: () -> Void

    init(session: WorkingCopyFileEditorSession, onSave: @escaping Save, onDone: @escaping () -> Void) {
        _session = State(initialValue: session)
        self.onSave = onSave
        self.onDone = onDone
    }

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            content
        }
    }

    private var header: some View {
        HStack(spacing: 10) {
            Image(systemName: "pencil.and.scribble")
                .foregroundStyle(Color.accentColor)
            VStack(alignment: .leading, spacing: 2) {
                Text("Edit Working-Copy File")
                    .jayjayFont(14, weight: .semibold)
                    .accessibilityIdentifier(AID.FileEditor.modal)
                Text(session.path)
                    .jayjayFont(11, design: .monospaced)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                    .truncationMode(.middle)
            }
            Spacer()
            if session.hasChanges {
                Text("Modified")
                    .jayjayFont(11, weight: .semibold)
                    .foregroundStyle(.orange)
            }
            Button("Cancel", action: onDone)
                .keyboardShortcut(.cancelAction)
                .accessibilityIdentifier(AID.FileEditor.cancel)
            Button(session.isSaving ? "Saving…" : "Save", action: save)
                .keyboardShortcut("s")
                .buttonStyle(.borderedProminent)
                .disabled(session.isSaving || !session.hasChanges)
                .accessibilityIdentifier(AID.FileEditor.save)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
    }

    @ViewBuilder
    private var content: some View {
        if let errorMessage = session.errorMessage {
            ContentUnavailableView(
                "Couldn’t Open File Editor",
                systemImage: "exclamationmark.triangle",
                description: Text(errorMessage)
            )
        } else {
            VStack(alignment: .leading, spacing: 0) {
                HStack {
                    Text("Edits are saved directly to the current working-copy change.")
                        .jayjayFont(11)
                        .foregroundStyle(.secondary)
                    Spacer()
                }
                .padding(.horizontal, 12)
                .padding(.vertical, 8)
                Divider()
                CodeTextView(
                    path: session.path,
                    text: Bindable(session).content,
                    isEditable: true,
                    wrapsLines: true,
                    presentation: .editorPane,
                    accessibilityIdentifier: AID.FileEditor.content,
                    preparedText: session.data?.content,
                    preparedHighlightedLines: session.highlightedLines
                )
            }
        }
    }

    private func save() {
        guard let data = session.data, session.hasChanges, !session.isSaving else { return }
        session.isSaving = true
        onSave(data, session.content) { success in
            session.isSaving = false
            if success {
                onDone()
            }
        }
    }
}
