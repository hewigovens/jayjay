import XCTest

final class InterdiffScene: SceneBase {
    func testShiftClickSelectsContinuousRange() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        let first = rows.element(boundBy: 0)
        let second = rows.element(boundBy: 1)
        let third = rows.element(boundBy: 2)
        let combinedDiffParent = rows.element(boundBy: 3)
        XCTAssertTrue(first.waitForExistence(timeout: 10), "DAG never populated")

        first.click()
        XCUIElement.perform(withKeyModifiers: .shift) {
            third.click()
        }

        XCTAssertTrue(
            app.staticTexts["3 Changes Selected"].waitForExistence(timeout: 5),
            "Combined-diff summary did not appear"
        )
        XCTAssertTrue(first.isSelected)
        XCTAssertTrue(second.isSelected)
        XCTAssertTrue(third.isSelected)
        XCTAssertFalse(combinedDiffParent.isSelected)
        XCTAssertFalse(app.element(AID.Compare.reverseDirection).exists)
        XCTAssertTrue(app.element(AID.Compare.combinedSelection).exists)

        rightClickCenter(second)
        let merge = app.windows.menuItems["Merge 3 selected"]
        XCTAssertTrue(merge.waitForExistence(timeout: 3))
        XCTAssertFalse(
            merge.isEnabled,
            "Linear changes cannot be merged as independent heads"
        )
        XCTAssertFalse(app.windows.menuItems["New change on top"].exists)
        XCTAssertFalse(app.windows.menuItems["Create bookmark here…"].exists)
        app.typeKey(.escape, modifierFlags: [])

        app.typeKey(.escape, modifierFlags: [])
        XCTAssertTrue(
            app.staticTexts["3 Changes Selected"].waitForNonExistence(timeout: 5),
            "Escape did not collapse the multi-selection"
        )
        XCTAssertFalse(first.isSelected)
        XCTAssertFalse(second.isSelected)
        XCTAssertTrue(third.isSelected)
    }
}
