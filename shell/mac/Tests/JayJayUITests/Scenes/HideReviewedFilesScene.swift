import XCTest

final class HideReviewedFilesScene: SceneBase {
    func testFilteringKeepsSelectionAndReviewActionsOnVisibleFiles() throws {
        let app = try XCTUnwrap(app)
        let first = fileRows(of: app).matching(identifier: AID.FileList.row("wip1.txt")).firstMatch
        let second = fileRows(of: app).matching(identifier: AID.FileList.row("wip2.txt")).firstMatch
        let review = app.buttons[AID.FileList.review("wip1.txt")]
        XCTAssertTrue(review.waitForExistence(timeout: 10))
        review.click()
        XCTAssertTrue(review.wait(for: \.label, toEqual: "Reviewed", timeout: 5))
        second.click()
        XCUIElement.perform(withKeyModifiers: .command) { first.click() }

        keyStroke(",", modifiers: .command)
        selectSettingsPage("diff", in: app)
        let hide = settingsWindow(in: app).descendants(matching: .any)[AID.Settings.hideReviewedFiles].firstMatch
        XCTAssertTrue(hide.waitForExistence(timeout: 5))
        hide.click()
        keyStroke("w", modifiers: .command)
        XCTAssertTrue(first.waitForNonExistence(timeout: 5))
        XCTAssertTrue(second.isSelected)

        app.terminate()
        let argument = try XCTUnwrap(app.launchArguments.firstIndex(of: "-jayjay.hideReviewedFiles"))
        app.launchArguments[argument + 1] = "YES"
        app.launch()
        XCTAssertTrue(second.waitForExistence(timeout: 10))
        XCTAssertTrue(second.wait(for: \.isSelected, toEqual: true, timeout: 5))
        XCTAssertFalse(first.exists)
        second.click()
        keyStroke(.space)
        XCTAssertTrue(second.waitForNonExistence(timeout: 5))

        keyStroke(",", modifiers: .command)
        selectSettingsPage("diff", in: app)
        hide.click()
        keyStroke("w", modifiers: .command)
        XCTAssertTrue(review.waitForExistence(timeout: 5))
        XCTAssertEqual(review.label, "Reviewed")
        XCTAssertEqual(app.buttons[AID.FileList.review("wip2.txt")].label, "Reviewed")
    }
}
