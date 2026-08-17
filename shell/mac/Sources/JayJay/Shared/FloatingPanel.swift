import AppKit

class FloatingPanel: NSPanel {
    init(contentRect: NSRect = .zero) {
        super.init(
            contentRect: contentRect,
            styleMask: [.nonactivatingPanel, .fullSizeContentView],
            backing: .buffered,
            defer: true
        )
        level = .floating
        isOpaque = false
        backgroundColor = .clear
        // The owning view reuses it; AppKit must not free it when a host window closes it.
        isReleasedWhenClosed = false
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
            dismissOnFocusLoss()
        }
    }

    func dismissOnFocusLoss() {
        dismiss()
    }
}
