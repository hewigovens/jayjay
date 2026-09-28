import JayJayCore
import SwiftUI

/// Tap targets, not Buttons: macOS drops a toolbar item that holds a wide Button.
struct RevsetBar: View {
    let actions: any RevsetActions
    let bookmarks: [BookmarkInfo]
    let vocabulary: RevsetVocabulary
    let editRequest: Int

    @State private var anchor = PickerAnchor()
    @State private var panel = PickerPanel()
    @State private var isEditing = false
    @State private var draft = ""
    @State private var completionPanel = RevsetCompletionPanel()
    @State private var completions: [RevsetCompletion] = []
    @State private var picked: Int?
    @State private var edit: FilterFieldEdit?
    /// Inserting a completion moves the caret, which must not reopen the list on what was just inserted.
    @State private var skipsCaretChange = false
    @Environment(\.jayjayFontSize) private var fontSize
    @Environment(\.jayjayFontFamily) private var fontFamily

    private var revset: String {
        actions.revsetFilter.revset
    }

    private var filter: RevsetFilter {
        revsetFilter(revset: revset)
    }

    private var isNarrowed: Bool {
        filter.kind != .default
    }

    var body: some View {
        HStack(spacing: 2) {
            if actions.revsetFilter.previous != nil, !isEditing {
                icon("chevron.backward", label: "Back to previous filter", stop: .revsetBack, action: actions.returnToPreviousRevset)
            }
            icon(
                filter.kind == .bookmark ? "bookmark" : "line.3.horizontal.decrease",
                label: "Presets, bookmarks and recent revsets",
                stop: .revsetPresets,
                tint: isNarrowed,
                showsTooltip: false,
                action: { togglePanel() }
            )
            if isEditing {
                FilterField(
                    text: $draft,
                    placeholder: "Revset",
                    accessibilityIdentifier: AID.Toolbar.revsetField,
                    isPlain: true,
                    endsEditingOnOutsideClick: true,
                    placesCaretAtEnd: true,
                    font: fontFamily.scaledNSFont(11, baseSize: fontSize, monospaced: true),
                    edit: edit,
                    onCaretChange: caretMoved,
                    onMove: movePick,
                    ownsWindow: { [completionPanel] in $0 === completionPanel },
                    onSubmit: submitDraft,
                    onCancel: cancel,
                    onEndEditing: {
                        showCompletions([])
                        isEditing = false
                    }
                )
            } else {
                summary
                    .onTapGesture(perform: beginEditing)
                    .accessibilityElement(children: .ignore)
                    .accessibilityAddTraits(.isButton)
                    .accessibilityLabel("Revset")
                    .accessibilityValue(revset)
                    .accessibilityAction(.default, beginEditing)
                    .accessibilityIdentifier(AID.Toolbar.revsetBar)
            }
            if isNarrowed, !isEditing {
                icon("xmark.circle.fill", label: "Reset to default", stop: .revsetReset) { actions.applyFilter("") }
            }
        }
        .padding(.leading, 4)
        .padding(.trailing, isNarrowed && !isEditing ? 4 : 10)
        .frame(minWidth: 280, idealWidth: 485, maxWidth: 485, minHeight: 28, maxHeight: 28)
        .background(background, in: Capsule())
        .overlay {
            if isEditing {
                Capsule().strokeBorder(Color.accentColor, lineWidth: 1.5)
            }
        }
        .background(PickerAnchorView(anchor: anchor))
        // On the bar, not the summary: the summary leaves while editing, and unregistering it would drop the Tab position.
        .keyboardFocusStop(.revsetFilter, action: beginEditing)
        .onChange(of: editRequest) {
            beginEditing()
        }
    }

    private var background: Color {
        if isEditing {
            return Color(nsColor: .textBackgroundColor)
        }
        return isNarrowed ? Color.accentColor.opacity(0.1) : Color.primary.opacity(0.04)
    }

    private var summary: some View {
        HStack(spacing: 7) {
            if !filter.label.isEmpty {
                Text(filter.label)
                    .jayjayFont(12, weight: isNarrowed ? .medium : .regular)
                    .foregroundStyle(isNarrowed ? Color.accentColor : .secondary)
                    .lineLimit(1)
            }
            if filter.kind != .bookmark {
                Text(revset)
                    .jayjayFont(11, design: .monospaced)
                    .foregroundStyle(filter.kind == .custom ? .primary : .tertiary)
                    .lineLimit(1)
                    .truncationMode(.tail)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .contentShape(Rectangle())
    }

    private func icon(
        _ systemName: String,
        label: String,
        stop: KeyboardFocusStop,
        tint: Bool = false,
        showsTooltip: Bool = true,
        action: @escaping () -> Void
    ) -> some View {
        Image(systemName: systemName)
            .foregroundStyle(tint ? Color.accentColor : .secondary)
            .frame(width: 26, height: 28)
            .contentShape(Rectangle())
            .keyboardFocusStop(stop, action: action)
            .onTapGesture(perform: action)
            .help(showsTooltip ? label : "")
            .accessibilityAddTraits(.isButton)
            .accessibilityLabel(label)
            .accessibilityAction(.default, action)
    }

    private func beginEditing() {
        panel.dismiss()
        draft = revset
        edit = nil
        skipsCaretChange = false
        isEditing = true
    }

    private func caretMoved(text: String, caret: Int) {
        if skipsCaretChange {
            skipsCaretChange = false
            return
        }
        showCompletions(revsetCompletions(text: text, cursor: UInt32(caret), vocabulary: vocabulary))
    }

    private func showCompletions(_ list: [RevsetCompletion], picked: Int? = nil) {
        completions = list
        self.picked = picked
        guard !list.isEmpty, let anchorView = anchor.view else {
            completionPanel.dismiss()
            return
        }
        completionPanel.show(
            under: anchorView,
            rows: RevsetCompletionRows(completions: list, selected: picked, onPick: accept),
            fontSize: fontSize,
            fontFamily: fontFamily
        )
    }

    private func movePick(_ delta: Int) -> Bool {
        guard !completions.isEmpty else { return false }
        let next = picked.map { min(max($0 + delta, 0), completions.count - 1) } ?? 0
        showCompletions(completions, picked: next)
        return true
    }

    private func accept(_ completion: RevsetCompletion) {
        skipsCaretChange = true
        edit = completion.edit(id: (edit?.id ?? 0) + 1)
        showCompletions([])
    }

    private func cancel() {
        if completions.isEmpty {
            isEditing = false
        } else {
            showCompletions([])
        }
    }

    private func submitDraft() {
        if let picked, completions.indices.contains(picked) {
            accept(completions[picked])
            return
        }
        showCompletions([])
        let text = draft.trimmingCharacters(in: .whitespacesAndNewlines)
        isEditing = false
        guard !text.isEmpty, text != revset else { return }
        if let error = actions.applyTyped(text, bookmarks: bookmarks) {
            togglePanel(query: text, error: error)
        }
    }

    private func togglePanel(query: String = "", error: String? = nil) {
        guard !panel.isVisible, !panel.wasJustDismissed else {
            panel.dismiss()
            return
        }
        guard let anchorView = anchor.view else { return }
        isEditing = false
        let content = RevsetPanelContent(
            actions: actions,
            bookmarks: bookmarks,
            vocabulary: vocabulary,
            query: query,
            error: error,
            onDismiss: { [weak panel] in panel?.dismiss() }
        )
        panel.show(under: anchorView, width: anchorView.bounds.width, centered: true, content: content)
    }
}
