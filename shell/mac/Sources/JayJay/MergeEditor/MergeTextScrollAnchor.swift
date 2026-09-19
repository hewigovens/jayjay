import AppKit

@MainActor
final class MergeTextScrollAnchor {
    private weak var textView: NSTextView?
    private var observer: NSObjectProtocol?
    private var lineStarts = [0]
    private var text = ""

    init(textView: NSTextView, onScroll: @escaping @MainActor () -> Void) {
        self.textView = textView
        if let scrollView = textView.enclosingScrollView {
            scrollView.contentView.postsBoundsChangedNotifications = true
            observer = NotificationCenter.default.addObserver(
                forName: NSView.boundsDidChangeNotification,
                object: scrollView.contentView,
                queue: .main
            ) { _ in
                MainActor.assumeIsolated { onScroll() }
            }
        }
        updateText()
    }

    deinit {
        if let observer {
            NotificationCenter.default.removeObserver(observer)
        }
    }

    func updateText() {
        guard let textView, text != textView.string else { return }
        text = textView.string
        lineStarts = [0]
        for (index, unit) in text.utf16.enumerated() where unit == 10 {
            lineStarts.append(index + 1)
        }
    }

    var centerLine: Double {
        guard let textView, let scrollView = textView.enclosingScrollView,
              let layout = textView.layoutManager, let container = textView.textContainer,
              !text.isEmpty else { return 0 }
        layout.ensureLayout(for: container)
        let y = max(0, scrollView.contentView.bounds.midY - textView.textContainerInset.height)
        let glyph = layout.glyphIndex(for: NSPoint(x: 0, y: y), in: container)
        let character = layout.characterIndexForGlyph(at: min(glyph, max(0, layout.numberOfGlyphs - 1)))
        let line = max(0, lineStarts.partitioningIndex { $0 > character } - 1)
        let rect = lineRect(line)
        return Double(line) + Double(max(0, min(1, (y - rect.minY) / max(1, rect.height))))
    }

    func scroll(to line: Double) {
        guard let textView, let scrollView = textView.enclosingScrollView else { return }
        let clamped = max(0, min(line, Double(lineStarts.count) - 0.000001))
        let index = Int(clamped)
        let rect = lineRect(index)
        let y = rect.minY + CGFloat(clamped - Double(index)) * rect.height + textView.textContainerInset.height
            - scrollView.contentView.bounds.height / 2
        let maxY = max(0, textView.frame.height - scrollView.contentView.bounds.height)
        scrollView.contentView.scroll(to: NSPoint(x: scrollView.contentView.bounds.minX, y: min(maxY, max(0, y))))
        scrollView.reflectScrolledClipView(scrollView.contentView)
    }

    private func lineRect(_ line: Int) -> NSRect {
        guard let textView, let layout = textView.layoutManager, let container = textView.textContainer else { return .zero }
        layout.ensureLayout(for: container)
        let start = lineStarts[line]
        let end = line + 1 < lineStarts.count ? lineStarts[line + 1] : (text as NSString).length
        if start == end {
            return layout.extraLineFragmentRect
        }
        let glyphs = layout.glyphRange(forCharacterRange: NSRange(location: start, length: end - start), actualCharacterRange: nil)
        return layout.boundingRect(forGlyphRange: glyphs, in: container)
    }
}

private extension [Int] {
    func partitioningIndex(where predicate: (Int) -> Bool) -> Int {
        var low = 0
        var high = count
        while low < high {
            let middle = (low + high) / 2
            if predicate(self[middle]) {
                high = middle
            } else {
                low = middle + 1
            }
        }
        return low
    }
}
