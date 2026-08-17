import AppKit
import SwiftUI

final class PickerPanel: FloatingPanel {
    private weak var hostWindow: NSWindow?
    private var hostWindowCloseObserver: NSObjectProtocol?
    private var focusLossDismissedAt: Date?

    /// The anchor click resigns key first, so the button action must not reopen.
    var wasJustDismissed: Bool {
        focusLossDismissedAt.map { Date().timeIntervalSince($0) < 0.3 } ?? false
    }

    func show(under anchor: NSView, size: NSSize, content: some View) {
        guard let window = anchor.window else { return }
        contentViewController = NSHostingController(rootView: content)
        appearance = window.appearance ?? NSApp.effectiveAppearance
        setContentSize(size)
        attach(to: window)
        let anchorRect = window.convertToScreen(anchor.convert(anchor.bounds, to: nil))
        var origin = NSPoint(x: anchorRect.minX, y: anchorRect.minY - size.height - 4)
        if let screen = window.screen ?? NSScreen.main {
            origin.x = min(origin.x, screen.visibleFrame.maxX - size.width - 8)
            origin.x = max(origin.x, screen.visibleFrame.minX + 8)
            origin.y = max(origin.y, screen.visibleFrame.minY + 8)
        }
        setFrameOrigin(origin)
        makeKeyAndOrderFront(nil)
    }

    override func dismiss() {
        super.dismiss()
        detachFromHostWindow()
    }

    override func dismissOnFocusLoss() {
        focusLossDismissedAt = Date()
        dismiss()
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
}

/// SwiftUI toolbar buttons have no NSView to anchor to.
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
