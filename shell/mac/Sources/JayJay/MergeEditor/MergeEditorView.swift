import JayJayCore
import SwiftUI

enum MergeResultMode: Hashable {
    case hunks
    case raw
}

struct MergeEditorView<Session: MergeEditingSession>: View {
    @Bindable var session: Session
    @State var showsBase = false
    @State var selectedHunk: UInt32?
    @State var scroll = MergeScrollCoordinator()

    let headerAccessibilityIdentifier: String?
    let onCancel: () -> Void
    let onSave: () -> Void

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            content
        }
        .background(
            KeyDownMonitor(
                isActive: { session.resultMode == .hunks },
                yieldsToText: \.isEditable,
                onKeyDown: handleMergeKey
            )
            .frame(width: 0, height: 0)
            .allowsHitTesting(false)
        )
        .onChange(of: session.resultMode, initial: true) { _, _ in
            updateScrollPresentation()
        }
        .onChange(of: selectedHunk) { _, hunk in scroll.reveal(hunk: hunk) }
        .task(id: session.isLoading ? nil : session.result) {
            guard let highlights = session.highlights else { return }
            scroll.invalidateResult()
            let result = session.result
            if result != highlights.resultText {
                try? await Task.sleep(for: .milliseconds(120))
            }
            guard !Task.isCancelled else { return }
            let map: MergeScrollMap = if result == highlights.resultText {
                highlights.scrollMap
            } else {
                await Task.detached(priority: .userInitiated) {
                    highlights.scrollMap.withResult(result: result)
                }.value
            }
            guard !Task.isCancelled, session.result == result else { return }
            scroll.update(map: map, hunks: highlights.hunks.map(\.id))
            updateScrollPresentation()
        }
    }

    private var header: some View {
        HStack(spacing: 10) {
            Image(systemName: "arrow.trianglehead.merge")
                .foregroundStyle(.orange)
            VStack(alignment: .leading, spacing: 2) {
                Text("Resolve Conflict")
                    .jayjayFont(14, weight: .semibold)
                    .accessibilityIdentifier(headerAccessibilityIdentifier ?? "")
                Text(session.path)
                    .jayjayFont(11, design: .monospaced)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                    .truncationMode(.middle)
            }
            Spacer()
            resolutionStatus
            Button("Cancel", action: onCancel)
                .keyboardShortcut(.cancelAction)
                .accessibilityIdentifier(AID.Conflict.editorCancel)
            Button(saveTitle, action: onSave)
                .keyboardShortcut(.defaultAction)
                .buttonStyle(.borderedProminent)
                .disabled(session.isLoading || session.isSaving || !session.canSave)
                .accessibilityIdentifier(AID.Conflict.editorSave)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
    }

    @ViewBuilder
    private var resolutionStatus: some View {
        if session.unresolvedCount > 0 {
            Label(
                "\(session.unresolvedCount) unresolved",
                systemImage: "exclamationmark.triangle.fill"
            )
            .foregroundStyle(.orange)
            .jayjayFont(11, weight: .semibold)
        } else if !session.isLoading, session.errorMessage == nil {
            if session.canSave {
                Label("Resolved", systemImage: "checkmark.circle.fill")
                    .foregroundStyle(.green)
                    .jayjayFont(11, weight: .semibold)
            } else {
                Label("Needs resolution", systemImage: "exclamationmark.triangle.fill")
                    .foregroundStyle(.orange)
                    .jayjayFont(11, weight: .semibold)
            }
        }
    }

    private var saveTitle: String {
        if session.isSaving {
            return "Saving…"
        }
        if session.unresolvedCount > 0 {
            return "Save Partial Resolution"
        }
        return "Save Resolution"
    }

    @ViewBuilder
    private var content: some View {
        if session.isLoading {
            ProgressView("Loading conflict sides…")
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else if let errorMessage = session.errorMessage {
            ContentUnavailableView(
                "Couldn’t Open Merge Session",
                systemImage: "exclamationmark.triangle",
                description: Text(errorMessage)
            )
        } else if session.showsSources {
            VSplitView {
                sourcesPane
                    .frame(minHeight: 220, idealHeight: 300)
                resultPane
                    .frame(minHeight: 320, idealHeight: 440)
                    .layoutPriority(1)
            }
        } else {
            resultPane
        }
    }
}
