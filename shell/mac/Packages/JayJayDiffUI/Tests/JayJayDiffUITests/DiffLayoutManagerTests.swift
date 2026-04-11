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
