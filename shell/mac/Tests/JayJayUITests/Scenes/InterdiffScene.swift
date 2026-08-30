import XCTest

final class InterdiffScene: SceneBase {
    func testShiftClickTwoRows() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        let source = rows.element(boundBy: 0)
        let intermediate = rows.element(boundBy: 1)
        source.click()
        XCUIElement.perform(withKeyModifiers: .shift) {
            let target = rows.element(boundBy: 3)
            XCTAssertTrue(target.waitForExistence(timeout: 5), "Comparison target row did not appear")
            target.click()
        }

        let banner = app.descendants(matching: .any)[AID.Compare.banner]
        XCTAssertTrue(banner.waitForExistence(timeout: 5), "Compare banner did not appear")
        let reverse = app.descendants(matching: .any)[AID.Compare.reverseDirection]
        XCTAssertTrue(reverse.exists)
        XCTAssertTrue(reverse.isEnabled)
        XCTAssertTrue(source.isSelected)
        XCTAssertFalse(intermediate.isSelected, "Shift-click should compare endpoints, not select the range")
    }

    func testCommandClickThreeRowsShowsCombinedDiff() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        let first = rows.element(boundBy: 0)
        let second = rows.element(boundBy: 1)
        let third = rows.element(boundBy: 2)
        let combinedDiffParent = rows.element(boundBy: 3)
        XCTAssertTrue(first.waitForExistence(timeout: 10), "DAG never populated")

        first.click()
        XCUIElement.perform(withKeyModifiers: .command) {
            second.click()
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
        let reverse = app.descendants(matching: .any)[AID.Compare.reverseDirection]
        XCTAssertTrue(reverse.exists)
        XCTAssertFalse(reverse.isEnabled)

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
        XCTAssertTrue(first.isSelected)
        XCTAssertFalse(second.isSelected)
        XCTAssertFalse(third.isSelected)
    }

    func testCommandClickNonConsecutiveRowsKeepsSelectionWithoutDiff() throws {
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

        XCTAssertTrue(first.isSelected)
        XCTAssertFalse(skipped.isSelected)
        XCTAssertTrue(third.isSelected)
        let noDiffState = app.descendants(matching: .any)[AID.Detail.nonConsecutiveSelection]
        XCTAssertTrue(
            noDiffState.waitForExistence(timeout: 5),
            "Non-consecutive selection did not show its no-diff state"
        )
        XCTAssertFalse(app.descendants(matching: .any)[AID.Compare.banner].exists)

        app.typeKey(.escape, modifierFlags: [])
        XCTAssertTrue(
            noDiffState.waitForNonExistence(timeout: 5),
            "Escape did not collapse the multi-selection"
        )
        XCTAssertFalse(first.isSelected)
        XCTAssertTrue(third.isSelected, "Escape should preserve the active change")
    }
}
