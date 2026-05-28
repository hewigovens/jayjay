import XCTest

final class BookmarkDiffScene: SceneBase {
    override class var fixtureName: String { "simple-bookmark-diff" }

    func testDiffBookmarkFromDagContextMenu() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        let baseBookmark = app.staticTexts["main"].firstMatch
        XCTAssertTrue(baseBookmark.waitForExistence(timeout: 5), "Base bookmark row did not appear")
        baseBookmark.click()
        XCTAssertTrue(
            app.descendants(matching: .any)[AID.FileList.row("feature.txt")].waitForExistence(timeout: 5),
            "Base bookmark row did not load"
        )

        rows.element(boundBy: 0).rightClick()
        let diffBookmark = app.menuItems["Diff Bookmark"].firstMatch
        XCTAssertTrue(diffBookmark.waitForExistence(timeout: 5), "Diff Bookmark menu item did not appear")
        diffBookmark.click()

        let banner = app.descendants(matching: .any)[AID.Compare.banner]
        XCTAssertTrue(banner.waitForExistence(timeout: 5), "Compare banner did not appear")
        XCTAssertTrue(app.staticTexts["PR Diff"].exists)
        XCTAssertTrue(app.staticTexts["main"].exists)
        XCTAssertTrue(app.staticTexts["bookmark-diff"].exists)

        let diff = app.descendants(matching: .any)[AID.Diff.section]
        XCTAssertTrue(diff.waitForExistence(timeout: 5), "Bookmark diff did not render")
    }
}
