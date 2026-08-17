import JayJayCore
import SwiftUI

struct PickerPanelRoot: View {
    static let width: CGFloat = 360
    private static let headerHeight: CGFloat = 45
    private static let sectionTitleHeight: CGFloat = 25

    let placeholder: String
    let actionLabel: String?
    let onAction: (() -> Void)?
    var onRefresh: (() -> Void)?
    let sections: [PickerSection]
    var emptyText = "No matches"
    let onDismiss: () -> Void

    @State private var query = ""
    @State private var selectedIndex: Int?
    @State private var hoveredID: String?
    @FocusState private var isSearchFocused: Bool

    /// Sized before SwiftUI lays out, so rows declare their heights.
    static func idealSize(sections: [PickerSection], width: CGFloat = Self.width) -> NSSize {
        let rows = sections.flatMap(\.rows)
        let content = rows.reduce(0) { $0 + $1.height }
            + CGFloat(sections.filter { $0.title != nil }.count) * sectionTitleHeight
            + headerHeight + 14
        return NSSize(width: width, height: min(max(content, 120), 480))
    }

    /// Section order is kept: a picker is browsed as much as searched.
    private var filteredSections: [PickerSection] {
        let trimmed = query.trimmingCharacters(in: .whitespaces)
        guard !trimmed.isEmpty else { return sections }
        return sections.compactMap { section in
            let matches = Set(fuzzyRank(query: trimmed, candidates: section.rows.map(\.searchText)))
            let rows = section.rows.enumerated().filter { matches.contains(UInt32($0.offset)) }.map(\.element)
            return rows.isEmpty ? nil : PickerSection(id: section.id, title: section.title, rows: rows)
        }
    }

    private var activatableRows: [PickerRow] {
        filteredSections.flatMap(\.rows).filter { $0.action != nil }
    }

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            rowList
        }
        .glassEffect(in: RoundedRectangle(cornerRadius: 12))
        .clipShape(RoundedRectangle(cornerRadius: 12))
        .paletteKeyNavigation(onMove: move, onEscape: onDismiss)
        .onAppear { isSearchFocused = true }
        .onChange(of: query) {
            // Preselect only while filtering.
            selectedIndex = query.trimmingCharacters(in: .whitespaces).isEmpty ? nil : 0
            hoveredID = nil
        }
    }

    private var header: some View {
        HStack(spacing: 8) {
            Image(systemName: "magnifyingglass")
                .foregroundStyle(.secondary)
                .frame(width: 14)
            TextField(placeholder, text: $query)
                .textFieldStyle(.plain)
                .font(.system(size: 13))
                .focused($isSearchFocused)
                .onSubmit(activateSelection)
            if let onRefresh {
                Button(action: onRefresh) {
                    Label("Refresh", systemImage: "arrow.triangle.2.circlepath")
                }
                .buttonStyle(.bordered)
                .controlSize(.small)
                .accessibilityIdentifier(AID.Picker.refresh)
            }
            if let actionLabel, let onAction {
                Button {
                    onDismiss()
                    onAction()
                } label: {
                    Label(actionLabel, systemImage: "plus")
                }
                .buttonStyle(.bordered)
                .controlSize(.small)
            }
        }
        .padding(.horizontal, 12)
        .frame(height: Self.headerHeight)
    }

    private var rowList: some View {
        ScrollViewReader { proxy in
            ScrollView(.vertical, showsIndicators: true) {
                LazyVStack(alignment: .leading, spacing: 0) {
                    ForEach(filteredSections) { section in
                        if let title = section.title {
                            Text(title)
                                .font(.system(size: 11, weight: .semibold))
                                .foregroundStyle(.secondary)
                                .padding(.horizontal, 14)
                                .padding(.top, 8)
                                .padding(.bottom, 3)
                        }
                        ForEach(section.rows) { row in
                            rowView(row)
                        }
                    }
                    if filteredSections.isEmpty {
                        Text(sections.isEmpty ? emptyText : "No matches")
                            .font(.system(size: 12))
                            .foregroundStyle(.secondary)
                            .frame(maxWidth: .infinity)
                            .padding(.vertical, 18)
                    }
                }
                .padding(.vertical, 4)
            }
            .onChange(of: selectedIndex) { _, index in
                guard let index, activatableRows.indices.contains(index) else { return }
                proxy.scrollTo(activatableRows[index].id, anchor: .center)
            }
        }
    }

    private func rowView(_ row: PickerRow) -> some View {
        let selected = isSelected(row)
        let highlighted = selected || hoveredID == row.id
        let identifier = AID.Picker.row(row.id)
        return Group {
            if let action = row.action {
                Button {
                    onDismiss()
                    action()
                } label: {
                    row.content(highlighted)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                .accessibilityIdentifier(identifier)
            } else {
                row.content(highlighted)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .accessibilityIdentifier(identifier)
            }
        }
        .frame(height: row.height)
        .onHover { hovering in
            if hovering {
                hoveredID = row.id
            } else if hoveredID == row.id {
                hoveredID = nil
            }
        }
        .background(
            RoundedRectangle(cornerRadius: 6)
                .fill(selected ? Color.accentColor.opacity(0.15) : (highlighted ? Color.primary.opacity(0.08) : .clear))
                .padding(.horizontal, 6)
        )
        .modifier(RowContextMenu(builder: row.contextMenu))
        .id(row.id)
    }

    private func isSelected(_ row: PickerRow) -> Bool {
        guard let selectedIndex, activatableRows.indices.contains(selectedIndex) else { return false }
        return activatableRows[selectedIndex].id == row.id
    }

    private func move(_ delta: Int) {
        guard !activatableRows.isEmpty else { return }
        let current = selectedIndex ?? (delta > 0 ? -1 : 0)
        selectedIndex = max(0, min(activatableRows.count - 1, current + delta))
        hoveredID = nil
    }

    private func activateSelection() {
        guard let index = selectedIndex, activatableRows.indices.contains(index), let action = activatableRows[index].action else { return }
        onDismiss()
        action()
    }
}

private struct RowContextMenu: ViewModifier {
    let builder: (() -> AnyView)?

    func body(content: Content) -> some View {
        if let builder {
            content.contextMenu { builder() }
        } else {
            content
        }
    }
}
