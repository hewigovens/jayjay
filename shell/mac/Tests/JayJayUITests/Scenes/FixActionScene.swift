import XCTest

final class FixActionScene: SceneBase {
    override class var fixtureName: String {
        "fix"
    }

    func testFixRewritesTheChangeAndItsDescendantsAndReportsTheSummary() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        rightClickCenter(rows.element(boundBy: 1))
        let fix = app.menuItems["Run formatters (jj fix)"]
        let menuLabels = app.menuItems.allElementsBoundByIndex.map(\.label)
        XCTAssertTrue(fix.waitForExistence(timeout: 5), "Run formatters menu item missing; menu=\(menuLabels)")
        XCTAssertTrue(fix.isEnabled, "a repo with [fix.tools] must enable Fix")
        fix.click()

        XCTAssertTrue(
            app.staticTexts["Formatters rewrote 2 of 2 changes"].waitForExistence(timeout: 15),
            "fix must report how many changes it rewrote"
        )
    }
}
