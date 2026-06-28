import AppKit
import JayJayCore
@testable import JayJayDiffUI
import SwiftUI
import XCTest

/// Regression: with a first line long enough to wrap at narrow widths, the freshly rendered unified diff must start at the top, not pre-scrolled past the first visual rows (seen on CI's 1024x768 runners).
@MainActor
final class NativeDiffViewScrollOriginTests: XCTestCase {
    func testFreshDiffStartsAtTopWhenFirstLineWraps() {
        let lines = (0 ..< 30).map { index in
            DiffLine(
                oldLineNo: nil,
                newLineNo: UInt32(index + 1),
                style: .added,
                spans: [DiffSpan(
                    text: index == 0 ? "func fibonacciReport(limit: Int) -> String { // long enough to wrap" : "line \(index)",
                    style: .added,
                    token: .plain
                )],
                conflictKind: .none,
                noEofNewline: false
            )
        }
        let diff = FileDiff(path: "f.swift", language: "swift", lines: lines, whitespaceOnlyHidden: false)

        let hosting = NSHostingView(rootView: NativeDiffView(diff: diff).frame(width: 366, height: 400))
        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 366, height: 400),
            styleMask: [.titled],
            backing: .buffered,
            defer: false
        )
        window.contentView = hosting
        hosting.layoutSubtreeIfNeeded()
        // Let SwiftUI run updateNSView and the deferred gutter/layout passes.
        for _ in 0 ..< 5 {
            RunLoop.main.run(until: Date().addingTimeInterval(0.05))
        }

        guard let container = findContainer(in: hosting) else {
            return XCTFail("DiffTextContainerView not found in hierarchy")
        }
        let contentOrigin = container.scrollView.contentView.bounds.origin.y
        let gutterOrigin = container.gutterScrollView.contentView.bounds.origin.y
        XCTAssertEqual(contentOrigin, 0, accuracy: 0.5, "content view must open at the top")
        XCTAssertEqual(gutterOrigin, 0, accuracy: 0.5, "gutter must open at the top")
    }

    private func findContainer(in view: NSView) -> DiffTextContainerView? {
        if let container = view as? DiffTextContainerView { return container }
        for subview in view.subviews {
            if let found = findContainer(in: subview) { return found }
        }
        return nil
    }
}
