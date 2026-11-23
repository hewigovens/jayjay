import JayJayCore
import SwiftUI

struct DiffEditHeader: View {
    let session: DiffEditViewModel

    var body: some View {
        HStack(spacing: 12) {
            Label(Localized.Repo.diffEdit, systemImage: "slider.horizontal.3")
                .jayjayFont(15, weight: .semibold)
            Text(String(session.detailRevision.prefix(12)))
                .jayjayFont(12, design: .monospaced)
                .foregroundStyle(.secondary)
            Spacer()
            FileCollapseButtons(onExpandAll: session.expandAllFiles, onCollapseAll: session.collapseAllFiles)
            Button {
                session.toggleBulkSelection()
            } label: {
                Label(session.selectionToggleTitle, systemImage: session.selectionToggleSystemImage)
            }
            .disabled(session.selectionToggleDisabled)
            .controlSize(.small)
            DiscardingCancelButton(hasChanges: session.hasSelectionChanges, onCancel: session.onDone)
                .accessibilityIdentifier(AID.DiffEdit.cancel)
        }
        .controlSize(.small)
        .padding(.horizontal, 18)
        .padding(.vertical, 12)
    }
}
