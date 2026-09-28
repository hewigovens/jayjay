import JayJayCore
import SwiftUI

/// Tap targets, not Buttons: macOS drops a toolbar item that holds a wide Button.
struct RevsetBar: View {
    let actions: any RevsetActions
    let changeCount: Int
    let bookmarks: [BookmarkInfo]
    let editRequest: Int

    @State private var anchor = PickerAnchor()
    @State private var panel = PickerPanel()
    @State private var isEditing = false
    @State private var draft = ""
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
                action: { togglePanel() }
            )
            if isEditing {
                FilterField(
                    text: $draft,
                    placeholder: "Revset",
                    accessibilityIdentifier: AID.Toolbar.revsetField,
                    isPlain: true,
                    endsEditingOnOutsideClick: true,
                    font: fontFamily.scaledNSFont(11, baseSize: fontSize, monospaced: true),
                    onSubmit: submitDraft,
                    onCancel: { isEditing = false },
                    onEndEditing: { isEditing = false }
                )
            } else {
                summary
                    .onTapGesture(perform: beginEditing)
                    .help("Edit the revset (⌘L)")
                    .accessibilityElement(children: .ignore)
                    .accessibilityAddTraits(.isButton)
                    .accessibilityLabel("Revset")
                    .accessibilityValue(revset)
                    .accessibilityAction(.default, beginEditing)
                    .accessibilityIdentifier(AID.Toolbar.revsetBar)
            }
            Text("\(changeCount)")
                .jayjayFont(11)
                .foregroundStyle(.secondary)
                .padding(.trailing, isNarrowed && !isEditing ? 0 : 6)
            if isNarrowed, !isEditing {
                icon("xmark.circle.fill", label: "Reset to default", stop: .revsetReset) { actions.applyFilter("") }
            }
        }
        .padding(.horizontal, 4)
        .frame(minWidth: 280, idealWidth: 480, maxWidth: 480, minHeight: 28, maxHeight: 28)
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
            Spacer(minLength: 4)
        }
        .contentShape(Rectangle())
    }

    private func icon(
        _ systemName: String,
        label: String,
        stop: KeyboardFocusStop,
        tint: Bool = false,
        action: @escaping () -> Void
    ) -> some View {
        Image(systemName: systemName)
            .foregroundStyle(tint ? Color.accentColor : .secondary)
            .frame(width: 26, height: 28)
            .contentShape(Rectangle())
            .keyboardFocusStop(stop, action: action)
            .onTapGesture(perform: action)
            .help(label)
            .accessibilityAddTraits(.isButton)
            .accessibilityLabel(label)
            .accessibilityAction(.default, action)
    }

    private func beginEditing() {
        panel.dismiss()
        draft = revset
        isEditing = true
    }

    private func submitDraft() {
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
            query: query,
            error: error,
            onDismiss: { [weak panel] in panel?.dismiss() }
        )
        panel.show(under: anchorView, size: content.idealSize(width: anchorView.bounds.width), centered: true, content: content)
    }
}
