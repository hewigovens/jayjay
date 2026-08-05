import AppKit
import JayJayCore
@testable import JayJayDiffUI
import SwiftUI
import XCTest

@MainActor
final class NativeDiffViewContextStateTests: XCTestCase {
    func testSelectionResetGenerationClearsNativeContentAndGutterSelections() {
        let diff = FileDiff(
            path: "file.swift",
            language: "swift",
            lines: [contextLine(1)],
            whitespaceOnlyHidden: false
        )
        let hosting = NSHostingView(rootView: NativeDiffView(
            diff: diff,
            resetSelectionGeneration: 0
        ).frame(width: 366, height: 160))
        let window = host(hosting, width: 366, height: 160)
        waitForRender(hosting)
        guard let container = findContainer(in: hosting) else {
            return XCTFail("DiffTextContainerView not found in hierarchy")
        }

        container.textView.setSelectedRange(NSRange(location: 0, length: 1))
        container.gutterTextView.setSelectedRange(NSRange(location: 0, length: 1))
        container.gutterTextView.selectionAnchorLine = 1
        hosting.rootView = NativeDiffView(
            diff: diff,
            resetSelectionGeneration: .max
        ).frame(width: 366, height: 160)
        waitForRender(hosting)

        XCTAssertEqual(container.textView.selectedRange().length, 0)
        XCTAssertEqual(container.gutterTextView.selectedRange().length, 0)
        XCTAssertNil(container.gutterTextView.selectionAnchorLine)

        container.textView.setSelectedRange(NSRange(location: 0, length: 1))
        hosting.rootView = NativeDiffView(
            diff: diff,
            resetSelectionGeneration: 0
        ).frame(width: 366, height: 160)
        waitForRender(hosting)

        XCTAssertEqual(container.textView.selectedRange().length, 0)
        _ = window
    }

    func testExpandedDiffPreservesStableVisibleLineAnchor() {
        let region = ContextRegion(
            id: 11,
            oldStartLine: 11,
            newStartLine: 11,
            lineCount: 20,
            initialLineCount: 20
        )
        let collapsedLines = (1 ... 10).map(contextLine)
            + [separatorLine(region)]
            + (31 ... 80).map(contextLine)
        let expandedRegion = ContextRegion(
            id: region.id,
            oldStartLine: region.oldStartLine,
            newStartLine: region.newStartLine,
            lineCount: 10,
            initialLineCount: region.initialLineCount
        )
        let expandedLines = (1 ... 10).map(contextLine)
            + [separatorLine(expandedRegion)]
            + (21 ... 80).map(contextLine)
        let collapsed = FileDiff(
            path: "file.swift",
            language: "swift",
            lines: collapsedLines,
            whitespaceOnlyHidden: false
        )
        let expanded = FileDiff(
            path: "file.swift",
            language: "swift",
            lines: expandedLines,
            whitespaceOnlyHidden: false
        )
        let hosting = NSHostingView(rootView: NativeDiffView(
            diff: collapsed,
            onExpandContext: { _ in }
        ).frame(width: 366, height: 180))
        let window = host(hosting, width: 366, height: 180)
        waitForRender(hosting)
        guard let container = findContainer(in: hosting) else {
            return XCTFail("DiffTextContainerView not found in hierarchy")
        }

        scroll("line 40", toTopIn: container)
        let before = container.captureViewportAnchor()
        hosting.rootView = NativeDiffView(
            diff: expanded,
            onExpandContext: { _ in }
        ).frame(width: 366, height: 180)
        waitForRender(hosting)
        let after = container.captureViewportAnchor()

        XCTAssertEqual(after?.identity, before?.identity)
        XCTAssertEqual(
            after?.offsetFromVisibleTop ?? 0,
            before?.offsetFromVisibleTop ?? 0,
            accuracy: 1
        )
        _ = window
    }

    func testTrailingRevealAnchorsToRevealedLinesNotTheMovedSeparator() {
        let region = ContextRegion(
            id: 5,
            oldStartLine: 5,
            newStartLine: 5,
            lineCount: 53,
            initialLineCount: 53
        )
        let collapsedLines = (1 ... 4).map(contextLine) + [separatorLine(region)]
        let movedRegion = ContextRegion(
            id: region.id,
            oldStartLine: 15,
            newStartLine: 15,
            lineCount: 43,
            initialLineCount: region.initialLineCount
        )
        let expandedLines = (1 ... 14).map(contextLine) + [separatorLine(movedRegion)]
        let collapsed = FileDiff(
            path: "file.swift",
            language: "swift",
            lines: collapsedLines,
            whitespaceOnlyHidden: false
        )
        let expanded = FileDiff(
            path: "file.swift",
            language: "swift",
            lines: expandedLines,
            whitespaceOnlyHidden: false
        )
        let hosting = NSHostingView(rootView: NativeDiffView(
            diff: collapsed,
            onExpandContext: { _ in }
        ).frame(width: 366, height: 90))
        let window = host(hosting, width: 366, height: 90)
        waitForRender(hosting)
        guard let container = findContainer(in: hosting) else {
            return XCTFail("DiffTextContainerView not found in hierarchy")
        }

        scroll("unmodified lines", toTopIn: container)
        hosting.rootView = NativeDiffView(
            diff: expanded,
            onExpandContext: { _ in }
        ).frame(width: 366, height: 90)
        waitForRender(hosting)
        let after = container.captureViewportAnchor()

        XCTAssertNil(
            after?.identity.contextRegionId,
            "the anchor must fall back to the revealed start line instead of following the moved separator: \(String(describing: after?.identity))"
        )
        XCTAssertEqual(after?.identity.newLine, 5)
        _ = window
    }

    private func contextLine(_ number: Int) -> DiffLine {
        DiffLine(
            oldLineNo: UInt32(number),
            newLineNo: UInt32(number),
            style: .context,
            spans: [DiffSpan(
                text: "line \(number)",
                style: .context,
                token: .plain
            )],
            conflictKind: .none,
            noEofNewline: false,
            contextRegion: nil
        )
    }

    private func separatorLine(_ region: ContextRegion) -> DiffLine {
        DiffLine(
            oldLineNo: nil,
            newLineNo: nil,
            style: .separator,
            spans: [DiffSpan(
                text: "\(region.lineCount) unmodified lines",
                style: .separator,
                token: .plain
            )],
            conflictKind: .none,
            noEofNewline: false,
            contextRegion: region
        )
    }

    private func host(
        _ hosting: NSHostingView<some View>,
        width: CGFloat,
        height: CGFloat
    ) -> NSWindow {
        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: width, height: height),
            styleMask: [.titled],
            backing: .buffered,
            defer: false
        )
        window.contentView = hosting
        return window
    }

    private func waitForRender(_ hosting: NSView) {
        hosting.layoutSubtreeIfNeeded()
        for _ in 0 ..< 5 {
            RunLoop.main.run(until: Date().addingTimeInterval(0.05))
        }
    }

    private func scroll(_ text: String, toTopIn container: DiffTextContainerView) {
        guard let range = container.textView.string.range(of: text),
              let layoutManager = container.textView.layoutManager,
              let textContainer = container.textView.textContainer
        else {
            return XCTFail("Expected line not found in rendered diff")
        }
        let characterRange = NSRange(range, in: container.textView.string)
        let glyphRange = layoutManager.glyphRange(
            forCharacterRange: characterRange,
            actualCharacterRange: nil
        )
        let rect = layoutManager.boundingRect(
            forGlyphRange: glyphRange,
            in: textContainer
        )
        container.scrollView.contentView.scroll(to: NSPoint(
            x: 0,
            y: rect.minY + container.textView.textContainerInset.height
        ))
        container.scrollView.reflectScrolledClipView(container.scrollView.contentView)
    }

    private func findContainer(in view: NSView) -> DiffTextContainerView? {
        if let container = view as? DiffTextContainerView {
            return container
        }
        for subview in view.subviews {
            if let found = findContainer(in: subview) {
                return found
            }
        }
        return nil
    }
}
