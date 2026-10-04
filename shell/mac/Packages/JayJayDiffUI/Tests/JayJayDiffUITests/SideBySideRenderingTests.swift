import AppKit
import JayJayCore
@testable import JayJayDiffUI
import XCTest

final class SideBySideRenderingTests: XCTestCase {
    func testUnchangedRenderPreservesTextStorageAndSelection() throws {
        let diff = computeFileDiff(path: "sample.rs", oldContent: "let value = 1;\n", newContent: "let value = 2;\n", ignoreWhitespace: false)
        let (coordinator, left, right) = makeRenderer(diff)
        coordinator.applySelectionResetGeneration(0)
        let storage = try XCTUnwrap(left.textView.textStorage)
        XCTAssertTrue(storage.string.contains("let value = 1;"))
        let selected = NSRange(location: 4, length: 5)
        left.textView.setSelectedRange(selected)
        var edits = 0
        let observer = NotificationCenter.default.addObserver(
            forName: NSTextStorage.didProcessEditingNotification, object: storage, queue: nil
        ) { _ in edits += 1 }
        defer { NotificationCenter.default.removeObserver(observer) }

        coordinator.diff = diff
        coordinator.font = .monospacedSystemFont(ofSize: 12, weight: .regular)
        coordinator.theme = DiffColors(isDark: false)
        coordinator.revealFeedback = ContextExpansionReveal(generation: 1, newLines: LineSpan(start: 1, count: 1))
        coordinator.renderIfNeeded()

        XCTAssertEqual(left.pendingRevealFeedback?.feedback.generation, 1)
        XCTAssertEqual(right.pendingRevealFeedback?.feedback.generation, 1)
        XCTAssertEqual(edits, 0)
        XCTAssertEqual(left.textView.selectedRange(), selected)
        coordinator.applySelectionResetGeneration(1)
        coordinator.renderIfNeeded()
        XCTAssertEqual(left.textView.selectedRange(), NSRange(location: 0, length: 0))
        XCTAssertEqual(edits, 0)
    }

    func testContentFontThemeAndWidthChangesUpdateRenderedText() throws {
        let source = "let value = 123; // " + String(repeating: "long comment ", count: 10) + "\n"
        let diff = computeFileDiff(path: "sample.rs", oldContent: source, newContent: source + "new line\n", ignoreWhitespace: false)
        let (coordinator, left, right) = makeRenderer(diff)
        let storage = try XCTUnwrap(left.textView.textStorage)
        let original = NSAttributedString(attributedString: storage)

        coordinator.theme = DiffColors(isDark: true)
        coordinator.renderIfNeeded()
        XCTAssertEqual(storage.string, original.string)
        XCTAssertNotEqual(
            storage.attribute(.foregroundColor, at: 0, effectiveRange: nil) as? NSColor,
            original.attribute(.foregroundColor, at: 0, effectiveRange: nil) as? NSColor
        )

        let font = NSFont.monospacedSystemFont(ofSize: 12, weight: .bold)
        coordinator.font = font
        coordinator.renderIfNeeded()
        XCTAssertEqual(storage.attribute(.font, at: 0, effectiveRange: nil) as? NSFont, font)

        let wideLineCount = storage.string.filter { $0 == "\n" }.count
        right.scrollView.frame.size.width = 200
        coordinator.renderIfNeeded()
        XCTAssertGreaterThan(storage.string.filter { $0 == "\n" }.count, wideLineCount)
        let rightNarrowLineCount = storage.string.filter { $0 == "\n" }.count
        left.scrollView.frame.size.width = 100
        coordinator.renderIfNeeded()
        XCTAssertGreaterThan(storage.string.filter { $0 == "\n" }.count, rightNarrowLineCount)

        coordinator.diff = computeFileDiff(path: "sample.rs", oldContent: "before\n", newContent: "after\n", ignoreWhitespace: false)
        coordinator.renderIfNeeded()
        XCTAssertEqual(storage.string, "before\n")
        XCTAssertEqual(right.textView.string, "after\n")
    }

    func testExpansionLinksFollowAvailabilityAndUseLatestCallback() throws {
        let source = (0 ..< 30).map { "line \($0)\n" }.joined()
        let diff = computeFileDiff(path: "sample.txt", oldContent: source, newContent: source + "added\n", ignoreWhitespace: false)
        let (coordinator, left, right) = makeRenderer(diff)
        let storage = try XCTUnwrap(left.textView.textStorage)
        func link() -> Any? {
            var result: Any?
            storage.enumerateAttribute(.link, in: NSRange(location: 0, length: storage.length)) { value, _, _ in
                if let value {
                    result = value
                }
            }
            return result
        }
        XCTAssertNil(link())
        coordinator.onExpandContext = { _ in XCTFail("stale callback") }
        coordinator.renderIfNeeded()
        let expansionLink = try XCTUnwrap(link())
        var received: ContextExpansionRequest?
        coordinator.onExpandContext = { received = $0 }
        let selected = NSRange(location: 0, length: 1)
        left.textView.setSelectedRange(selected)
        coordinator.renderIfNeeded()
        XCTAssertEqual(left.textView.selectedRange(), selected)
        XCTAssertTrue(coordinator.textView(left.textView, clickedOnLink: expansionLink, at: 0))
        XCTAssertNotNil(received)
        coordinator.onExpandContext = nil
        coordinator.renderIfNeeded()
        XCTAssertNil(link())
        XCTAssertFalse(right.textView.string.isEmpty)
    }

    private func makeRenderer(_ diff: FileDiff) -> (SideBySideCoordinator, DiffTextContainerView, DiffTextContainerView) {
        let view = SideBySideRepresentable(diff: diff)
        let left = view.makeContainer()
        let right = view.makeContainer()
        let coordinator = view.makeCoordinator()
        coordinator.leftContainer = left
        coordinator.rightContainer = right
        coordinator.diff = diff
        coordinator.font = .monospacedSystemFont(ofSize: 12, weight: .regular)
        coordinator.theme = DiffColors(isDark: false)
        for pane in [left, right] {
            pane.wrapsText = false
            pane.scrollView.frame.size = NSSize(width: 600, height: 400)
        }
        coordinator.renderIfNeeded()
        return (coordinator, left, right)
    }
}
