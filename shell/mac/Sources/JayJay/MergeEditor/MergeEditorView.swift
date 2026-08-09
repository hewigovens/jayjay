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

    let headerAccessibilityIdentifier: String?
    let onCancel: () -> Void
    let onSave: () -> Void

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            content
        }
        .onKeyPress { press in
            guard session.resultMode == .hunks, press.modifiers == [.option] else { return .ignored }
            switch press.key {
                case .leftArrow:
                    useSelectedHunk(.left)
                    return .handled
                case .rightArrow:
                    useSelectedHunk(.right)
                    return .handled
                case .upArrow:
                    moveHunkSelection(-1)
                    return .handled
                case .downArrow:
                    moveHunkSelection(1)
                    return .handled
                default:
                    return .ignored
            }
        }
        .onAppear { selectFirstUnresolvedHunk() }
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
        if !session.canSave {
            return "Resolve All Conflicts"
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
