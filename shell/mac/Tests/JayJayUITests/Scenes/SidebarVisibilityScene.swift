import XCTest

final class SidebarVisibilityScene: SceneBase {
    func testTogglePreservesFileSelectionAndSidebarWidth() throws {
        let app = try XCTUnwrap(app)
        let divider = app.descendants(matching: .any)[AID.Sidebar.divider]
        let column = app.descendants(matching: .any)[AID.FileList.column]
        let toggle = app.buttons[AID.Toolbar.sidebarToggle]
        XCTAssertTrue(divider.waitForExistence(timeout: 10))
        XCTAssertTrue(column.waitForExistence(timeout: 10))
        let sidebarEdge = divider.frame.minX
        let selectedFile = fileRows(of: app).firstMatch
        XCTAssertTrue(selectedFile.waitForExistence(timeout: 10))
        selectedFile.click()
        let diffHeader = app.staticTexts[AID.Diff.section].firstMatch
        XCTAssertTrue(diffHeader.waitForExistence(timeout: 10))
        let selectedPath = diffHeader.label

        toggle.click()
        XCTAssertTrue(divider.waitForNonExistence(timeout: 5))
        XCTAssertTrue(column.exists)
        XCTAssertLessThan(column.frame.minX, sidebarEdge)
        XCTAssertEqual(diffHeader.label, selectedPath)

        app.typeKey("s", modifierFlags: [.control, .command])
        XCTAssertTrue(divider.waitForExistence(timeout: 5))
        XCTAssertEqual(divider.frame.minX, sidebarEdge, accuracy: 2)
        XCTAssertEqual(diffHeader.label, selectedPath)
    }
}
