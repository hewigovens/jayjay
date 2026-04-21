import XCTest

final class AnnotateScene: SceneBase {
    func testRightClickAnnotate() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        // Parent of @ in the simple fixture is "add feature" — one modified file.
        rows.element(boundBy: 1).click()

        let fileRow = fileRows(of: app).element(boundBy: 0)
        XCTAssertTrue(fileRow.waitForExistence(timeout: 5), "No file rows after selecting commit")

        fileRow.rightClick()
        let annotate = app.menuItems["Annotate (Blame)"]
        XCTAssertTrue(annotate.waitForExistence(timeout: 3), "Annotate menu item missing")
        annotate.click()
    }
}
