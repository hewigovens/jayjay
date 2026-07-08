import AppKit
@testable import JayJayDiffUI
import XCTest

final class DiffLayoutManagerTests: XCTestCase {
    /// Wrapped content must report how many visual rows each logical diff line
    /// occupies so the gutter can insert blank continuation rows.
    func test_longLinesWrap_intoExtraFragments() {
        let (manager, storage) = makeWrappingLayout(width: 120)
        let shortLine = "fn main() {\n"
        // Much wider than any sensible viewport.
        let longLine = String(repeating: "wrapped ", count: 80) + "\n"
        let finalLine = "}\n"
        storage.append(NSAttributedString(
            string: shortLine + longLine + finalLine,
            attributes: [.font: NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)]
        ))

        manager.ensureLayout(for: manager.textContainers[0])

        let fragmentCount = countLineFragments(in: manager)
        XCTAssertGreaterThan(fragmentCount, 3)
        let visualLineCounts = manager.visualLineCounts(logicalLineCount: 3)
        XCTAssertEqual(visualLineCounts[0], 1)
        XCTAssertGreaterThan(visualLineCounts[1], 1)
        XCTAssertEqual(visualLineCounts[2], 1)
    }

    private func countLineFragments(in manager: NSLayoutManager) -> Int {
        var count = 0
        let glyphRange = NSRange(location: 0, length: manager.numberOfGlyphs)
        manager.enumerateLineFragments(forGlyphRange: glyphRange) { _, _, _, _, _ in
            count += 1
        }
        return count
    }

    /// Background-fill regression: when `containerSize.width` is
    /// `.greatestFiniteMagnitude` (no-wrap layout), the old code filled each
    /// row background with a rect of that width, which rendered incorrectly
    /// under clipping. `lineBackgroundFillWidth` must be finite and at least
    /// as wide as the widest laid-out line.
    func test_lineBackgroundFillWidth_isFiniteForInfiniteContainer() {
        let (manager, storage) = makeNoWrapLayout()
        let longLine = String(repeating: "x", count: 500)
        storage.append(NSAttributedString(
            string: longLine + "\n",
            attributes: [.font: NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)]
        ))

        let width = manager.lineBackgroundFillWidth
        XCTAssertTrue(width.isFinite, "draw width must be finite, got \(width)")
        XCTAssertGreaterThan(width, 0)
        // Sanity: 500 glyphs of a monospace 12pt font should push well past
        // a few hundred points. If this ever fails, layout produced nothing.
        XCTAssertGreaterThan(width, 300)
    }

    /// Multi-line selection across a short→long line pair must clamp each
    /// rect to its OWN line's used width. The bug we hit: probing the rect's
    /// leading edge with glyphIndex(for:) returned the previous (shorter)
    /// line's tail glyph, so the long line got mis-clamped to the short
    /// line's EOL — selecting up into `pub(super) enum CommandOutput {`
    /// after `#[derive(Clone)]` only highlighted `pub(super)`.
    func test_rectArray_clampsEachLineToOwnUsedWidth() throws {
        let (manager, storage) = makeNoWrapLayout()
        let shortLine = "ab\n"
        let longLine = "abcdefghijklmnop\n"
        storage.append(NSAttributedString(
            string: shortLine + longLine,
            attributes: [.font: NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)]
        ))
        manager.ensureLayout(for: manager.textContainers[0])

        // Select from start-of-document (short line, col 0) through the
        // entire long line — covers both lines fully.
        let totalLen = (shortLine + longLine).utf16.count
        let charRange = NSRange(location: 0, length: totalLen)
        var count = 0
        let rects = withUnsafeMutablePointer(to: &count) { rectCountPtr in
            manager.rectArray(
                forCharacterRange: charRange,
                withinSelectedCharacterRange: charRange,
                in: manager.textContainers[0],
                rectCount: rectCountPtr
            )
        }
        XCTAssertNotNil(rects, "expected at least one rect")
        XCTAssertGreaterThanOrEqual(count, 2, "expected one rect per line")

        // Compute each line's used width directly so we can compare.
        var lineWidths: [CGFloat] = []
        manager.enumerateLineFragments(forGlyphRange: NSRange(location: 0, length: manager.numberOfGlyphs)) {
            _, usedRect, _, _, _ in
            lineWidths.append(NSMaxX(usedRect))
        }
        XCTAssertEqual(lineWidths.count, 2)
        let shortEol = lineWidths[0]
        let longEol = lineWidths[1]
        XCTAssertGreaterThan(longEol, shortEol, "long line should be wider than short")

        // Find each rect's matching line by Y, assert width == that line's eol.
        for i in 0 ..< count {
            let rect = try XCTUnwrap(rects?[i])
            // Skip zero-width sentinel rects (e.g. the trailing newline glyph).
            guard rect.width > 0 else { continue }
            // Match by midY against line fragments.
            var matchedEol: CGFloat? = nil
            manager.enumerateLineFragments(forGlyphRange: NSRange(location: 0, length: manager.numberOfGlyphs)) {
                lineRect, usedRect, _, _, stop in
                if lineRect.minY <= rect.midY, rect.midY < lineRect.maxY {
                    matchedEol = NSMaxX(usedRect)
                    stop.pointee = true
                }
            }
            guard let expectedEol = matchedEol else { continue }
            XCTAssertEqual(
                NSMaxX(rect), expectedEol, accuracy: 0.5,
                "rect \(i) at y=\(rect.midY) should clamp to its own line's EOL \(expectedEol), got \(NSMaxX(rect))"
            )
        }
    }

    /// Empty storage must still produce a sane (non-NaN, non-infinite) width.
    func test_lineBackgroundFillWidth_empty_isZero() {
        let (manager, _) = makeNoWrapLayout()
        let width = manager.lineBackgroundFillWidth
        XCTAssertTrue(width.isFinite)
        XCTAssertEqual(width, 0, accuracy: 0.001)
    }

    func test_rectArray_includesSingleFindMatch() throws {
        let (manager, storage) = makeNoWrapLayout()
        let prefix = "let stdout = "
        let match = "trim_output"
        storage.append(NSAttributedString(
            string: "\(prefix)\(match)(&output.stdout);\n",
            attributes: [.font: NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)]
        ))
        manager.ensureLayout(for: manager.textContainers[0])

        let charRange = NSRange(location: prefix.utf16.count, length: match.utf16.count)
        var count = 0
        let rects = withUnsafeMutablePointer(to: &count) { rectCountPtr in
            manager.rectArray(
                forCharacterRange: charRange,
                withinSelectedCharacterRange: charRange,
                in: manager.textContainers[0],
                rectCount: rectCountPtr
            )
        }

        XCTAssertNotNil(rects)
        XCTAssertEqual(count, 1)
        let rect = try XCTUnwrap(rects?[0])
        XCTAssertGreaterThan(rect.width, 20)
        XCTAssertGreaterThan(rect.height, 8)
    }

    func test_wordHighlightRects_onlyCollectVisibleHighlights() {
        let (manager, storage) = makeNoWrapLayout()
        let font = NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)
        let text = NSMutableAttributedString()
        for index in 0 ..< 400 {
            let line = "line \(index) target-\(index) end\n"
            let start = text.length
            text.append(NSAttributedString(string: line, attributes: [.font: font]))
            let targetRange = (line as NSString).range(of: "target-\(index)")
            text.addAttribute(
                .diffWordHighlightColor,
                value: NSColor.systemRed,
                range: NSRange(location: start + targetRange.location, length: targetRange.length)
            )
        }
        storage.setAttributedString(text)
        manager.ensureLayout(for: manager.textContainers[0])

        let visibleRange = (storage.string as NSString).range(of: "target-200")
        let visibleGlyphRange = manager.glyphRange(forCharacterRange: visibleRange, actualCharacterRange: nil)

        let rects = manager.wordHighlightRects(visibleGlyphRange: visibleGlyphRange, in: manager.textContainers[0])

        XCTAssertEqual(rects.count, 1)
        XCTAssertGreaterThan(rects[0].rect.width, 20)
    }

    func test_findMatchRanges_returnsAllVisibleMatches() {
        let (manager, storage) = makeNoWrapLayout()
        let text = """
        let stdout = trim_output(&output.stdout);
        let stderr = trim_output(&output.stderr);
        let other = no_match();
        """
        storage.append(NSAttributedString(
            string: text,
            attributes: [.font: NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)]
        ))
        manager.ensureLayout(for: manager.textContainers[0])

        let ranges = manager.findMatchRanges("trim_output")

        XCTAssertEqual(ranges.count, 2)
        XCTAssertEqual((storage.string as NSString).substring(with: ranges[0]), "trim_output")
        XCTAssertEqual((storage.string as NSString).substring(with: ranges[1]), "trim_output")
    }

    func test_diffTextView_usesFindSelectionColorOnlyForCurrentFindMatch() {
        let (textView, _) = makeScrollableDiffTextView()
        textView.textStorage?.setAttributedString(NSAttributedString(
            string: "let stdout = trim_output(&output.stdout);\nlet other = no_match();\n",
            attributes: [.font: NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)]
        ))
        let theme = DiffColors(isDark: false)
        textView.configureFindSelectionColors(theme)

        NSPasteboard(name: .find).clearContents()
        NSPasteboard(name: .find).setString("trim_output", forType: .string)
        textView.showsFindHighlights = true
        textView.setSelectedRange(NSRange(location: "let stdout = ".utf16.count, length: "trim_output".utf16.count))

        XCTAssertEqual(
            textView.selectedTextAttributes[.backgroundColor] as? NSColor,
            theme.findCurrentMatchBg
        )

        textView.setSelectedRange(NSRange(location: 0, length: "let".utf16.count))
        XCTAssertEqual(
            textView.selectedTextAttributes[.backgroundColor] as? NSColor,
            NSColor.selectedTextBackgroundColor
        )
    }

    func test_diffTextView_scrollsCurrentFindSelectionToVisible() throws {
        let (textView, scrollView) = makeScrollableDiffTextView(height: 80)
        let prefix = String(repeating: "context\n", count: 120)
        let match = "trim_output"
        let text = prefix + match + "\n"
        textView.textStorage?.setAttributedString(NSAttributedString(
            string: text,
            attributes: [.font: NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)]
        ))
        try textView.layoutManager?.ensureLayout(for: XCTUnwrap(textView.textContainer))
        let usedRect = try textView.layoutManager?.usedRect(for: XCTUnwrap(textView.textContainer)) ?? .zero
        textView.frame.size.height = max(usedRect.height, scrollView.contentSize.height)

        NSPasteboard(name: .find).clearContents()
        NSPasteboard(name: .find).setString(match, forType: .string)
        textView.showsFindHighlights = true
        textView.setSelectedRange(NSRange(location: prefix.utf16.count, length: match.utf16.count))
        scrollView.contentView.scroll(to: .zero)
        scrollView.reflectScrolledClipView(scrollView.contentView)

        textView.scrollCurrentFindSelectionToVisible()

        XCTAssertGreaterThan(scrollView.contentView.bounds.origin.y, 0)
    }

    func test_diffTextView_debouncesFindTypingAndSelectsFirstMatch() {
        let (textView, _) = makeScrollableDiffTextView()
        let prefix = "let stdout = "
        let match = "trim_output"
        textView.textStorage?.setAttributedString(NSAttributedString(
            string: "\(prefix)\(match)(&output.stdout);\n",
            attributes: [.font: NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)]
        ))
        textView.setSelectedRange(NSRange(location: 0, length: 0))

        NSPasteboard(name: .find).clearContents()
        NSPasteboard(name: .find).setString(match, forType: .string)
        textView.performFindPanelAction(findMenuItem(.setFindString))
        RunLoop.current.run(until: Date().addingTimeInterval(0.25))

        XCTAssertEqual(textView.activeFindQuery, match)
        XCTAssertEqual(
            textView.selectedRanges.first?.rangeValue,
            NSRange(location: prefix.utf16.count, length: match.utf16.count)
        )
    }

    func test_diffTextView_findNavigationKeepsNativePaneScope() {
        let (left, _) = makeScrollableDiffTextView()
        let (right, _) = makeScrollableDiffTextView()
        left.findPartner = right
        right.findPartner = left

        let prefix = "right side "
        let match = "trim_output"
        left.textStorage?.setAttributedString(NSAttributedString(
            string: "left side has no match\n",
            attributes: [.font: NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)]
        ))
        right.textStorage?.setAttributedString(NSAttributedString(
            string: "\(prefix)\(match)\n",
            attributes: [.font: NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)]
        ))

        NSPasteboard(name: .find).clearContents()
        NSPasteboard(name: .find).setString(match, forType: .string)
        left.performFindPanelAction(findMenuItem(.next))

        XCTAssertEqual(left.activeFindQuery, match)
        XCTAssertEqual(right.activeFindQuery, match)
        XCTAssertTrue(right.showsFindHighlights)
        XCTAssertNotEqual(right.selectedRanges.first?.rangeValue.length, match.utf16.count)
    }

    func test_diffTextView_clearsHighlightsWhenFindBarCloses() {
        let (textView, scrollView) = makeScrollableDiffTextView()
        let match = "trim_output"
        textView.textStorage?.setAttributedString(NSAttributedString(
            string: "\(match)\n",
            attributes: [.font: NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)]
        ))

        NSPasteboard(name: .find).clearContents()
        NSPasteboard(name: .find).setString(match, forType: .string)
        scrollView.isFindBarVisible = true
        textView.performFindPanelAction(findMenuItem(.setFindString))
        XCTAssertTrue(textView.showsFindHighlights)

        scrollView.isFindBarVisible = false
        textView.syncFindBarVisibility()

        XCTAssertFalse(textView.showsFindHighlights)
        XCTAssertNil(textView.activeFindQuery)
    }

    // MARK: - Fixtures

    private func makeNoWrapLayout() -> (DiffLayoutManager, NSTextStorage) {
        let container = NSTextContainer(containerSize: NSSize(
            width: CGFloat.greatestFiniteMagnitude,
            height: CGFloat.greatestFiniteMagnitude
        ))
        container.widthTracksTextView = false
        container.lineFragmentPadding = 4

        let manager = DiffLayoutManager()
        manager.addTextContainer(container)

        let storage = NSTextStorage()
        storage.addLayoutManager(manager)
        return (manager, storage)
    }

    private func findMenuItem(_ action: NSFindPanelAction) -> NSMenuItem {
        let item = NSMenuItem()
        item.tag = Int(action.rawValue)
        return item
    }

    private func makeScrollableDiffTextView(
        width: CGFloat = 240,
        height: CGFloat = 160
    ) -> (DiffTextView, NSScrollView) {
        let scrollView = NSScrollView(frame: NSRect(x: 0, y: 0, width: width, height: height))
        scrollView.hasVerticalScroller = true
        scrollView.hasHorizontalScroller = false

        let container = NSTextContainer(containerSize: NSSize(width: width, height: CGFloat.greatestFiniteMagnitude))
        container.widthTracksTextView = true
        container.lineFragmentPadding = 4

        let manager = DiffLayoutManager()
        manager.addTextContainer(container)

        let storage = NSTextStorage()
        storage.addLayoutManager(manager)

        let textView = DiffTextView(frame: NSRect(x: 0, y: 0, width: width, height: height), textContainer: container)
        textView.textContainerInset = .zero
        textView.isVerticallyResizable = true
        textView.isHorizontallyResizable = false
        scrollView.documentView = textView

        return (textView, scrollView)
    }

    private func makeWrappingLayout(width: CGFloat) -> (DiffLayoutManager, NSTextStorage) {
        let container = NSTextContainer(containerSize: NSSize(
            width: width,
            height: CGFloat.greatestFiniteMagnitude
        ))
        container.widthTracksTextView = false
        container.lineFragmentPadding = 4
        container.lineBreakMode = .byWordWrapping

        let manager = DiffLayoutManager()
        manager.addTextContainer(container)

        let storage = NSTextStorage()
        storage.addLayoutManager(manager)
        return (manager, storage)
    }
}
