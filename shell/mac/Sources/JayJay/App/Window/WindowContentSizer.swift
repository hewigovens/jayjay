import AppKit
import SwiftUI

struct WindowContentSizer: NSViewRepresentable {
    let targetSize: NSSize
    let minimumOnly: Bool

    func makeNSView(context: Context) -> NSView {
        let view = NSView(frame: .zero)
        DispatchQueue.main.async {
            resizeWindow(for: view)
        }
        return view
    }

    func updateNSView(_ nsView: NSView, context: Context) {
        DispatchQueue.main.async {
            resizeWindow(for: nsView)
        }
    }

    private func resizeWindow(for view: NSView) {
        guard let window = view.window else { return }
        let currentSize = window.contentLayoutRect.size
        let visibleFrame = window.screen?.visibleFrame ?? NSScreen.main?.visibleFrame

        if minimumOnly,
           currentSize.width >= targetSize.width,
           currentSize.height >= targetSize.height
        {
            guard let visibleFrame else { return }
            let fittedFrame = Self.fittedFrame(window.frame, within: visibleFrame)
            guard fittedFrame != window.frame else { return }
            window.setFrame(fittedFrame, display: true)
            return
        }

        let sizeChanged =
            abs(currentSize.width - targetSize.width) > 1 ||
            abs(currentSize.height - targetSize.height) > 1
        guard sizeChanged else { return }

        let frameInset = NSSize(
            width: window.frame.width - currentSize.width,
            height: window.frame.height - currentSize.height
        )
        let targetFrameSize = NSSize(
            width: targetSize.width + frameInset.width,
            height: targetSize.height + frameInset.height
        )
        let newOrigin = NSPoint(
            x: window.frame.midX - targetFrameSize.width / 2,
            y: window.frame.midY - targetFrameSize.height / 2
        )
        let newFrame = NSRect(origin: newOrigin, size: targetFrameSize)
        window.setFrame(visibleFrame.map { Self.fittedFrame(newFrame, within: $0) } ?? newFrame, display: true)
    }

    static func fittedFrame(_ frame: NSRect, within visibleFrame: NSRect) -> NSRect {
        let size = NSSize(
            width: min(frame.width, visibleFrame.width),
            height: min(frame.height, visibleFrame.height)
        )
        let origin = NSPoint(
            x: min(max(frame.minX, visibleFrame.minX), visibleFrame.maxX - size.width),
            y: min(max(frame.minY, visibleFrame.minY), visibleFrame.maxY - size.height)
        )
        return NSRect(origin: origin, size: size)
    }
}
