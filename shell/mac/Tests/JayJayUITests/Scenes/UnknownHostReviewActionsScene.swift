import XCTest

/// An unknown-host remote keeps ordinary Git actions but offers no forge review actions.
final class UnknownHostReviewActionsScene: SceneBase {
    override class var fixtureName: String {
        "unknown-host"
    }

    func testUnsupportedForgeHidesPullRequestAndStackedPrActions() throws {
        let app = try XCTUnwrap(app)
        let rows = try waitForDAGRows()

        rightClickCenter(rows.element(boundBy: 0))
        XCTAssertTrue(app.menuItems["New Change on Top"].waitForExistence(timeout: 3), "Change menu missing")
        XCTAssertFalse(
            app.menuItems["Create / Update Stacked PRs…"].exists,
            "Unknown-host remotes must not offer stacked PRs"
        )
        keyStroke(.escape)

        keyStroke("b", modifiers: [.command, .shift])
        let title = app.staticTexts["Bookmark Manager"]
        XCTAssertTrue(title.waitForExistence(timeout: 5), "Bookmark Manager did not open")
        let bookmark = app.sheets.staticTexts["feature"]
        XCTAssertTrue(bookmark.waitForExistence(timeout: 5))
        bookmark.rightClick()
        XCTAssertTrue(app.menuItems["Push"].waitForExistence(timeout: 3), "Push must stay available")
        XCTAssertFalse(
            app.menuItems.matching(NSPredicate(format: "title BEGINSWITH 'Pull Request'")).firstMatch.exists,
            "Unknown-host remotes must not offer pull requests"
        )
    }
}
