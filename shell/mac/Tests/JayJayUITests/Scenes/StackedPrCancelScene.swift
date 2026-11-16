import XCTest

/// Opens the stacked-PR panel from a change's context menu, confirms the preview
/// renders, and cancels — so the flow is exercised without pushing or creating
/// any PRs (read-only; the fixture has a supported forge remote but no network).
final class StackedPrCancelScene: SceneBase {
    override class var fixtureName: String {
        "stacked-pr"
    }

    func testOpenStackedPrPanelThenCancel() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        // Open "Create / Update Stacked PRs…" from the tip change's context menu.
        rightClickCenter(rows.element(boundBy: 0))
        let menuItem = app.menuItems["Create / Update Stacked PRs…"]
        if !menuItem.waitForExistence(timeout: 3) {
            // The forge capability arrives with the first full refresh, after the rows.
            keyStroke(.escape)
            rightClickCenter(rows.element(boundBy: 0))
        }
        XCTAssertTrue(menuItem.waitForExistence(timeout: 3), "Stacked PRs menu item missing")
        menuItem.click()

        // The preview renders: header plus a Cancel button (Submit not tapped).
        let title = app.staticTexts["Stacked Pull Requests"]
        XCTAssertTrue(title.waitForExistence(timeout: 10), "Stacked PRs panel did not appear")
        let cancel = app.buttons["Cancel"]
        XCTAssertTrue(cancel.waitForExistence(timeout: 5), "Cancel button missing")

        // Cancel dismisses the panel without submitting.
        cancel.click()
        XCTAssertTrue(title.waitForNonExistence(timeout: 5), "Stacked PRs panel did not dismiss")
    }
}
