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
            noEofNewline: false,
            contextRegion: nil
        )

        let stripe = view.groupStripeColor(for: changedLine, groupRange: 2 ... 3, theme: DiffColors(isDark: false))

        XCTAssertGreaterThan(stripe.alphaComponent, 0)
        XCTAssertLessThanOrEqual(stripe.alphaComponent, 0.45)
    }

    func test_reviewedStateUsesStripeWhileNoteMarkerRendersAsGlyph() {
        let lines = [
            DiffLine(
                oldLineNo: nil,
                newLineNo: 1,
                style: .added,
                spans: [DiffSpan(text: "added", style: .added, token: .plain)],
                conflictKind: .none,
                noEofNewline: false,
                contextRegion: nil
            )
        ]
        let view = NativeDiffView(diff: FileDiff(
            path: "file.swift",
            language: "swift",
            lines: lines,
            whitespaceOnlyHidden: false
        ))
        let font = NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)
        let container = NSTextContainer(containerSize: NSSize(width: 100, height: CGFloat.greatestFiniteMagnitude))
        let manager = DiffLayoutManager()
        manager.addTextContainer(container)
        let storage = NSTextStorage()
        storage.addLayoutManager(manager)
        let gutter = DiffGutterTextView(frame: NSRect(x: 0, y: 0, width: 80, height: 40), textContainer: container)
        let reviewActions = AllReviewedActions()

        _ = view.renderWrappedGutter(
            gutterTextView: gutter,
            gutterLayoutManager: manager,
            context: NativeDiffGutterRenderContext(
                content: .init(
                    lines: lines,
                    rows: diffRenderRows(displayLines: lines, notesByLine: [:]),
                    visualLineCounts: [1]
                ),
                style: .init(
                    font: font,
                    theme: DiffColors(isDark: false),
                    gutterAttrs: [.font: font],
                    gutterParagraphStyle: NSMutableParagraphStyle(),
                    maxLineDigits: 2
                ),
                layout: .init(
                    groupStripeWidth: 6,
                    gutterHorizontalInset: 8,
                    gutterTrailingPadding: 10,
                    showsCheckboxColumn: false,
                    showsNoteColumn: true
                ),
                review: .init(
                    reviewModeEnabled: true,
                    groupIndexAtLineNumber: [1: 0],
                    reviewActions: reviewActions,
                    notedLines: [1],
                    resolvedOnlyLines: [],
                    currentSelectedLineRange: nil
                )
            )
        )

        let rendered = gutter.textStorage?.string ?? ""
        let firstLine = rendered.components(separatedBy: "\n").first ?? ""
        XCTAssertFalse(firstLine.contains("✓"), "reviewed state must not add a glyph; the stripe is the signal: \(firstLine)")
        XCTAssertTrue(firstLine.contains("●"), "note marker must render in its own column: \(firstLine)")
        XCTAssertEqual(
            manager.lineStripeColors.first,
            .controlAccentColor,
            "reviewed group shows through the accent stripe"
        )
    }

    func test_changedSinceReviewUsesOrangeStripe() {
        let lines = [
            DiffLine(
                oldLineNo: nil,
                newLineNo: 1,
                style: .added,
                spans: [DiffSpan(text: "added", style: .added, token: .plain)],
                conflictKind: .none,
                noEofNewline: false,
                contextRegion: nil
            )
        ]
        let view = NativeDiffView(diff: FileDiff(
            path: "file.swift",
            language: "swift",
            lines: lines,
            whitespaceOnlyHidden: false
        ))
        let font = NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)
        let container = NSTextContainer(containerSize: NSSize(width: 100, height: CGFloat.greatestFiniteMagnitude))
        let manager = DiffLayoutManager()
        manager.addTextContainer(container)
        let storage = NSTextStorage()
        storage.addLayoutManager(manager)
        let gutter = DiffGutterTextView(frame: NSRect(x: 0, y: 0, width: 80, height: 40), textContainer: container)

        _ = view.renderWrappedGutter(
            gutterTextView: gutter,
            gutterLayoutManager: manager,
            context: NativeDiffGutterRenderContext(
                content: .init(
                    lines: lines,
                    rows: diffRenderRows(displayLines: lines, notesByLine: [:]),
                    visualLineCounts: [1]
                ),
                style: .init(
                    font: font,
                    theme: DiffColors(isDark: false),
                    gutterAttrs: [.font: font],
                    gutterParagraphStyle: NSMutableParagraphStyle(),
                    maxLineDigits: 2
                ),
                layout: .init(
                    groupStripeWidth: 6,
                    gutterHorizontalInset: 8,
                    gutterTrailingPadding: 10,
                    showsCheckboxColumn: false,
                    showsNoteColumn: false
                ),
                review: .init(
                    reviewModeEnabled: true,
                    groupIndexAtLineNumber: [1: 0],
                    reviewActions: FixedReviewActions(state: .changedSinceReview),
                    notedLines: [],
                    resolvedOnlyLines: [],
                    currentSelectedLineRange: nil
                )
            )
        )

        XCTAssertEqual(manager.lineStripeColors.first, .systemOrange)
    }

    func test_reviewStateMapsToGutterStripeColors() {
        XCTAssertEqual(DiffGutterHunkReviewState.reviewed.stripeColor, .controlAccentColor)
        XCTAssertEqual(DiffGutterHunkReviewState.unreviewed.stripeColor, .selectedTextBackgroundColor)
        XCTAssertEqual(DiffGutterHunkReviewState.changedSinceReview.stripeColor, .systemOrange)
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
            noEofNewline: false,
            contextRegion: nil
        )

        let stripe = view.groupStripeColor(for: changedLine, groupRange: 2 ... 2, theme: DiffColors(isDark: false))

        XCTAssertEqual(stripe.alphaComponent, 0)
    }
}

private final class AllReviewedActions: DiffGutterReviewActions {
    var currentSelectedLineRange: ClosedRange<Int>?
    var reviewModeEnabled = true
    func didSelectLines(_ lineRange: ClosedRange<Int>) {}
    func hunkReviewState(groupIndex _: UInt32) -> DiffGutterHunkReviewState {
        .reviewed
    }

    func toggleHunkReviewed(groupIndex _: UInt32) {}
}

private final class FixedReviewActions: DiffGutterReviewActions {
    var currentSelectedLineRange: ClosedRange<Int>?
    var reviewModeEnabled = true
    let state: DiffGutterHunkReviewState

    init(state: DiffGutterHunkReviewState) {
        self.state = state
    }

    func didSelectLines(_ lineRange: ClosedRange<Int>) {}
    func hunkReviewState(groupIndex _: UInt32) -> DiffGutterHunkReviewState {
        state
    }

    func toggleHunkReviewed(groupIndex _: UInt32) {}
}
