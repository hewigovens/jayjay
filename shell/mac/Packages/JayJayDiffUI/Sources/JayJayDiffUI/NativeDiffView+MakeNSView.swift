import AppKit
import SwiftUI

public extension NativeDiffView {
    func makeCoordinator() -> NativeDiffContextCoordinator {
        NativeDiffContextCoordinator()
    }

    func makeNSView(context: Context) -> DiffTextContainerView {
        let gutterContainer = NSTextContainer(
            containerSize: NSSize(width: 0, height: CGFloat.greatestFiniteMagnitude)
        )
        gutterContainer.widthTracksTextView = true
        gutterContainer.lineFragmentPadding = 0

        let gutterLayoutManager = DiffLayoutManager()
        gutterLayoutManager.addTextContainer(gutterContainer)

        let gutterStorage = NSTextStorage()
        gutterStorage.addLayoutManager(gutterLayoutManager)

        let gutterScrollView = NSScrollView()
        gutterScrollView.hasVerticalScroller = false
        gutterScrollView.hasHorizontalScroller = false
        gutterScrollView.autohidesScrollers = true
        gutterScrollView.drawsBackground = false

        let gutterTextView = DiffGutterTextView(frame: gutterScrollView.bounds, textContainer: gutterContainer)
        gutterTextView.isEditable = false
        gutterTextView.isSelectable = true
        // The layout manager draws its own inset highlight; the system styling would also recolor the ✓/● marker glyphs to selectedTextColor.
        gutterTextView.selectedTextAttributes = [:]
        gutterTextView.isVerticallyResizable = true
        gutterTextView.isHorizontallyResizable = false
        gutterTextView.autoresizingMask = [.width]
        gutterTextView.textContainerInset = NSSize(width: 8, height: 8)
        gutterTextView.drawsBackground = false
        gutterTextView.identifier = NSUserInterfaceItemIdentifier("diff.gutter")
        gutterTextView.setAccessibilityIdentifier("diff.gutter")
        gutterTextView.minSize = NSSize(width: 0, height: 0)
        gutterTextView.maxSize = NSSize(width: CGFloat.greatestFiniteMagnitude, height: CGFloat.greatestFiniteMagnitude)
        gutterScrollView.documentView = gutterTextView

        let scrollView = NSScrollView()
        scrollView.hasVerticalScroller = true
        scrollView.hasHorizontalScroller = false
        scrollView.autohidesScrollers = true
        scrollView.drawsBackground = false

        let textContainer = NSTextContainer(containerSize: NSSize(
            width: 0,
            height: CGFloat.greatestFiniteMagnitude
        ))
        textContainer.widthTracksTextView = true
        textContainer.lineFragmentPadding = 4
        textContainer.lineBreakMode = .byWordWrapping

        let layoutManager = DiffLayoutManager()
        layoutManager.addTextContainer(textContainer)

        let storage = NSTextStorage()
        storage.addLayoutManager(layoutManager)

        let textView = DiffTextView(frame: scrollView.bounds, textContainer: textContainer)
        textView.isEditable = false
        textView.isSelectable = true
        textView.autoresizingMask = [.width]
        textView.isVerticallyResizable = true
        textView.isHorizontallyResizable = false
        textView.textContainerInset = NSSize(width: 4, height: 8)
        textView.drawsBackground = false
        textView.usesFindBar = true
        textView.isIncrementalSearchingEnabled = true
        textView.identifier = NSUserInterfaceItemIdentifier("diffTextView")
        textView.setAccessibilityIdentifier("diff.text")
        textView.minSize = NSSize(width: 0, height: 0)
        textView.maxSize = NSSize(width: CGFloat.greatestFiniteMagnitude, height: CGFloat.greatestFiniteMagnitude)
        textView.delegate = context.coordinator

        scrollView.documentView = textView
        return DiffTextContainerView(
            gutterScrollView: gutterScrollView,
            gutterTextView: gutterTextView,
            scrollView: scrollView,
            textView: textView
        )
    }
}
