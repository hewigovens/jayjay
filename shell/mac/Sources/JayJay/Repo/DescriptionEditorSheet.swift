import JayJayCore
import SwiftUI

struct DescriptionEditorSheet: View {
    let revision: String
    let original: String
    let onCancel: () -> Void
    let onSave: (String) -> Void
    @State private var summary: String
    @State private var details: String

    init(revision: String, description: String, onCancel: @escaping () -> Void, onSave: @escaping (String) -> Void) {
        self.revision = revision
        original = description
        self.onCancel = onCancel
        self.onSave = onSave
        _summary = State(initialValue: commitSummary(message: description))
        _details = State(initialValue: commitBody(message: description))
    }

    var body: some View {
        SheetContainer(
            title: "Edit Description",
            subtitle: revision,
            inlineHeaderIcon: "pencil",
            cancelLabel: "Cancel",
            confirmLabel: "Save",
            onCancel: onCancel,
            onConfirm: { onSave(updateCommitMessage(original: original, summary: summary, body: details)) },
            content: {
                CommitMessageEditor(summary: $summary, details: $details, bodyHeight: 190, focusSummaryAtEnd: true)
            }
        )
        .frame(width: 520)
    }
}
