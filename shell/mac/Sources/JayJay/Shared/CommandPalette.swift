import JayJayCore
import SwiftUI

struct PaletteRoot: View {
    let items: [CommandPaletteItem]
    let repoPath: String
    let onDismiss: () -> Void

    @State var query = ""
    @State var selectedIndex = 0
    @State var jjResult: JjCommandRun?
    @State var jjError: String?
    @State var isRunning = false
    @State var history = CommandPaletteHistory.load()
    @State var historyIndex: Int?
    @State var isRecallingHistory = false

    var filtered: [CommandPaletteItem] {
        guard !isJJ else { return [] }
        guard !query.isEmpty else { return items }
        return items.filter {
            $0.title.lowercased().contains(query.lowercased())
                || $0.category.lowercased().contains(query.lowercased())
        }
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
                    .accessibilityIdentifier(AID.Palette.textField)
                    .onSubmit { execute() }
            }
            .padding(12)

            Divider()

            resultArea
        }
        .frame(width: 520, height: 360)
        .background(.regularMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 12))
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
                if press.characters == "p" { move(-1)
                    return .handled
                }
                if press.characters == "n" { move(1)
                    return .handled
                }
            }
            return .ignored
        }
        .onKeyPress(.escape) {
            onDismiss()
            return .handled
        }
        .onChange(of: query) {
            selectedIndex = 0
            jjResult = nil
            jjError = nil
            if isRecallingHistory {
                isRecallingHistory = false
            } else {
                historyIndex = nil
            }
        }
    }

    @ViewBuilder
    private var resultArea: some View {
        if isJJ {
            jjSection
        } else {
            ScrollViewReader { proxy in
                List {
                    ForEach(Array(filtered.enumerated()), id: \.element.id) { index, item in
                        Button { item.action()
                            onDismiss()
                        } label: {
                            HStack(spacing: 10) {
                                Image(systemName: item.icon).frame(width: 18).foregroundStyle(.secondary)
                                VStack(alignment: .leading, spacing: 1) {
                                    Text(item.title).font(.system(size: 13))
                                    Text(item.category).font(.system(size: 10)).foregroundStyle(.tertiary)
                                }
                                Spacer()
                            }
                        }
                        .buttonStyle(.plain)
                        .listRowBackground(index == selectedIndex ? Color.accentColor.opacity(0.15) : .clear)
                        .id(index)
                    }
                }
                .listStyle(.plain)
                .scrollContentBackground(.hidden)
                .onChange(of: selectedIndex) { _, idx in
                    withAnimation { proxy.scrollTo(idx, anchor: .center) }
                }
            }
        }
    }

    private func move(_ delta: Int) {
        guard !filtered.isEmpty else { return }
        selectedIndex = max(0, min(filtered.count - 1, selectedIndex + delta))
    }
}
