import XCTest

final class DAGKeyboardNavigationScene: SceneBase {
    override class var fixtureName: String {
        "dag-long"
    }

    func testArrowKeysKeepTheSelectionVisibleAndLoadTheSettledChange() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.firstMatch.waitForExistence(timeout: 10), "DAG never populated")
        rows.firstMatch.click()

        for _ in 0 ..< 15 {
            keyStroke(.downArrow)
        }

        let settledFile = app.descendants(matching: .any)[AID.FileList.row("file-10.txt")]
        XCTAssertTrue(settledFile.waitForExistence(timeout: 10), "Detail did not load the change the keys settled on")
        let selectedRow = rows.matching(NSPredicate(format: "isSelected == YES")).firstMatch
        XCTAssertTrue(selectedRow.exists, "Selected row is not rendered")
        XCTAssertTrue(selectedRow.isHittable, "Selected row scrolled out of view")
    }
}
