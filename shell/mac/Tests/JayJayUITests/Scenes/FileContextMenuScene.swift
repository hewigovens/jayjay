import AppKit
import XCTest

final class FileContextMenuScene: SceneBase {
    override class var launchEnvironment: [String: String] {
        ["JAYJAY_CAPTURE_SHOW_IN_FINDER_PASTEBOARD": "1"]
    }

    func testShowInFinderFromFileListContextMenu() throws {
        let app = try XCTUnwrap(app)
        XCTAssertTrue(dagRows(of: app).element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        let expectedPath = try XCTUnwrap(fixtureURL)
            .appendingPathComponent("wip1.txt")
            .path
        NSPasteboard.general.clearContents()

        let fileRow = app.descendants(matching: .any)
            .matching(NSPredicate(format: "identifier == %@", AID.FileList.row("wip1.txt")))
            .firstMatch
        rightClickCenter(fileRow, timeout: 10, message: "wip1.txt row missing")

        let finderItems = app.menuItems.matching(identifier: "Show in Finder")
        XCTAssertTrue(finderItems.firstMatch.waitForExistence(timeout: 3), "\"Show in Finder\" menu item missing")
        let showInFinder = try XCTUnwrap(finderItems.allElementsBoundByIndex.first(where: \.isHittable))
        XCTAssertTrue(showInFinder.isEnabled, "\"Show in Finder\" menu item disabled")
        keyStroke("f")
        let revealed = NSPredicate { _, _ in NSPasteboard.general.string(forType: .string) == expectedPath }
        XCTAssertEqual(
            XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: revealed, object: nil)], timeout: 3),
            .completed
        )

        NSPasteboard.general.clearContents()
        keyStroke("f")
        XCTAssertEqual(
            XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: revealed, object: nil)], timeout: 1),
            .timedOut,
            "F ran a context-menu action after the menu closed"
        )
    }
}
