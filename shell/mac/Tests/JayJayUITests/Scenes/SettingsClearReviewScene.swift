import XCTest

final class SettingsClearReviewScene: SceneBase {
    func testClearingReviewDataFromSettingsUnmarksOpenWindows() throws {
        let app = try XCTUnwrap(app)
        XCTAssertTrue(dagRows(of: app).element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")
        let review = app.buttons[AID.FileList.review("wip1.txt")]
        XCTAssertTrue(review.waitForExistence(timeout: 5), "File review control missing")
        XCTAssertEqual(review.label, "Unreviewed")
        review.click()
        XCTAssertTrue(review.wait(for: \.label, toEqual: "Reviewed", timeout: 5), "file did not become reviewed")

        app.typeKey(",", modifierFlags: .command)
        let diffTab = app.descendants(matching: .any)
            .matching(NSPredicate(format: "label == 'Diff'"))
            .firstMatch
        XCTAssertTrue(diffTab.waitForExistence(timeout: 5), "Diff settings tab missing")
        diffTab.click()
        let clear = app.buttons[AID.Settings.clearReviewData]
        XCTAssertTrue(clear.waitForExistence(timeout: 5), "Clear button missing")
        clear.click()
        // app-wide "Clear" also matches the Touch Bar item, which XCUITest cannot click.
        let confirm = app.sheets.buttons["Clear"].firstMatch
        XCTAssertTrue(confirm.waitForExistence(timeout: 5), "Confirmation missing")
        confirm.click()

        XCTAssertTrue(review.wait(for: \.label, toEqual: "Unreviewed", timeout: 10), "open window kept stale review state")
    }
}
