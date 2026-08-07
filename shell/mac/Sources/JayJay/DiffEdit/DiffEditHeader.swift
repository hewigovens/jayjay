import SwiftUI

struct DiffEditHeader: View {
    let session: DiffEditSession

    var body: some View {
        HStack(spacing: 12) {
            Label("Diff Edit", systemImage: "slider.horizontal.3")
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
                Label("Expand All", systemImage: "rectangle.expand.vertical")
            }
            .keyboardShortcut("e", modifiers: [.command, .option])
            .accessibilityIdentifier(AID.DiffEdit.expandAll)
            Button {
                session.collapseAllFiles()
            } label: {
                Label("Collapse All", systemImage: "rectangle.compress.vertical")
            }
            .keyboardShortcut("c", modifiers: [.command, .option])
            .accessibilityIdentifier(AID.DiffEdit.collapseAll)
            Button {
                session.toggleBulkSelection()
            } label: {
                Label(session.selectionToggleTitle, systemImage: session.selectionToggleSystemImage)
            }
            .disabled(session.selectionToggleDisabled)
            .controlSize(.small)
            Button("Cancel", action: session.onDone)
                .keyboardShortcut(.cancelAction)
                .accessibilityIdentifier(AID.DiffEdit.cancel)
        }
        .controlSize(.small)
        .padding(.horizontal, 18)
        .padding(.vertical, 12)
        .background(.background)
    }
}
