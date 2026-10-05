import AppKit
import JayJayCore
@testable import JayJayDiffUI
import SwiftUI
import XCTest

@MainActor
final class NativeDiffViewSymbolFontTests: XCTestCase {
    func testTextDefaultSymbolsKeepATextFontWhileEmojiStayEmoji() {
        let code = "case .return: \"↩\" // ❤\u{FE0F} 😀 🏃\u{200D}♀"
        let diff = FileDiff(
            path: "file.swift",
            language: "swift",
            lines: [DiffLine(
                oldLineNo: nil,
                newLineNo: 1,
                style: .added,
                spans: [DiffSpan(text: code, style: .added, token: .plain)],
                conflictKind: .none,
                noEofNewline: false,
                contextRegion: nil
            )],
            whitespaceOnlyHidden: false
        )
        let hosting = NSHostingView(rootView: NativeDiffView(diff: diff).frame(width: 366, height: 160))
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 366, height: 160), styleMask: [.titled], backing: .buffered, defer: false)
        window.contentView = hosting
        hosting.layoutSubtreeIfNeeded()
        for _ in 0 ..< 5 {
            RunLoop.main.run(until: Date().addingTimeInterval(0.05))
        }
        guard let storage = findContainer(in: hosting)?.textView.textStorage else {
            return XCTFail("DiffTextContainerView not found in hierarchy")
        }

        let text = storage.string as NSString
        XCTAssertFalse(fontName(at: text.range(of: "↩").location, in: storage).contains("Emoji"))
        XCTAssertTrue(fontName(at: text.range(of: "❤", options: .literal).location, in: storage).contains("Emoji"))
        XCTAssertTrue(fontName(at: text.range(of: "😀").location, in: storage).contains("Emoji"))
        XCTAssertTrue(fontName(at: text.range(of: "♀", options: .literal).location, in: storage).contains("Emoji"))
    }

    private func fontName(at index: Int, in storage: NSTextStorage) -> String {
        (storage.attribute(.font, at: index, effectiveRange: nil) as? NSFont)?.fontName ?? ""
    }

    private func findContainer(in view: NSView) -> DiffTextContainerView? {
        if let container = view as? DiffTextContainerView {
            return container
        }
        return view.subviews.lazy.compactMap { self.findContainer(in: $0) }.first
    }
}
