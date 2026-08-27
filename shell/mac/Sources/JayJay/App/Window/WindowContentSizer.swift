import AppKit

enum WindowContentSizer {
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
