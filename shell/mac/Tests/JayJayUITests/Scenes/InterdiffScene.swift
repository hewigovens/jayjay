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
        XCTAssertFalse(app.descendants(matching: .any)[AID.Compare.reverseDirection].exists)
        XCTAssertTrue(app.descendants(matching: .any)[AID.Compare.combinedSelection].exists)

        rightClickCenter(second)
        let merge = app.menuItems["Merge 3 selected"]
        XCTAssertTrue(merge.waitForExistence(timeout: 3))
        XCTAssertFalse(
            merge.isEnabled,
            "Linear changes cannot be merged as independent heads"
        )
        XCTAssertFalse(app.menuItems["New change on top"].exists)
        XCTAssertFalse(app.menuItems["Create bookmark here..."].exists)
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

    func testCommandClickNonConsecutiveRowsComparesOutermostSelection() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        let first = rows.element(boundBy: 0)
        let skipped = rows.element(boundBy: 1)
        let third = rows.element(boundBy: 2)
        XCTAssertTrue(first.waitForExistence(timeout: 10), "DAG never populated")

        first.click()
        XCUIElement.perform(withKeyModifiers: .command) {
            third.click()
        }

        let compareBanner = app.descendants(matching: .any)[AID.Compare.banner]
        XCTAssertTrue(
            compareBanner.waitForExistence(timeout: 5),
            "Non-consecutive selection did not load its outermost comparison"
        )
        XCTAssertTrue(first.isSelected)
        XCTAssertFalse(skipped.isSelected)
        XCTAssertTrue(third.isSelected)
        XCTAssertTrue(app.descendants(matching: .any)[AID.Compare.reverseDirection].isEnabled)
        XCTAssertFalse(app.descendants(matching: .any)[AID.Detail.selectionWithoutDiff].exists)

        app.typeKey(.escape, modifierFlags: [])
        XCTAssertTrue(
            compareBanner.waitForNonExistence(timeout: 5),
            "Escape did not collapse the multi-selection"
        )
        XCTAssertFalse(first.isSelected)
        XCTAssertTrue(third.isSelected, "Escape should preserve the active change")
    }
}
