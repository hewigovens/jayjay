import JayJayCore
import SwiftUI

struct DescriptionEditorSheet: View {
    let revision: String
    let original: String
    let workingCopy: RepoViewModel?
    let onDismiss: () -> Void
    let onSave: (String) -> Void
    @State private var isGenerating = false
    @State private var summary: String
    @State private var details: String

    init(
        revision: String, description: String, workingCopy: RepoViewModel?,
        onDismiss: @escaping () -> Void, onSave: @escaping (String) -> Void
    ) {
        self.workingCopy = workingCopy
        self.revision = revision
        original = description
        self.onDismiss = onDismiss
        self.onSave = onSave
        _summary = State(initialValue: commitSummary(message: description))
        _details = State(initialValue: commitBody(message: description))
    }

    var body: some View {
        SheetContainer(
            title: Localized.Repo.editDescription,
            subtitle: revision,
            inlineHeaderIcon: "pencil",
            cancelLabel: Localized.Common.cancel,
            confirmLabel: Localized.Common.save,
            onCancel: onDismiss,
            onConfirm: { onSave(updateCommitMessage(original: original, summary: summary, body: details)) },
            titleAccessory: workingCopy.map {
                AnyView(GenerateMessageButton(viewModel: $0, summary: $summary, details: $details, isGenerating: $isGenerating))
            },
            additionalActions: workingCopy.map {
                AnyView(WorkingCopyMessageActions(
                    viewModel: $0, summary: $summary, details: $details,
                    isGenerating: $isGenerating, original: original, onCommitted: onDismiss
                ))
            },
            content: {
                CommitMessageEditor(summary: $summary, details: $details, bodyHeight: 190 ... .infinity, focusSummaryAtEnd: true)
            }
        )
        .frame(minWidth: 520, idealWidth: 680, maxWidth: 900, minHeight: 360, idealHeight: 480, maxHeight: 680)
        .presentationSizing(.fitted)
        .disabled(workingCopy?.isCommitting == true)
        .interactiveDismissDisabled(workingCopy?.isCommitting == true)
    }
}
