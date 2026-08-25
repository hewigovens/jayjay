import AppKit
import XCTest

final class WindowFramePersistenceScene: SceneBase {
    override class var startsWithDefaultLayout: Bool {
        false
    }

    func testMovedAndResizedRepositoryWindowKeepsItsFrameAcrossLaunches() throws {
        let app = try XCTUnwrap(app)
        let repoWindow = app.windows["simple"]
        XCTAssertTrue(repoWindow.waitForExistence(timeout: 10), "Repository window did not appear")
        let launchFrame = repoWindow.frame

        // A screen-sized window has no resize handle and AppKit pulls it back on relaunch, so persistence cannot be exercised here.
        let visible = NSScreen.main?.visibleFrame ?? .zero
        let roomLeft = launchFrame.minX - visible.minX
        let roomRight = visible.maxX - launchFrame.maxX
        guard launchFrame.height + 100 < visible.height, max(roomLeft, roomRight) >= 100 else {
            throw XCTSkip("Screen too small to move or resize the repository window: \(launchFrame.size) on \(visible.size)")
        }

        let titleBar = try freeTitleBarPoint(in: repoWindow)
        titleBar.press(forDuration: 0.3, thenDragTo: titleBar.withOffset(CGVector(dx: roomLeft >= 100 ? -90 : 90, dy: 20)))
        sleep(1)
        XCTAssertNotEqual(repoWindow.frame.minX, launchFrame.minX, "Dragging the title bar did not move the repository window")

        let corner = repoWindow.coordinate(withNormalizedOffset: CGVector(dx: 1, dy: 1))
        let delta = repoWindow.frame.width > 1000 ? CGVector(dx: -120, dy: -80) : CGVector(dx: 120, dy: 80)
        corner.press(forDuration: 0.3, thenDragTo: corner.withOffset(delta))
        sleep(1)
        XCTAssertNotEqual(repoWindow.frame.size, launchFrame.size, "Dragging the window corner did not resize the repository window")
        let moved = repoWindow.frame

        app.terminate()
        app.launch()

        let relaunched = app.windows["simple"]
        XCTAssertTrue(relaunched.waitForExistence(timeout: 10), "Repository window did not reappear after relaunch")
        XCTAssertEqual(relaunched.frame.minX, moved.minX, accuracy: 4, "Repository window lost its horizontal position")
        XCTAssertEqual(relaunched.frame.minY, moved.minY, accuracy: 4, "Repository window lost its vertical position")
        XCTAssertEqual(relaunched.frame.width, moved.width, accuracy: 4, "Repository window lost its width")
        XCTAssertEqual(relaunched.frame.height, moved.height, accuracy: 4, "Repository window lost its height")
    }

    /// A title-bar spot no toolbar item claims, so the press starts a window drag.
    private func freeTitleBarPoint(in window: XCUIElement) throws -> XCUICoordinate {
        let frame = window.frame
        let y = frame.minY + 14
        let covered = window.toolbars.firstMatch.descendants(matching: .any).allElementsBoundByIndex
            .map(\.frame)
            .filter { !$0.isEmpty }
        let x = try XCTUnwrap(
            stride(from: frame.minX + 90, to: frame.maxX - 20, by: 16).first { x in
                !covered.contains { $0.contains(CGPoint(x: x, y: y)) }
            },
            "No free title-bar spot in \(covered)"
        )
        return window.coordinate(withNormalizedOffset: .zero).withOffset(CGVector(dx: x - frame.minX, dy: 14))
    }
}
