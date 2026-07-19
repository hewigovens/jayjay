import AppKit
import XCTest

@testable import JayJay

final class WindowContentSizerTests: XCTestCase {
    private let visibleFrame = NSRect(x: 0, y: 25, width: 1024, height: 743)

    func testFittedFrameShrinksOversizedWindowOntoScreen() {
        let frame = NSRect(x: 382, y: 31, width: 1100, height: 677)

        XCTAssertEqual(
            WindowContentSizer.fittedFrame(frame, within: visibleFrame),
            NSRect(x: 0, y: 31, width: 1024, height: 677)
        )
    }

    func testFittedFrameMovesWindowOntoScreenWithoutResizing() {
        let frame = NSRect(x: 900, y: 100, width: 300, height: 400)

        XCTAssertEqual(
            WindowContentSizer.fittedFrame(frame, within: visibleFrame),
            NSRect(x: 724, y: 100, width: 300, height: 400)
        )
    }

    func testFittedFrameLeavesVisibleWindowUnchanged() {
        let frame = NSRect(x: 100, y: 100, width: 800, height: 600)

        XCTAssertEqual(WindowContentSizer.fittedFrame(frame, within: visibleFrame), frame)
    }
}
