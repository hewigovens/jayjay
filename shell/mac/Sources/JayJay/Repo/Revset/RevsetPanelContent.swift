import JayJayCore
import SwiftUI

struct RevsetPanelContent: View {
    private static let fieldHeight: CGFloat = 40
    private static let maxRowsHeight: CGFloat = 320
    private static let rowHeight: CGFloat = 28
    private static let sectionTitleHeight: CGFloat = 25

    let actions: any RevsetActions
    let bookmarks: [BookmarkInfo]
    let workspaces: [String]
    let vocabulary: RevsetVocabulary
    let onDismiss: () -> Void
    private let revset: String

    @State private var query: String
    @State private var error: String?
    /// Counts through the completions first, then `rows`.
    @State private var selectedIndex: Int?
    @State private var completions: [RevsetCompletion] = []
    @State private var edit: FilterFieldEdit?
    @State private var focusRequest = 0
    @Environment(\.jayjayTypography) private var typography

    init(
        actions: any RevsetActions,
        bookmarks: [BookmarkInfo],
        workspaces: [String],
        vocabulary: RevsetVocabulary,
        query: String = "",
        error: String? = nil,
        onDismiss: @escaping () -> Void
    ) {
        self.actions = actions
        self.bookmarks = bookmarks
        self.workspaces = workspaces
        self.vocabulary = vocabulary
        self.onDismiss = onDismiss
        revset = actions.revsetFilter.revset
        _query = State(initialValue: query)
        _error = State(initialValue: error)
    }

    private var trimmedQuery: String {
        query.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private var presets: [RevsetPreset] {
        [actions.defaultRevsetPreset] + RevsetFilterPresets.all
    }

    private var rows: [RevsetSuggestion] {
        revsetSuggestions(query: query, state: actions.revsetFilter, bookmarks: bookmarks, workspaces: workspaces)
    }

    var body: some View {
        VStack(spacing: 0) {
            field
            Divider()
            if let error {
                errorBanner(error)
            }
            chips
            ScrollView(.vertical) {
                VStack(alignment: .leading, spacing: 0) {
                    let rows = rows
                    if !completions.isEmpty {
                        sectionTitle(Localized.Repo.completions)
                        RevsetCompletionRows(completions: completions, selected: selectedIndex, onPick: accept)
                    }
                    section(Localized.Repo.current, rows.filter { $0.kind == .current })
                    section(Localized.Repo.bookmarks, rows.filter { $0.kind == .bookmark })
                    section(Localized.Repo.workspaces, rows.filter { $0.kind == .workspace })
                    section(Localized.Repo.recent, rows.filter { $0.kind == .recent })
                    if rows.isEmpty, completions.isEmpty {
                        Text(Localized.Repo.returnAppliesItAsARevset)
                            .jayjayFont(12)
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity)
                            .padding(.vertical, 16)
                    }
                }
                .padding(.bottom, 4)
            }
            .frame(maxHeight: Self.maxRowsHeight)
            Divider()
            footer
        }
        .glassEffect(in: RoundedRectangle(cornerRadius: 12))
        .clipShape(RoundedRectangle(cornerRadius: 12))
        .onChange(of: query) {
            error = nil
            selectedIndex = nil
        }
    }

    private var field: some View {
        HStack(spacing: 8) {
            Image(systemName: error == nil ? "line.3.horizontal.decrease" : "exclamationmark.triangle")
                .foregroundStyle(error == nil ? AnyShapeStyle(.secondary) : AnyShapeStyle(.red))
                .frame(width: 14)
            FilterField(
                text: $query,
                placeholder: Localized.Repo.revsetBookmarkOrPreset,
                accessibilityIdentifier: AID.Toolbar.revsetField,
                focusGeneration: focusRequest,
                isPlain: true,
                font: typography.nsFont(13, monospaced: true),
                edit: edit,
                onCaretChange: caretMoved,
                onMove: move,
                onComplete: complete,
                onSubmit: submit,
                onCancel: onDismiss
            )
        }
        .padding(.horizontal, 12)
        .frame(height: Self.fieldHeight)
    }

    private func errorBanner(_ message: String) -> some View {
        Text(message)
            .jayjayFont(11, design: .monospaced)
            .foregroundStyle(.red)
            .textSelection(.enabled)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .background(Color.red.opacity(0.08))
    }

    private var chips: some View {
        FlowLayout {
            ForEach(presets, id: \.id) { preset in
                let isActive = preset.revset == revset
                Button {
                    apply(preset.revset)
                } label: {
                    Text(preset.label)
                        .jayjayFont(12)
                        .foregroundStyle(isActive ? Color.accentColor : .primary)
                        .padding(.horizontal, 10)
                        .frame(height: 24)
                        .background(isActive ? Color.accentColor.opacity(0.15) : Color.primary.opacity(0.06), in: Capsule())
                }
                .buttonStyle(.plain)
                .help(preset.revset)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(12)
    }

    @ViewBuilder
    private func section(_ title: String, _ sectionRows: [RevsetSuggestion]) -> some View {
        if !sectionRows.isEmpty {
            sectionTitle(title)
            ForEach(sectionRows, id: \.id) { row in
                rowView(row)
            }
        }
    }

    private func sectionTitle(_ title: String) -> some View {
        Text(title)
            .jayjayFont(11, weight: .semibold)
            .foregroundStyle(.secondary)
            .padding(.horizontal, 14)
            .frame(height: Self.sectionTitleHeight, alignment: .bottomLeading)
    }

    private func rowView(_ row: RevsetSuggestion) -> some View {
        let isSelected = selectedRow?.id == row.id
        return Button {
            activate(row)
        } label: {
            HStack(spacing: 8) {
                if let icon = row.icon {
                    Image(systemName: icon)
                        .imageScale(.small)
                        .foregroundStyle(.secondary)
                        .frame(width: 14)
                }
                if row.kind == .current {
                    ViewThatFits(in: .horizontal) {
                        Text(row.title).jayjayFont(11, design: .monospaced).lineLimit(1).fixedSize()
                        Text(row.title).jayjayFont(11, design: .monospaced).lineLimit(3)
                    }
                } else {
                    Text(row.title)
                        .jayjayFont(12, design: .monospaced)
                        .lineLimit(1)
                        .truncationMode(.middle)
                }
                Spacer(minLength: 0)
            }
            .padding(.leading, 14)
            .padding(.trailing, row.kind == .recent ? 34 : 14)
            .padding(.vertical, 6)
            .frame(minHeight: Self.rowHeight)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .accessibilityIdentifier(AID.Picker.row(row.id))
        .overlay(alignment: .trailing) {
            if row.kind == .recent {
                Button {
                    forget(row)
                } label: {
                    Image(systemName: "xmark")
                        .imageScale(.small)
                        .foregroundStyle(.tertiary)
                        .frame(width: 22, height: 22)
                        .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                .padding(.trailing, 10)
                .help(Localized.Repo.removeFromRecent)
                .accessibilityLabel(Localized.Repo.removeTitleFromRecent(title: row.title))
            }
        }
        .background(
            RoundedRectangle(cornerRadius: 6)
                .fill(isSelected ? Color.accentColor.opacity(0.15) : .clear)
                .padding(.horizontal, 6)
        )
    }

    private var footer: some View {
        HStack(spacing: 14) {
            Text(Localized.Repo.returnToApply)
            Text(Localized.Repo.escToCancel)
            Spacer()
            Link(Localized.Repo.revsetLanguage, destination: AppMetadata.revsetDocsURL)
        }
        .jayjayFont(11)
        .foregroundStyle(.tertiary)
        .padding(.horizontal, 14)
        .frame(height: 30)
    }

    private var selectedRow: RevsetSuggestion? {
        guard let index = selectedIndex.map({ $0 - completions.count }), rows.indices.contains(index) else {
            return nil
        }
        return rows[index]
    }

    /// A bookmark or workspace typed as the whole query is left to the panel's own rows, which match on the whole query.
    private func caretMoved(text: String, caret: Int) {
        let wholeQuery = UInt32(text.utf16.count)
        completions = revsetCompletions(text: text, cursor: UInt32(caret), vocabulary: vocabulary)
            .filter { ($0.kind != .bookmark && $0.kind != .workspace) || $0.len != wholeQuery }
    }

    private func accept(_ completion: RevsetCompletion) {
        edit = completion.edit(id: (edit?.id ?? 0) + 1)
        selectedIndex = nil
    }

    private func move(_ delta: Int) -> Bool {
        let count = completions.count + rows.count
        guard count > 0 else { return true }
        let current = selectedIndex ?? (delta > 0 ? -1 : 0)
        selectedIndex = max(0, min(count - 1, current + delta))
        return true
    }

    private func complete() -> Bool {
        if let selectedIndex, completions.indices.contains(selectedIndex) {
            accept(completions[selectedIndex])
        } else if let selectedRow {
            activate(selectedRow)
        } else if let first = completions.first {
            accept(first)
        } else {
            return false
        }
        return true
    }

    private func submit() {
        if let selectedIndex, completions.indices.contains(selectedIndex) {
            accept(completions[selectedIndex])
            return
        }
        if let selectedRow {
            activate(selectedRow)
            return
        }
        guard !trimmedQuery.isEmpty else {
            onDismiss()
            return
        }
        if let message = actions.applyTyped(trimmedQuery, bookmarks: bookmarks) {
            error = message
        } else {
            onDismiss()
        }
    }

    private func activate(_ row: RevsetSuggestion) {
        if row.kind == .current {
            query = row.revset
            focusRequest += 1
        } else {
            apply(row.revset)
        }
    }

    private func forget(_ row: RevsetSuggestion) {
        selectedIndex = nil
        actions.forgetRecentRevset(row.revset)
    }

    private func apply(_ revset: String) {
        onDismiss()
        actions.applyFilter(revset)
    }
}

private extension RevsetSuggestion {
    var id: String {
        "\(kind)-\(title)"
    }

    var icon: String? {
        switch kind {
            case .current: nil
            case .bookmark: "bookmark"
            case .workspace: "folder"
            case .recent: "clock"
        }
    }
}
