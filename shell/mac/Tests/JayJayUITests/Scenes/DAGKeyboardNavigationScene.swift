import XCTest

final class DAGKeyboardNavigationScene: SceneBase {
    override class var fixtureName: String {
        "dag-long"
    }

    func testArrowKeysKeepTheSelectionVisibleAndLoadTheSettledChange() throws {
        let app = try XCTUnwrap(app)
        let rows = try waitForDAGRows()
        rows.firstMatch.click()

        for _ in 0 ..< 15 {
            keyStroke(.downArrow)
        }

        let settledFile = app.element(AID.FileList.row("file-10.txt"))
        XCTAssertTrue(settledFile.waitForExistence(timeout: 10), "Detail did not load the change the keys settled on")
        let selectedRow = rows.matching(NSPredicate(format: "isSelected == YES")).firstMatch
        XCTAssertTrue(selectedRow.exists, "Selected row is not rendered")
        XCTAssertTrue(selectedRow.isHittable, "Selected row scrolled out of view")
    }

    func testControlReturnOpensTheMenuForASelectionScrolledOutOfView() throws {
        let app = try XCTUnwrap(app)
        let rows = try waitForDAGRows()
        let selected = app.element(rows.firstMatch.identifier)
        selected.click()
        app.scrollViews.firstMatch.scroll(byDeltaX: 0, deltaY: -5000)
        XCTAssertTrue(waitForHittable(selected, isHittable: false), "Scrolling did not move the selection out of view")

        keyStroke(.return, modifiers: [.control])
        XCTAssertTrue(app.menuItems["Show evolution…"].waitForExistence(timeout: 3), "Control-Return did not open the menu for the scrolled-out selection")
        XCTAssertTrue(selected.isHittable, "The menu opened without revealing the selected change")
        keyStroke(.escape)
    }
}
