import AppKit
import JayJayCore
import SwiftUI

struct CommandPaletteItem: Identifiable {
    let id = UUID()
    let title: String
    let icon: String
    let category: String
    let action: () -> Void
}

// MARK: - NSPanel

final class CommandPalettePanel: NSPanel {
    init() {
        super.init(
            contentRect: NSRect(x: 0, y: 0, width: 480, height: 300),
            styleMask: [.titled, .fullSizeContentView, .nonactivatingPanel],
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

    func show(items: [CommandPaletteItem], repoPath: String) {
        // Create a fresh view controller each time (guarantees fresh @State)
        let vc = NSHostingController(rootView: PaletteRoot(
            items: items,
            repoPath: repoPath,
            onDismiss: { [weak self] in self?.dismiss() }
        ))
        contentViewController = vc
        setContentSize(NSSize(width: 480, height: 300))

        // Center on parent window
        let parentFrame = NSApp.windows.first(where: { $0.isKeyWindow && $0 !== self })?.frame
            ?? NSScreen.main?.frame ?? .zero
        let x = parentFrame.midX - 240
        let y = parentFrame.midY + 40
        setFrameOrigin(NSPoint(x: x, y: y))
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

// MARK: - Root view (owns @State, destroyed on dismiss)

private struct PaletteRoot: View {
    let items: [CommandPaletteItem]
    let repoPath: String
    let onDismiss: () -> Void

    @State private var query = ""
    @State private var selectedIndex = 0
    @State private var jjOutput: String?
    @State private var isRunning = false

    private var isJJ: Bool {
        query.hasPrefix("!")
    }

    private var jjCmd: String {
        String(query.dropFirst()).trimmingCharacters(in: .whitespaces)
    }

    private var filtered: [CommandPaletteItem] {
        guard !isJJ else { return [] }
        guard !query.isEmpty else { return items }
        let q = query.lowercased()
        return items.filter { $0.title.lowercased().contains(q) || $0.category.lowercased().contains(q) }
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 8) {
                Image(systemName: isJJ ? "terminal" : "magnifyingglass")
                    .foregroundStyle(.secondary)
                    .frame(width: 16)
                TextField("Type a command or ! for jj CLI...", text: $query)
                    .textFieldStyle(.plain)
                    .font(.system(size: 14))
                    .onSubmit { execute() }
            }
            .padding(12)

            Divider()

            resultArea
        }
        .frame(width: 480, height: 300)
        .background(.regularMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 12))
        .onKeyPress(.upArrow) { move(-1)
            return .handled
        }
        .onKeyPress(.downArrow) { move(1)
            return .handled
        }
        .onKeyPress(.escape) { onDismiss()
            return .handled
        }
        .onChange(of: query) { selectedIndex = 0
            jjOutput = nil
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

    @ViewBuilder
    private var jjSection: some View {
        if let output = jjOutput {
            ScrollView {
                Text(output)
                    .font(.system(size: 11, design: .monospaced))
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(12)
            }
        } else if isRunning {
            ProgressView().controlSize(.small).frame(maxWidth: .infinity, maxHeight: .infinity)
        } else {
            HStack {
                Text("jj \(jjCmd)").font(.system(size: 12, design: .monospaced)).foregroundStyle(.secondary)
                Spacer()
                if !jjCmd.isEmpty { Text("Enter ↵").font(.system(size: 10)).foregroundStyle(.tertiary) }
            }
            .padding(12)
            Spacer()
        }
    }

    private func move(_ delta: Int) {
        guard !filtered.isEmpty else { return }
        selectedIndex = max(0, min(filtered.count - 1, selectedIndex + delta))
    }

    private func execute() {
        if isJJ {
            guard !jjCmd.isEmpty else { return }
            isRunning = true
            let args = jjCmd.components(separatedBy: " ")
            let path = repoPath
            Task.detached {
                let status = checkJjEnvironment()
                let proc = Process()
                proc.executableURL = URL(fileURLWithPath: status.path)
                proc.arguments = args
                proc.currentDirectoryURL = URL(fileURLWithPath: path)
                let pipe = Pipe()
                proc.standardOutput = pipe
                proc.standardError = pipe
                try? proc.run()
                proc.waitUntilExit()
                let data = pipe.fileHandleForReading.readDataToEndOfFile()
                let output = String(data: data, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines) ?? "(no output)"
                await MainActor.run {
                    jjOutput = output
                    isRunning = false
                }
            }
        } else if !filtered.isEmpty, selectedIndex < filtered.count {
            filtered[selectedIndex].action()
            onDismiss()
        }
    }
}
