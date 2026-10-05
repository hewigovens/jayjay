import AppKit
import JayJayCore
@testable import JayJayDiffUI
import XCTest

final class NSTextStorageReplacementTests: XCTestCase {
    func testReplacementPublishesCompleteAttributedTextInOneEdit() throws {
        let pane = makePane()
        let view = pane.textView
        let storage = try XCTUnwrap(view.textStorage)
        storage.setAttributedString(NSAttributedString(
            string: "old content with obsolete attributes",
            attributes: [.backgroundColor: NSColor.red, .link: "obsolete"]
        ))
        view.setSelectedRange(NSRange(location: 20, length: 10))
        let text = NSMutableAttributedString(
            string: "👩🏽‍💻 e\u{301} 日本語\nexpand\n",
            attributes: [.font: NSFont.monospacedSystemFont(ofSize: 14, weight: .bold), .foregroundColor: NSColor.blue]
        )
        let linkRange = (text.string as NSString).range(of: "expand")
        text.addAttribute(.link, value: "expand-context", range: linkRange)
        text.fixAttributes(in: NSRange(location: 0, length: text.length))
        var published: [NSAttributedString] = []
        let observer = NotificationCenter.default.addObserver(
            forName: NSTextStorage.didProcessEditingNotification, object: storage, queue: nil
        ) { _ in published.append(NSAttributedString(attributedString: storage)) }
        defer { NotificationCenter.default.removeObserver(observer) }

        for replacement in [text, NSAttributedString(string: ""), text] {
            published.removeAll()
            storage.replaceContents(with: replacement)
            XCTAssertEqual(published.count, 1)
            XCTAssertEqual(published.first, replacement)
            XCTAssertEqual(storage, replacement)
            XCTAssertTrue(view.textStorage === storage)
            XCTAssertTrue(view.layoutManager?.textStorage === storage)
            XCTAssertLessThanOrEqual(NSMaxRange(view.selectedRange()), storage.length)
            let manager = try XCTUnwrap(view.layoutManager as? DiffLayoutManager)
            XCTAssertEqual(manager.findMatchRanges("日本語").count, replacement.length > 0 ? 1 : 0)
            try manager.ensureLayout(for: XCTUnwrap(view.textContainer))
            XCTAssertEqual(manager.numberOfGlyphs > 0, replacement.length > 0)
        }
    }

    func testReplacingManyAttributeRunsRemainsResponsive() throws {
        let pane = makePane()
        let storage = try XCTUnwrap(pane.textView.textStorage)
        let text = NSMutableAttributedString()
        let font = NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)
        for _ in 0 ..< 6000 {
            for token in 0 ..< 16 {
                text.append(NSAttributedString(
                    string: token == 15 ? "value;\n" : "value ",
                    attributes: [.font: font, .foregroundColor: token.isMultiple(of: 2) ? NSColor.red : NSColor.blue]
                ))
            }
        }
        storage.setAttributedString(text)

        let started = CFAbsoluteTimeGetCurrent()
        storage.replaceContents(with: text)
        let elapsedMs = (CFAbsoluteTimeGetCurrent() - started) * 1000

        XCTAssertEqual(storage, text)
        // Direct replacement takes over a second; clearing first leaves ample headroom under this limit.
        XCTAssertLessThan(elapsedMs, 500, "Replacing styled text took \(elapsedMs) ms")
    }

    private func makePane() -> DiffTextContainerView {
        let diff = FileDiff(path: "test.rs", language: "rust", lines: [], whitespaceOnlyHidden: false)
        return SideBySideRepresentable(diff: diff).makeContainer()
    }
}
