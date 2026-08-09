import JayJayCore
import SwiftUI

struct ExternalDiffToolView: View {
    @State private var session: ExternalDiffSession

    init(left: String, right: String, editable: Bool, onLoadFailure: @escaping () -> Void) {
        _session = State(initialValue: ExternalDiffSession(
            left: left,
            right: right,
            editable: editable,
            onLoadFailure: onLoadFailure
        ))
    }

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            content
        }
        .frame(minWidth: 900, minHeight: 620)
        .task { await session.load() }
    }

    private var header: some View {
        HStack(spacing: 10) {
            Image(systemName: session.editable ? "rectangle.and.pencil.and.ellipsis" : "square.split.2x1")
                .foregroundStyle(Color.accentColor)
            VStack(alignment: .leading, spacing: 2) {
                Text(session.editable ? "Edit Diff" : "Folder Comparison")
                    .jayjayFont(14, weight: .semibold)
                    .accessibilityIdentifier(AID.ExternalTool.diff)
                Text(session.editable
                    ? "Keep the checked files and lines in the edited result."
                    : "Reviewing the exact left and right snapshots supplied by the calling tool.")
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
            }
            Spacer()
            Text("\(session.files.count) changed \(session.files.count == 1 ? "file" : "files")")
                .jayjayFont(11, design: .monospaced)
                .foregroundStyle(.secondary)
            Button("Cancel", action: session.cancel)
                .keyboardShortcut(.cancelAction)
            if session.editable {
                Button(session.isSaving ? "Saving…" : "Done", action: session.save)
                    .keyboardShortcut(.defaultAction)
                    .buttonStyle(.borderedProminent)
                    .disabled(!session.canSave)
                    .accessibilityIdentifier(AID.ExternalTool.save)
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
    }

    @ViewBuilder
    private var content: some View {
        if session.isLoading {
            ProgressView("Loading comparison…")
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else if let error = session.errorMessage {
            ContentUnavailableView(
                "Couldn’t Open Tool Session",
                systemImage: "exclamationmark.triangle",
                description: Text(error)
            )
        } else if session.files.isEmpty {
            ContentUnavailableView(
                "No Differences",
                systemImage: "checkmark.circle",
                description: Text("The supplied snapshots have identical contents.")
            )
        } else {
            ScrollView {
                LazyVStack(spacing: 12) {
                    ForEach(session.files) { file in
                        ExternalDiffFileCard(
                            file: file,
                            editable: session.editable,
                            onToggleFile: { session.toggleFile(file) }
                        )
                    }
                }
                .padding(14)
            }
        }
    }
}
