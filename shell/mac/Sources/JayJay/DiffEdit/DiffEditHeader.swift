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
            Text(session.selectionSummary)
                .jayjayFont(11)
                .foregroundStyle(.secondary)
            Button {
                session.expandAllFiles()
            } label: {
                Label(Localized.Repo.expandAll, systemImage: "rectangle.expand.vertical")
            }
            .keyboardShortcut(AppShortcut.expandAllFiles)
            .accessibilityIdentifier(AID.DiffEdit.expandAll)
            Button {
                session.collapseAllFiles()
            } label: {
                Label(Localized.Repo.collapseAll, systemImage: "rectangle.compress.vertical")
            }
            .keyboardShortcut(AppShortcut.collapseAllFiles)
            .accessibilityIdentifier(AID.DiffEdit.collapseAll)
            Button {
                session.toggleBulkSelection()
            } label: {
                Label(session.selectionToggleTitle, systemImage: session.selectionToggleSystemImage)
            }
            .disabled(session.selectionToggleDisabled)
            .controlSize(.small)
            Button(Localized.Common.cancel, action: session.onDone)
                .keyboardShortcut(.cancelAction)
                .accessibilityIdentifier(AID.DiffEdit.cancel)
        }
        .controlSize(.small)
        .padding(.horizontal, 18)
        .padding(.vertical, 12)
    }
}
