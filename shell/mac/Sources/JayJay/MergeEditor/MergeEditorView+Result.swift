import JayJayCore
import SwiftUI

extension MergeEditorView {
    var resultPane: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack {
                Text("Result")
                    .jayjayFont(12, weight: .semibold)
                if hasHunkView {
                    resultModePicker
                }
                Spacer()
                Text(resultHint)
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            Divider()
            if session.resultMode == .hunks, hasHunkView {
                MergeHunkList(
                    highlights: session.highlights?.hunks ?? [],
                    result: session.result,
                    selectedHunk: $selectedHunk,
                    onUseSource: session.useHunkSource
                )
            } else {
                HighlightedRawResultView(
                    path: session.path,
                    text: $session.result,
                    isEditable: session.isText,
                    accessibilityIdentifier: AID.Conflict.editorResult,
                    onTextChanged: session.resultChanged,
                    preparedText: session.highlights?.resultText,
                    preparedHighlightedLines: session.highlights?.result
                )
            }
        }
    }

    private var resultModePicker: some View {
        Picker("Result view", selection: $session.resultMode) {
            Label("Hunks", systemImage: "square.split.2x1")
                .tag(MergeResultMode.hunks)
                .accessibilityIdentifier(AID.Conflict.editorHunks)
            Label("Raw", systemImage: "chevron.left.forwardslash.chevron.right")
                .tag(MergeResultMode.raw)
                .accessibilityIdentifier(AID.Conflict.editorRaw)
        }
        .pickerStyle(.segmented)
        .tint(Color(nsColor: .unemphasizedSelectedContentBackgroundColor))
        .labelsHidden()
        .controlSize(.small)
        .fixedSize()
        .onChange(of: session.resultMode) { _, mode in
            if mode == .hunks {
                selectFirstUnresolvedHunk()
            }
        }
    }

    private var resultHint: String {
        if let selectedSource = session.selectedSource {
            return "Using \(selectedSource.label)"
        }
        if session.isText {
            return "Edit freely; remaining markers are saved as a partial resolution."
        }
        return "Non-text conflicts can be resolved with Use Ours or Use Theirs."
    }

    private var hasHunkView: Bool {
        !(session.highlights?.hunks.isEmpty ?? true)
    }

    private var unresolvedHunks: [MergeHunkHighlights] {
        (session.highlights?.hunks ?? []).filter { mergeHunkIsUnresolved(result: session.result, hunk: $0.hunk) }
    }

    func selectFirstUnresolvedHunk() {
        if !unresolvedHunks.contains(where: { $0.id == selectedHunk }) {
            selectedHunk = unresolvedHunks.first?.id
        }
    }

    func useSelectedHunk(_ source: MergeHunkSource) {
        selectFirstUnresolvedHunk()
        guard let selectedHunk,
              let hunk = unresolvedHunks.first(where: { $0.id == selectedHunk })?.hunk
        else { return }
        session.useHunkSource(hunk, source)
        self.selectedHunk = unresolvedHunks.first(where: { $0.id != selectedHunk })?.id
    }

    func moveHunkSelection(_ delta: Int) {
        let hunks = unresolvedHunks
        guard !hunks.isEmpty else { selectedHunk = nil
            return
        }
        let current = hunks.firstIndex(where: { $0.id == selectedHunk }) ?? 0
        selectedHunk = hunks[(current + delta + hunks.count) % hunks.count].id
    }
}

private struct HighlightedRawResultView: View {
    let path: String
    @Binding var text: String
    let isEditable: Bool
    let accessibilityIdentifier: String
    let onTextChanged: () -> Void
    @State private var highlightedText: String?
    @State private var highlightedLines: [[DiffSpan]]?
    @State private var isReady: Bool

    init(
        path: String,
        text: Binding<String>,
        isEditable: Bool,
        accessibilityIdentifier: String,
        onTextChanged: @escaping () -> Void,
        preparedText: String?,
        preparedHighlightedLines: [[DiffSpan]]?
    ) {
        self.path = path
        _text = text
        self.isEditable = isEditable
        self.accessibilityIdentifier = accessibilityIdentifier
        self.onTextChanged = onTextChanged
        let ready = preparedText == text.wrappedValue && preparedHighlightedLines != nil
        _highlightedText = State(initialValue: ready ? preparedText : nil)
        _highlightedLines = State(initialValue: ready ? preparedHighlightedLines : nil)
        _isReady = State(initialValue: ready)
    }

    var body: some View {
        if isReady {
            CodeTextView(
                path: path,
                text: $text,
                isEditable: isEditable,
                wrapsLines: true,
                presentation: .editorPane,
                accessibilityIdentifier: accessibilityIdentifier,
                onTextChanged: onTextChanged,
                preparedText: highlightedText,
                preparedHighlightedLines: highlightedLines
            )
        } else {
            LoadingHUD(accessibilityIdentifier: AID.Conflict.editorPreparing)
                .task(id: text) {
                    let requestedText = text
                    let path = path
                    let lines = await Task.detached(priority: .userInitiated) {
                        highlightFileLines(path: path, content: requestedText)
                    }.value
                    guard !Task.isCancelled, text == requestedText else { return }
                    highlightedText = requestedText
                    highlightedLines = lines
                    isReady = true
                }
        }
    }
}
