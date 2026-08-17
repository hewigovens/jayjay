import AppKit
import JayJayCore
import SwiftUI

struct PaletteRoot: View {
    let items: [CommandPaletteItem]
    let runJjCommand: (String) async throws -> JjCommandResult
    let onJjCommandFinished: (JjCommandResult) -> Void
    let onDismiss: () -> Void

    @State var query = ""
    @State var selectedIndex = 0
    @State var hoveredIndex: Int?
    @State var jjResult: JjCommandResult?
    @State var jjError: String?
    @State var isRunning = false
    @State var history: [String] = []
    @State var historyIndex: Int?
    @State var isRecallingHistory = false
    @FocusState private var isSearchFocused: Bool

    private var filtered: [CommandPaletteItem] {
        guard !isJJ else { return [] }
        return CommandPaletteSearch.rank(query: query, items: items)
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 8) {
                Image(systemName: isJJ ? "terminal" : "magnifyingglass")
                    .foregroundStyle(.secondary)
                    .frame(width: 16)
                TextField("Search commands, type `jj status`, or use `!status`", text: $query)
                    .textFieldStyle(.plain)
                    .font(.system(size: 14))
                    .focused($isSearchFocused)
                    .accessibilityIdentifier(AID.Palette.textField)
                    .onSubmit { execute() }
            }
            .padding(12)

            Divider()

            resultArea
        }
        .frame(width: 520, height: 360)
        .glassEffect(in: RoundedRectangle(cornerRadius: 16))
        .clipShape(RoundedRectangle(cornerRadius: 16))
        .onKeyPress(.upArrow) {
            if isJJ {
                recallHistory(older: true)
            } else {
                move(-1)
            }
            return .handled
        }
        .onKeyPress(.downArrow) {
            if isJJ {
                recallHistory(older: false)
            } else {
                move(1)
            }
            return .handled
        }
        .onKeyPress { press in
            if press.modifiers.contains(.control) {
                if press.characters == "p" {
                    move(-1)
                    return .handled
                }
                if press.characters == "n" {
                    move(1)
                    return .handled
                }
            }
            return .ignored
        }
        .onKeyPress(.escape) { onDismiss()
            return .handled
        }
        .onAppear {
            isSearchFocused = true
        }
        .onChange(of: query) { selectedIndex = 0
            hoveredIndex = nil
            jjResult = nil
            jjError = nil
            if isRecallingHistory {
                isRecallingHistory = false
            } else {
                historyIndex = nil
            }
            isSearchFocused = true
        }
    }

    @ViewBuilder
    private var resultArea: some View {
        if isJJ {
            jjSection
        } else {
            ScrollViewReader { proxy in
                let visible = filtered
                if visible.isEmpty, shouldOfferFeatureRequest {
                    emptyHelpResult
                } else {
                    ScrollView {
                        LazyVStack(spacing: 0) {
                            ForEach(Array(visible.enumerated()), id: \.element.id) { index, item in
                                Button {
                                    if let action = item.action {
                                        action()
                                        onDismiss()
                                    }
                                } label: {
                                    HStack(spacing: 10) {
                                        Image(systemName: item.icon)
                                            .frame(width: 18)
                                            .foregroundStyle(item.isInfo ? .tertiary : .secondary)
                                        VStack(alignment: .leading, spacing: 1) {
                                            Text(item.title).font(.system(size: 13))
                                            Text(item.detail ?? item.category)
                                                .font(.system(size: 10))
                                                .foregroundStyle(.tertiary)
                                                .lineLimit(1)
                                        }
                                        Spacer()
                                        if let shortcut = item.shortcut {
                                            Text(shortcut)
                                                .font(.system(size: 11, weight: .medium))
                                                .foregroundStyle(.secondary)
                                                .padding(.horizontal, 6)
                                                .padding(.vertical, 2)
                                                .background(
                                                    Color.secondary.opacity(0.12),
                                                    in: RoundedRectangle(cornerRadius: 5)
                                                )
                                        }
                                    }
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                    .padding(.horizontal, 12)
                                    .padding(.vertical, 5)
                                    .contentShape(Rectangle())
                                }
                                .buttonStyle(.plain)
                                .accessibilityIdentifier(AID.Palette.item(item.title))
                                // Hover only highlights; it never moves the selection or scrolls (VS Code style).
                                .onHover { hovering in
                                    if hovering {
                                        hoveredIndex = index
                                    } else if hoveredIndex == index {
                                        hoveredIndex = nil
                                    }
                                }
                                .background(
                                    RoundedRectangle(cornerRadius: 6)
                                        .fill(
                                            index == selectedIndex
                                                ? Color.accentColor.opacity(0.15)
                                                : (index == hoveredIndex ? Color.primary.opacity(0.08) : .clear)
                                        )
                                        .padding(.horizontal, 6)
                                )
                                .id(item.id)
                            }
                        }
                    }
                    .onChange(of: selectedIndex) { _, idx in
                        guard visible.indices.contains(idx) else { return }
                        withAnimation { proxy.scrollTo(visible[idx].id, anchor: .center) }
                    }
                }
            }
        }
    }

    private var shouldOfferFeatureRequest: Bool {
        !query.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }

    private var emptyHelpResult: some View {
        VStack(spacing: 12) {
            Spacer()
            Image(systemName: "questionmark.bubble")
                .font(.system(size: 28))
                .foregroundStyle(.secondary)
            Text("No matching JayJay feature")
                .font(.system(size: 13, weight: .semibold))
            Text("Open a prefilled issue if this is something JayJay should support.")
                .font(.system(size: 11))
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
                .frame(maxWidth: 300)
            Button {
                openFeatureRequest()
            } label: {
                Label("Request Feature", systemImage: "exclamationmark.bubble")
            }
            .controlSize(.small)
            Spacer()
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private var jjSection: some View {
        rawJjSection
    }

    private func move(_ delta: Int) {
        guard !filtered.isEmpty else { return }
        selectedIndex = max(0, min(filtered.count - 1, selectedIndex + delta))
    }

    private func execute() {
        if isJJ {
            executeJJ()
        } else if filtered.indices.contains(selectedIndex), let action = filtered[selectedIndex].action {
            action()
            onDismiss()
        } else if filtered.isEmpty, shouldOfferFeatureRequest {
            openFeatureRequest()
        }
    }

    private func openFeatureRequest() {
        NSWorkspace.shared.open(HelpBook.requestFeatureURL(query: query))
        onDismiss()
    }
}
