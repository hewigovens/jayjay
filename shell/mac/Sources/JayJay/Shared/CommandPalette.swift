import AppKit
import JayJayCore
import SwiftUI

struct CommandPaletteItem: Identifiable {
    let id = UUID()
    let title: String
    let icon: String
    let category: String
    let keywords: [String]
    let action: () -> Void

    init(
        title: String,
        icon: String,
        category: String,
        keywords: [String] = [],
        action: @escaping () -> Void
    ) {
        self.title = title
        self.icon = icon
        self.category = category
        self.keywords = keywords
        self.action = action
    }

    func matches(query: String) -> Bool {
        CommandPaletteSearch.matches(
            query: query,
            title: title,
            category: category,
            keywords: keywords
        )
    }
}

final class CommandPalettePanel: NSPanel {
    init() {
        super.init(
            contentRect: NSRect(x: 0, y: 0, width: 480, height: 300),
            styleMask: [.nonactivatingPanel, .fullSizeContentView],
            backing: .buffered,
            defer: true
        )
        titleVisibility = .hidden
        titlebarAppearsTransparent = true
        isMovableByWindowBackground = true
        level = .floating
        isOpaque = false
        backgroundColor = .clear
        hidesOnDeactivate = true
    }

    func show(
        items: [CommandPaletteItem],
        repoPath: String,
        onJjCommandFinished: @escaping (JjCommandResult) -> Void = { _ in }
    ) {
        let vc = NSHostingController(rootView: PaletteRoot(
            items: items,
            repoPath: repoPath,
            onJjCommandFinished: onJjCommandFinished,
            onDismiss: { [weak self] in self?.dismiss() }
        ))
        contentViewController = vc
        if let parentAppearance = NSApp.windows.first(where: { $0.isKeyWindow && $0 !== self })?.appearance {
            appearance = parentAppearance
        } else {
            appearance = NSApp.effectiveAppearance
        }
        setContentSize(NSSize(width: 520, height: 360))

        let parentFrame = NSApp.windows.first(where: { $0.isKeyWindow && $0 !== self })?.frame
            ?? NSScreen.main?.frame ?? .zero
        setFrameOrigin(NSPoint(x: parentFrame.midX - 260, y: parentFrame.midY + 40))
        makeKeyAndOrderFront(nil)
    }

    func dismiss() {
        orderOut(nil)
        contentViewController = nil
    }

    override func cancelOperation(_ sender: Any?) {
        dismiss()
    }

    override var canBecomeKey: Bool {
        true
    }
}

struct PaletteRoot: View {
    let items: [CommandPaletteItem]
    let repoPath: String
    let onJjCommandFinished: (JjCommandResult) -> Void
    let onDismiss: () -> Void

    @State var query = ""
    @State var selectedIndex = 0
    @State var jjResult: JjCommandResult?
    @State var jjError: String?
    @State var isRunning = false
    @State var history: [String] = []
    @State var historyIndex: Int?
    @State var isRecallingHistory = false

    private var filtered: [CommandPaletteItem] {
        guard !isJJ else { return [] }
        guard !query.isEmpty else { return items }
        return items.filter { $0.matches(query: query) }
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
        .onKeyPress(.escape) { onDismiss()
            return .handled
        }
        .onChange(of: query) { selectedIndex = 0
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
                let visible = filtered
                ScrollView {
                    LazyVStack(spacing: 0) {
                        ForEach(Array(visible.enumerated()), id: \.element.id) { index, item in
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
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .padding(.horizontal, 12)
                                .padding(.vertical, 6)
                                .contentShape(Rectangle())
                            }
                            .buttonStyle(.plain)
                            .onHover { hovering in
                                if hovering, selectedIndex != index {
                                    selectedIndex = index
                                }
                            }
                            .background(index == selectedIndex ? Color.accentColor.opacity(0.15) : .clear)
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
        } else if !filtered.isEmpty, selectedIndex < filtered.count {
            filtered[selectedIndex].action()
            onDismiss()
        }
    }
}
