import AppKit
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
                    scroll: scroll,
                    onUseSource: useHunkSource
                )
            } else {
                HighlightedRawResultView(
                    path: session.path,
                    text: $session.result,
                    isEditable: session.isText,
                    accessibilityIdentifier: AID.Conflict.editorResult,
                    onTextChanged: session.resultChanged,
                    preparedText: session.highlights?.resultText,
                    preparedHighlightedLines: session.highlights?.result,
                    scroll: scroll
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

    func updateScrollPresentation() {
        scroll.isRaw = session.resultMode == .raw || !hasHunkView
        if session.resultMode == .hunks {
            ensureHunkSelection()
        }
        scroll.reveal(hunk: selectedHunk)
    }

    private func ensureHunkSelection() {
        if !(session.highlights?.hunks.contains(where: { $0.id == selectedHunk }) ?? false) {
            selectedHunk = unresolvedHunks.first?.id
        }
    }

    func useSelectedHunk(_ source: MergeHunkSource) {
        ensureHunkSelection()
        guard let selectedHunk,
              let hunk = unresolvedHunks.first(where: { $0.id == selectedHunk })?.hunk
        else { return }
        useHunkSource(hunk, source)
    }

    func useHunkSource(_ hunk: MergeEditorHunk, _ source: MergeHunkSource) {
        guard session.useHunkSource(hunk, source) else { return }
        let remaining = unresolvedHunks
        selectedHunk = remaining.first(where: { $0.id > hunk.index })?.id
            ?? remaining.first(where: { $0.id != hunk.index })?.id
    }

    func handleMergeKey(_ event: NSEvent) -> Bool {
        guard event.modifierFlags.intersection([.command, .control, .option, .shift]) == .option else { return false }
        switch event.keyCode {
            case KeyCode.leftArrow: useSelectedHunk(.left)
            case KeyCode.rightArrow: useSelectedHunk(.right)
            case KeyCode.upArrow: moveHunkSelection(-1)
            case KeyCode.downArrow: moveHunkSelection(1)
            default: return false
        }
        return true
    }

    func moveHunkSelection(_ delta: Int) {
        let hunks = unresolvedHunks
        guard let current = selectedHunk else {
            selectedHunk = delta > 0 ? hunks.first?.id : hunks.last?.id
            return
        }
        if delta > 0 {
            selectedHunk = hunks.first(where: { $0.id > current })?.id ?? hunks.first?.id
        } else {
            selectedHunk = hunks.last(where: { $0.id < current })?.id ?? hunks.last?.id
        }
    }
}

private struct HighlightedRawResultView: View {
    let path: String
    @Binding var text: String
    let isEditable: Bool
    let accessibilityIdentifier: String
    let onTextChanged: () -> Void
    let scroll: MergeScrollCoordinator
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
        preparedHighlightedLines: [[DiffSpan]]?,
        scroll: MergeScrollCoordinator
    ) {
        self.path = path
        _text = text
        self.isEditable = isEditable
        self.accessibilityIdentifier = accessibilityIdentifier
        self.onTextChanged = onTextChanged
        self.scroll = scroll
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
                preparedHighlightedLines: highlightedLines,
                mergeScroll: scroll,
                mergePane: .result
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
