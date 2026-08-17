import AppKit
import SwiftUI

/// Anchored dropdown panel shared by toolbar pickers: opens instantly under its anchor with no animation, dismisses on Escape or focus loss, and hosts arbitrary SwiftUI content such as a filterable list.
final class PickerPanel: NSPanel {
    private weak var hostWindow: NSWindow?
    private var hostWindowCloseObserver: NSObjectProtocol?

    init() {
        super.init(
            contentRect: .zero,
            styleMask: [.nonactivatingPanel, .fullSizeContentView],
            backing: .buffered,
            defer: true
        )
        titleVisibility = .hidden
        titlebarAppearsTransparent = true
        level = .floating
        isOpaque = false
        backgroundColor = .clear
        hidesOnDeactivate = true
    }

    func show(under anchor: NSView, size: NSSize, content: some View) {
        contentViewController = NSHostingController(rootView: content)
        appearance = anchor.window?.appearance ?? NSApp.effectiveAppearance
        setContentSize(size)
        if let window = anchor.window {
            attach(to: window)
            let anchorRect = window.convertToScreen(anchor.convert(anchor.bounds, to: nil))
            var origin = NSPoint(x: anchorRect.minX, y: anchorRect.minY - size.height - 4)
            if let screen = window.screen ?? NSScreen.main {
                origin.x = min(origin.x, screen.visibleFrame.maxX - size.width - 8)
                origin.x = max(origin.x, screen.visibleFrame.minX + 8)
                origin.y = max(origin.y, screen.visibleFrame.minY + 8)
            }
            setFrameOrigin(origin)
        }
        makeKeyAndOrderFront(nil)
    }

    func dismiss() {
        dismissedAt = Date()
        orderOut(nil)
        contentViewController = nil
        detachFromHostWindow()
    }

    private func attach(to window: NSWindow) {
        detachFromHostWindow()
        hostWindow = window
        window.addChildWindow(self, ordered: .above)
        hostWindowCloseObserver = NotificationCenter.default.addObserver(
            forName: NSWindow.willCloseNotification,
            object: window,
            queue: .main
        ) { [weak self] _ in
            self?.dismiss()
        }
    }

    private func detachFromHostWindow() {
        if let hostWindowCloseObserver {
            NotificationCenter.default.removeObserver(hostWindowCloseObserver)
            self.hostWindowCloseObserver = nil
        }
        hostWindow?.removeChildWindow(self)
        hostWindow = nil
    }

    private var dismissedAt: Date?

    /// True right after a dismissal. A click on the anchor button closes the panel via resignKey before the button's action runs, so the action must treat that click as a toggle-close instead of reopening.
    var wasJustDismissed: Bool {
        dismissedAt.map { Date().timeIntervalSince($0) < 0.3 } ?? false
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

/// Stable NSView anchor placed behind a toolbar button so the panel opens from the button's frame.
@MainActor
final class PickerAnchor {
    weak var view: NSView?
}

struct PickerAnchorView: NSViewRepresentable {
    let anchor: PickerAnchor

    func makeNSView(context: Context) -> NSView {
        let view = NSView()
        anchor.view = view
        return view
    }

    func updateNSView(_ view: NSView, context: Context) {
        anchor.view = view
    }
}
