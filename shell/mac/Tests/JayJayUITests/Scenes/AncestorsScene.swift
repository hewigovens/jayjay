import XCTest

final class AncestorsScene: SceneBase {
    func testShowAncestorsKeepsTargetAndReturnsToPreviousFilter() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        let target = rows.matching(NSPredicate(format: "value CONTAINS %@", "add hello")).firstMatch
        XCTAssertTrue(target.waitForExistence(timeout: 10))
        let newer = rows.matching(NSPredicate(format: "value CONTAINS %@", "add feature")).firstMatch
        XCTAssertTrue(newer.exists)

        rightClickCenter(target)
        app.menuItems["Show ancestors…"].click()
        XCTAssertTrue(newer.waitForNonExistence(timeout: 10))
        XCTAssertTrue(target.isSelected)
        XCTAssertTrue(rows.matching(NSPredicate(format: "value CONTAINS %@", "initial")).firstMatch.exists)
        let filter = app.textFields["Revset expression"]
        XCTAssertTrue(filter.exists)
        XCTAssertTrue((filter.value as? String)?.hasPrefix("::commit_id(") == true)

        let screenshot = XCTAttachment(screenshot: app.windows.firstMatch.screenshot())
        screenshot.name = "Show ancestors filter"
        screenshot.lifetime = .keepAlways
        add(screenshot)

        app.buttons["Back to previous filter"].click()
        XCTAssertTrue(newer.waitForExistence(timeout: 10))
        XCTAssertFalse(app.buttons["Back to previous filter"].exists)
    }
}
