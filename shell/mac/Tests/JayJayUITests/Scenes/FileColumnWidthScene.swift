import XCTest

final class FileColumnWidthScene: SceneBase {
    override class var startsWithDefaultLayout: Bool {
        false
    }

    func testFileColumnWidthSurvivesChangeSwitchAndRelaunch() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.firstMatch.waitForExistence(timeout: 10), "DAG never populated")
        let column = app.descendants(matching: .any)[AID.FileList.column]
        XCTAssertTrue(column.waitForExistence(timeout: 10), "File column missing")
        let initial = column.frame.width

        let divider = app.descendants(matching: .any)[AID.FileList.columnDivider]
        XCTAssertTrue(divider.waitForExistence(timeout: 5), "File column divider missing")
        let grip = divider.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.5))
        grip.press(forDuration: 0.2, thenDragTo: grip.withOffset(CGVector(dx: -30, dy: 0)))
        let resized = column.frame.width
        XCTAssertEqual(resized, max(220, initial - 30), accuracy: 4, "Dragging the divider did not resize the file column")

        rows.element(boundBy: 1).click()
        XCTAssertTrue(fileRows(of: app).firstMatch.waitForExistence(timeout: 10), "Second change never loaded its files")
        XCTAssertEqual(column.frame.width, resized, accuracy: 2, "Switching changes altered the file column width")

        app.terminate()
        app.launch()
        let relaunched = app.descendants(matching: .any)[AID.FileList.column]
        XCTAssertTrue(relaunched.waitForExistence(timeout: 10), "File column missing after relaunch")
        XCTAssertEqual(relaunched.frame.width, resized, accuracy: 2, "File column width did not persist")
    }
}
