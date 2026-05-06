import AppKit
@testable import JayJayDiffUI
import XCTest

final class DiffLayoutManagerTests: XCTestCase {
    /// No-wrap invariant: every newline-terminated line must map to exactly one
    /// line fragment. If a long line wraps, the gutter (one row per diff line)
    /// falls out of alignment with the content text view — the regression we
    /// fixed by switching `NativeDiffView`'s content container to a no-wrap
    /// layout with horizontal scrolling.
    func test_longLines_doNotWrap_intoExtraFragments() {
        let (manager, storage) = makeNoWrapLayout()
        let shortLine = "fn main() {\n"
        // Much wider than any sensible viewport.
        let longLine = String(repeating: "x", count: 2000) + "\n"
        let finalLine = "}\n"
        storage.append(NSAttributedString(
            string: shortLine + longLine + finalLine,
            attributes: [.font: NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)]
        ))

        manager.ensureLayout(for: manager.textContainers[0])

        // One fragment per newline-terminated line — the long line must not
        // split into multiple fragments from wrapping.
        let fragmentCount = countLineFragments(in: manager)
        XCTAssertEqual(fragmentCount, 3)
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
    func test_rectArray_clampsEachLineToOwnUsedWidth() {
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
        for i in 0..<count {
            let rect = rects![i]
            // Skip zero-width sentinel rects (e.g. the trailing newline glyph).
            guard rect.width > 0 else { continue }
            // Match by midY against line fragments.
            var matchedEol: CGFloat? = nil
            manager.enumerateLineFragments(forGlyphRange: NSRange(location: 0, length: manager.numberOfGlyphs)) {
                lineRect, usedRect, _, _, stop in
                if lineRect.minY <= rect.midY && rect.midY < lineRect.maxY {
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
}
