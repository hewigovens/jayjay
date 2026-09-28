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

    func show<Content: View>(under anchor: NSView, size: NSSize, centered: Bool = false, content: Content) {
        guard let window = anchor.window else { return }
        // Replacing the hosting view while visible blanks the panel for a frame and drops the filter text.
        if isVisible, let host = contentViewController as? NSHostingController<Content> {
            host.rootView = content
        } else {
            contentViewController = NSHostingController(rootView: content)
            appearance = window.appearance ?? NSApp.effectiveAppearance
            attach(to: window)
        }
        setContentSize(size)
        let anchorRect = window.convertToScreen(anchor.convert(anchor.bounds, to: nil))
        var origin = NSPoint(x: centered ? anchorRect.midX - size.width / 2 : anchorRect.minX, y: anchorRect.minY - size.height - 4)
        if let screen = window.screen ?? NSScreen.main {
            origin.x = min(origin.x, screen.visibleFrame.maxX - size.width - 8)
            origin.x = max(origin.x, screen.visibleFrame.minX + 8)
            origin.y = max(origin.y, screen.visibleFrame.minY + 8)
        }
        setFrameOrigin(origin)
        makeKeyAndOrderFront(nil)
    }

    /// Sized to the content's own height, keeping the top edge under the anchor as the content grows or shrinks.
    func show(under anchor: NSView, width: CGFloat, centered: Bool = false, content: some View) {
        let fitted = HeightFittingContent(content: content, width: width) { [weak self] height in
            self?.fitHeight(height)
        }
        let height = NSHostingView(rootView: fitted).fittingSize.height
        show(under: anchor, size: NSSize(width: width, height: height), centered: centered, content: fitted)
    }

    private func fitHeight(_ height: CGFloat) {
        guard isVisible, abs(frame.height - height) >= 1 else { return }
        setFrame(NSRect(x: frame.minX, y: frame.maxY - height, width: frame.width, height: height), display: true)
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

private struct HeightFittingContent<Content: View>: View {
    let content: Content
    let width: CGFloat
    let onHeightChange: (CGFloat) -> Void

    var body: some View {
        content
            .frame(width: width)
            .fixedSize(horizontal: false, vertical: true)
            .onGeometryChange(for: CGFloat.self) { $0.size.height } action: { onHeightChange($0) }
            .frame(maxHeight: .infinity, alignment: .top)
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
