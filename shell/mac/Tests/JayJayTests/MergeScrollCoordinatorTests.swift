import AppKit
@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class MergeScrollCoordinatorTests: XCTestCase {
    func testUnequalWrappedPanesFollowEveryLeaderAndRespectMode() {
        let text = (0 ..< 100).map { "line \($0) " + String(repeating: "wide 日本語 ", count: 12) }.joined(separator: "\n")
        let controller = MergeScrollCoordinator()
        controller.isRaw = true
        controller.update(map: MergeScrollMap(left: text, base: text, right: text, original: text, hunks: []))
        let left = makePane(text, width: 220, pane: .left, controller: controller)
        let right = makePane(text, width: 340, pane: .right, controller: controller)
        let result = makePane(text, width: 700, height: 420, pane: .result, controller: controller)
        let panes = [left, right, result]
        for (index, pane) in panes.enumerated() {
            let line = 20.4 + Double(index) * 10
            pane.anchor.scroll(to: line)
            for follower in panes {
                XCTAssertEqual(follower.anchor.centerLine, line, accuracy: 0.08)
            }
        }
        left.anchor.scroll(to: 45)
        XCTAssertEqual(result.anchor.centerLine, 45, accuracy: 0.08)
        controller.isRaw = false
        left.anchor.scroll(to: 55)
        XCTAssertEqual(result.anchor.centerLine, 45, accuracy: 0.08)
        XCTAssertEqual(right.anchor.centerLine, 55, accuracy: 0.08)
        controller.isRaw = true
        controller.invalidateResult()
        left.anchor.scroll(to: 65)
        XCTAssertEqual(right.anchor.centerLine, 65, accuracy: 0.08)
        XCTAssertEqual(result.anchor.centerLine, 45, accuracy: 0.08)
    }

    func testHunkRevealAndBaseRestoration() async {
        let before = (0 ..< 40).map { "before \($0)\n" }.joined()
        let after = (0 ..< 80).map { "after \($0)\n" }.joined()
        let raw = "<<<<<<<\nleft\n=======\nright\n>>>>>>>\n"
        let original = before + raw + after
        let source = before + "source\n" + after
        let hunk = MergeEditorHunk(index: 0, occurrence: 0, raw: raw, left: "source\n", base: "source\n", right: "source\n")
        let controller = MergeScrollCoordinator()
        controller.update(map: MergeScrollMap(left: source, base: source, right: source, original: original, hunks: [hunk]))
        controller.reveal(hunk: 0)
        let left = makePane(source, width: 250, pane: .left, controller: controller)
        await waitForLine(40, in: left.anchor)
        let result = makePane(original, width: 500, pane: .result, controller: controller)
        let initialResultLine = result.anchor.centerLine
        controller.reveal(hunk: 0)
        XCTAssertEqual(left.anchor.centerLine, 40, accuracy: 0.08)
        XCTAssertEqual(result.anchor.centerLine, initialResultLine, accuracy: 0.08)
        let right = makePane(source, width: 350, pane: .right, controller: controller)
        await waitForLine(40, in: right.anchor)
        right.anchor.scroll(to: 70)
        let base = makePane(source, width: 500, pane: .base, controller: controller)
        await waitForLine(70, in: base.anchor)
        XCTAssertEqual(left.anchor.centerLine, 70, accuracy: 0.08)
        XCTAssertEqual(right.anchor.centerLine, 70, accuracy: 0.08)
        controller.isRaw = true
        base.anchor.scroll(to: 20)
        controller.reveal(hunk: 0)
        XCTAssertEqual(base.anchor.centerLine, 20, accuracy: 0.08)
        controller.unregister(.base, anchor: base.anchor)
        let restoredBase = makePane(source, width: 500, pane: .base, controller: controller)
        await waitForLine(20, in: restoredBase.anchor)
    }

    func testHunkListAndSourcesFollowEachOther() {
        let context = (0 ..< 50).map { "context \($0)\n" }.joined()
        let raw = "<<<<<<<\nleft\n=======\nright\n>>>>>>>\n"
        let original = context + raw + context + raw + context
        let source = context + "source\n" + context + "source\n" + context
        let hunks = (0 ..< 2).map {
            MergeEditorHunk(index: UInt32($0), occurrence: UInt32($0), raw: raw, left: "source\n", base: "source\n", right: "source\n")
        }
        let controller = MergeScrollCoordinator()
        controller.update(map: MergeScrollMap(left: source, base: source, right: source, original: original, hunks: hunks), hunks: [0, 1])
        let left = makePane(source, width: 250, pane: .left, controller: controller)
        let right = makePane(source, width: 500, height: 340, pane: .right, controller: controller)
        controller.reveal(hunk: 0)
        right.anchor.scroll(to: 110)
        XCTAssertEqual(left.anchor.centerLine, 110, accuracy: 0.08)
        XCTAssertEqual(controller.visibleHunk, 1)
        controller.didScroll(hunk: 0)
        XCTAssertEqual(left.anchor.centerLine, 50, accuracy: 0.08)
        XCTAssertEqual(right.anchor.centerLine, 50, accuracy: 0.08)
        controller.didScroll(hunk: 1)
        XCTAssertEqual(left.anchor.centerLine, 101, accuracy: 0.08)
        XCTAssertEqual(right.anchor.centerLine, 101, accuracy: 0.08)
    }

    func testResultMapRefreshReconcilesCurrentResultPosition() throws {
        let source = (0 ..< 120).map { "line \($0)\n" }.joined()
        let edited = String(repeating: "inserted\n", count: 10) + source
        let map = MergeScrollMap(left: source, base: source, right: source, original: source, hunks: [])
        let controller = MergeScrollCoordinator()
        controller.isRaw = true
        controller.update(map: map)
        let left = makePane(source, width: 250, pane: .left, controller: controller)
        let result = makePane(source, width: 500, pane: .result, controller: controller)
        result.anchor.scroll(to: 30)
        controller.invalidateResult()
        let view = try XCTUnwrap(result.scrollView.documentView as? NSTextView)
        view.string = edited
        view.sizeToFit()
        result.anchor.updateText()
        result.anchor.scroll(to: 70)
        XCTAssertEqual(left.anchor.centerLine, 30, accuracy: 0.08)
        controller.update(map: map.withResult(result: edited))
        XCTAssertEqual(left.anchor.centerLine, 60, accuracy: 0.08)
    }

    func testNewRawPaneRestoresSourcePositionAfterPendingMapRefresh() async {
        let source = (0 ..< 120).map { "line \($0)\n" }.joined()
        let edited = "inserted\n" + source
        let map = MergeScrollMap(left: source, base: source, right: source, original: source, hunks: [])
        let controller = MergeScrollCoordinator()
        controller.update(map: map)
        let left = makePane(source, width: 250, pane: .left, controller: controller)
        left.anchor.scroll(to: 60)
        controller.invalidateResult()
        controller.isRaw = true
        let result = makePane(edited, width: 500, pane: .result, controller: controller)
        controller.update(map: map.withResult(result: edited))
        await waitForLine(61, in: result.anchor)
        XCTAssertEqual(left.anchor.centerLine, 60, accuracy: 0.08)
    }

    private func waitForLine(_ line: Double, in anchor: MergeTextScrollAnchor, file: StaticString = #filePath, lineNumber: UInt = #line) async {
        let deadline = ContinuousClock.now.advanced(by: .seconds(5))
        while abs(anchor.centerLine - line) > 0.08, ContinuousClock.now < deadline {
            await Task.yield()
        }
        XCTAssertEqual(anchor.centerLine, line, accuracy: 0.08, file: file, line: lineNumber)
    }

    private func makePane(_ text: String, width: CGFloat, height: CGFloat = 180, pane: MergePane, controller: MergeScrollCoordinator) -> (scrollView: NSScrollView, anchor: MergeTextScrollAnchor) {
        let scroll = NSScrollView(frame: NSRect(x: 0, y: 0, width: width, height: height))
        let view = NSTextView(frame: scroll.bounds)
        view.isVerticallyResizable = true
        view.isHorizontallyResizable = false
        view.autoresizingMask = [.width]
        view.textContainer?.widthTracksTextView = true
        view.textContainer?.containerSize = NSSize(width: width, height: .greatestFiniteMagnitude)
        view.font = CodeTextView.editorFont
        view.textContainerInset = NSSize(width: 10, height: 10)
        view.string = text
        scroll.documentView = view
        view.sizeToFit()
        let anchor = MergeTextScrollAnchor(textView: view) { [weak controller] in controller?.didScroll(pane) }
        controller.register(pane, anchor: anchor)
        return (scroll, anchor)
    }
}
