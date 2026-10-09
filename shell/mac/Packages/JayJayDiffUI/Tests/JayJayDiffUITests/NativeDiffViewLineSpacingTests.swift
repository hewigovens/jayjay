import AppKit
import JayJayCore
@testable import JayJayDiffUI
import SwiftUI
import XCTest

@MainActor
final class NativeDiffViewLineSpacingTests: XCTestCase {
    func testLineSpacingKeepsGutterRowsAlignedWithWrappedText() throws {
        let natural = try fragmentRects(lineSpacing: 1)
        let spaced = try fragmentRects(lineSpacing: 1.5)
        XCTAssertGreaterThan(spaced.text.count, 3, "the long line should wrap")
        XCTAssertEqual(spaced.gutter.map(\.minY), spaced.text.map(\.minY))
        XCTAssertEqual(spaced.gutter.map(\.height), spaced.text.map(\.height))
        XCTAssertGreaterThan(try XCTUnwrap(spaced.text.first).height, try XCTUnwrap(natural.text.first).height * 1.4)
    }

    private func fragmentRects(lineSpacing: Double) throws -> (gutter: [NSRect], text: [NSRect]) {
        let diff = FileDiff(
            path: "file.txt",
            language: "",
            lines: [
                line(old: 1, new: 1, style: .context, text: "before"),
                line(old: nil, new: 2, style: .added, text: String(repeating: "wrapped words ", count: 12)),
                line(old: 2, new: 3, style: .context, text: "after")
            ],
            whitespaceOnlyHidden: false
        )
        let hosting = NSHostingView(rootView: NativeDiffView(diff: diff)
            .environment(\.diffTypography, DiffTypography(lineSpacing: lineSpacing))
            .frame(width: 260, height: 300))
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 260, height: 300), styleMask: [.titled], backing: .buffered, defer: false)
        window.contentView = hosting
        hosting.layoutSubtreeIfNeeded()
        for _ in 0 ..< 5 {
            RunLoop.main.run(until: Date().addingTimeInterval(0.05))
        }
        let container = try XCTUnwrap(findContainer(in: hosting))
        return try (rects(in: container.gutterTextView), rects(in: container.textView))
    }

    private func rects(in textView: NSTextView) throws -> [NSRect] {
        let layout = try XCTUnwrap(textView.layoutManager)
        let container = try XCTUnwrap(textView.textContainer)
        layout.ensureLayout(for: container)
        var rects: [NSRect] = []
        layout.enumerateLineFragments(forGlyphRange: layout.glyphRange(for: container)) { rect, _, _, _, _ in
            rects.append(rect)
        }
        return rects
    }

    private func line(old: UInt32?, new: UInt32?, style: DiffSpanStyle, text: String) -> DiffLine {
        DiffLine(
            oldLineNo: old,
            newLineNo: new,
            style: style,
            spans: [DiffSpan(text: text, style: style, token: .plain)],
            conflictKind: .none,
            noEofNewline: false,
            contextRegion: nil
        )
    }

    private func findContainer(in view: NSView) -> DiffTextContainerView? {
        if let container = view as? DiffTextContainerView {
            return container
        }
        return view.subviews.lazy.compactMap { self.findContainer(in: $0) }.first
    }
}
