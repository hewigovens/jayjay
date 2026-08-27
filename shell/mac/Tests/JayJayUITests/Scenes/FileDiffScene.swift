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

        let column = app.descendants(matching: .any)[AID.FileList.column]
        XCTAssertTrue(column.waitForExistence(timeout: 5), "File column missing")
        XCTAssertEqual(file.frame.minX - column.frame.minX, 4, accuracy: 1, "Selected row should reach the column's leading edge")
        XCTAssertEqual(column.frame.maxX - file.frame.maxX, 4, accuracy: 1.5, "Selected row should reach the column's trailing edge")
    }
}
