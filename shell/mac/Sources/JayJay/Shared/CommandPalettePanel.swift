import AppKit
import SwiftUI

struct CommandPaletteItem: Identifiable {
    let id = UUID()
    let title: String
    let icon: String
    let category: String
    let action: () -> Void
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

    func show(items: [CommandPaletteItem], repoPath: String) {
        let vc = NSHostingController(rootView: PaletteRoot(
            items: items,
            repoPath: repoPath,
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
