import AppKit
import JayJayCore
@testable import JayJayDiffUI
import XCTest

final class DiffGutterRenderingTests: XCTestCase {
    func test_multiLineChangedGroupColumnUsesDrawnStripeNotGlyph() {
        let view = NativeDiffView(diff: FileDiff(
            path: "file.swift",
            language: "swift",
            lines: [],
            whitespaceOnlyHidden: false
        ))
        let changedLine = DiffLine(
            oldLineNo: 1,
            newLineNo: nil,
            style: .removed,
            spans: [],
            conflictKind: .none,
            noEofNewline: false
        )

        XCTAssertEqual(view.groupText(), "  ")
        let stripe = view.groupStripeColor(for: changedLine, groupRange: 2 ... 3, theme: DiffColors(isDark: false))

        XCTAssertGreaterThan(stripe.alphaComponent, 0)
        XCTAssertLessThanOrEqual(stripe.alphaComponent, 0.45)
    }

    func test_singleLineChangeDoesNotDrawGroupStripe() {
        let view = NativeDiffView(diff: FileDiff(
            path: "file.swift",
            language: "swift",
            lines: [],
            whitespaceOnlyHidden: false
        ))
        let changedLine = DiffLine(
            oldLineNo: 1,
            newLineNo: nil,
            style: .removed,
            spans: [],
            conflictKind: .none,
            noEofNewline: false
        )

        let stripe = view.groupStripeColor(for: changedLine, groupRange: 2 ... 2, theme: DiffColors(isDark: false))

        XCTAssertEqual(stripe.alphaComponent, 0)
    }
}
