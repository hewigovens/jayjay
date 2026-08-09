import SwiftUI

struct ExternalMergeToolView: View {
    @State private var session: ExternalMergeSession

    init(
        left: String,
        base: String,
        right: String,
        output: String,
        path: String,
        outputIsInitialized: Bool,
        markerLength: UInt32,
        onLoadFailure: @escaping () -> Void
    ) {
        _session = State(initialValue: ExternalMergeSession(
            left: left,
            base: base,
            right: right,
            output: output,
            path: path,
            outputIsInitialized: outputIsInitialized,
            markerLength: markerLength,
            onLoadFailure: onLoadFailure
        ))
    }

    var body: some View {
        MergeEditorView(
            session: session,
            headerAccessibilityIdentifier: AID.ExternalTool.merge,
            onCancel: session.cancel,
            onSave: session.save
        )
        .frame(minWidth: 900, minHeight: 620)
        .task { await session.load() }
    }
}
