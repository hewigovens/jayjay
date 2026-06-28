import AppKit

extension NSTextView {
    /// NSTextView scrolls its selection into view when resized (`_setFrameSize:forceScroll:`), which knocks a freshly rendered diff off the top; wrap `super.setFrameSize` in this to pin the scroll position instead (clamped in case the document shrank).
    func pinningClipOrigin(_ resize: () -> Void) {
        guard let clip = enclosingScrollView?.contentView else { return resize() }
        let saved = clip.bounds.origin
        resize()
        var target = saved
        target.y = min(target.y, max(0, frame.height - clip.bounds.height))
        if abs(clip.bounds.origin.y - target.y) > 0.5 || abs(clip.bounds.origin.x - target.x) > 0.5 {
            clip.scroll(to: target)
        }
    }
}
