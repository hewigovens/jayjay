import XCTest

final class FileDiffScene: SceneBase {
    func testSelectFileShowsDiff() throws {
        let app = try XCTUnwrap(app)
        XCTAssertTrue(dagRows(of: app).element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        // @ in the simple fixture has wip1.txt — select it and expect the diff section to render.
        let file = fileRows(of: app).element(boundBy: 0)
        XCTAssertTrue(file.waitForExistence(timeout: 5), "No file rows in @")
        file.click()

        let diff = app.descendants(matching: .any)[AID.Diff.section]
        XCTAssertTrue(diff.waitForExistence(timeout: 5), "Diff section did not appear")
    }
}
