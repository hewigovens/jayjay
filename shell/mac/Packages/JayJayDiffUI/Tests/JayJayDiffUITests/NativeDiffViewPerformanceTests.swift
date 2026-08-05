import AppKit
import JayJayCore
@testable import JayJayDiffUI
import XCTest

final class NativeDiffViewPerformanceTests: XCTestCase {
    func test_gutterRender_largeDiff_doesNotBlockMainThread() {
        // ~1500 display lines with scattered change groups, like a lockfile bump.
        let lines: [DiffLine] = (0 ..< 1500).map { i in
            let style: DiffSpanStyle = (i % 6 == 0 || i % 6 == 1) ? .added : .context
            return DiffLine(
                oldLineNo: UInt32(i + 1),
                newLineNo: UInt32(i + 1),
                style: style,
                spans: [],
                conflictKind: .none,
                noEofNewline: false,
                contextRegion: nil
            )
        }
        let view = NativeDiffView(diff: FileDiff(
            path: "Cargo.lock",
            language: "",
            lines: lines,
            whitespaceOnlyHidden: false
        ))
        let (gutterTextView, gutterLayoutManager) = makeGutter()
        let context = makeContext(lines: lines)

        let start = Date()
        _ = view.renderWrappedGutter(
            gutterTextView: gutterTextView,
            gutterLayoutManager: gutterLayoutManager,
            context: context
        )
        let elapsedMs = Date().timeIntervalSince(start) * 1000

        // Post-fix: single-digit ms; pre-fix O(n^2) FFI: multiple seconds. 500ms gates the regression with CI headroom.
        XCTAssertLessThan(
            elapsedMs, 500,
            "gutter render took \(Int(elapsedMs))ms for 1500 lines — per-line diffDisplayLines FFI likely reintroduced"
        )
    }

    // MARK: - Fixtures

    private func makeGutter() -> (DiffGutterTextView, DiffLayoutManager) {
        let container = NSTextContainer(containerSize: NSSize(
            width: 80,
            height: CGFloat.greatestFiniteMagnitude
        ))
        container.widthTracksTextView = true
        container.lineFragmentPadding = 0

        let manager = DiffLayoutManager()
        manager.addTextContainer(container)

        let storage = NSTextStorage()
        storage.addLayoutManager(manager)

        let textView = DiffGutterTextView(
            frame: NSRect(x: 0, y: 0, width: 80, height: 400),
            textContainer: container
        )
        return (textView, manager)
    }

    private func makeContext(lines: [DiffLine]) -> NativeDiffGutterRenderContext {
        let font = NSFont.monospacedSystemFont(ofSize: 12, weight: .regular)
        let paragraph = NSMutableParagraphStyle()
        return NativeDiffGutterRenderContext(
            content: .init(
                lines: lines,
                rows: diffRenderRows(displayLines: lines, notesByLine: [:]),
                visualLineCounts: Array(repeating: 1, count: lines.count)
            ),
            style: .init(
                font: font,
                theme: DiffColors(isDark: false),
                gutterAttrs: [.font: font],
                gutterParagraphStyle: paragraph,
                maxLineDigits: 4
            ),
            layout: .init(
                groupStripeWidth: 6,
                gutterHorizontalInset: 8,
                gutterTrailingPadding: 10,
                showsCheckboxColumn: false,
                showsNoteColumn: false
            ),
            review: .init(
                reviewModeEnabled: false,
                groupIndexAtLineNumber: [:],
                reviewActions: nil,
                notedLines: [],
                resolvedOnlyLines: [],
                currentSelectedLineRange: nil
            )
        )
    }
}
