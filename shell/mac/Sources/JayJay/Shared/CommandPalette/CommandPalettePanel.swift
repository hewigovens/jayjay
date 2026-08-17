import AppKit
import JayJayCore
import SwiftUI

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
        // Remember the position whenever the user drags the panel.
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(saveFrameOrigin),
            name: NSWindow.didMoveNotification,
            object: self
        )
    }

    static let originKey = "commandPalette.frameOrigin"

    @objc private func saveFrameOrigin() {
        UserDefaults.standard.set(NSStringFromPoint(frame.origin), forKey: Self.originKey)
    }

    func show(
        items: [CommandPaletteItem],
        runJjCommand: @escaping (String) async throws -> JjCommandResult,
        onJjCommandFinished: @escaping (JjCommandResult) -> Void = { _ in }
    ) {
        let vc = NSHostingController(rootView: PaletteRoot(
            items: items,
            runJjCommand: runJjCommand,
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
        // Restore the last position the user dragged to; only center on first use.
        let saved = UserDefaults.standard.string(forKey: Self.originKey).map(NSPointFromString)
        if let origin = saved, origin != .zero {
            setFrameOrigin(origin)
        } else {
            let parentFrame = NSApp.windows.first(where: { $0.isKeyWindow && $0 !== self })?.frame
                ?? NSScreen.main?.frame ?? .zero
            setFrameOrigin(NSPoint(x: parentFrame.midX - 260, y: parentFrame.midY + 10))
        }
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

    override func resignKey() {
        super.resignKey()
        // Dismiss on focus loss (e.g. a click outside), unless it immediately regains key.
        DispatchQueue.main.async { [weak self] in
            guard let self, !self.isKeyWindow else { return }
            dismiss()
        }
    }
}
