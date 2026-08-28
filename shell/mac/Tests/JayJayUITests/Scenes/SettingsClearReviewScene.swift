import XCTest

final class SettingsClearReviewScene: SceneBase {
    func testClearingReviewDataFromSettingsUnmarksOpenWindows() throws {
        let app = try XCTUnwrap(app)
        XCTAssertTrue(dagRows(of: app).element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")
        let review = app.buttons[AID.FileList.review("wip1.txt")]
        XCTAssertTrue(review.waitForExistence(timeout: 5), "File review control missing")
        review.click()
        XCTAssertTrue(review.wait(for: \.label, toEqual: "Reviewed", timeout: 5), "file did not become reviewed")

        app.typeKey(",", modifierFlags: .command)
        let settingsWindow = app.windows.matching(NSPredicate(format: "title CONTAINS 'Settings'")).firstMatch
        XCTAssertTrue(settingsWindow.waitForExistence(timeout: 5), "Settings window did not open")
        let diffTab = settingsWindow.descendants(matching: .any)
            .matching(NSPredicate(format: "label == 'Diff'"))
            .firstMatch
        XCTAssertTrue(diffTab.waitForExistence(timeout: 5), "Diff settings tab missing")
        diffTab.click()
        let clear = settingsWindow.buttons[AID.Settings.clearReviewData]
        XCTAssertTrue(clear.waitForExistence(timeout: 5), "Clear button missing")
        clear.click()
        let confirm = app.buttons["Clear"].firstMatch
        XCTAssertTrue(confirm.waitForExistence(timeout: 5), "Confirmation missing")
        confirm.click()
        settingsWindow.typeKey(.escape, modifierFlags: [])

        XCTAssertTrue(review.wait(for: \.label, toEqual: "Unreviewed", timeout: 10), "open window kept stale review state")
    }
}
