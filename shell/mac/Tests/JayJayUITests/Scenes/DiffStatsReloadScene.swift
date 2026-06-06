import XCTest

final class DiffStatsReloadScene: SceneBase {
    // Dedicated fixture: restoring a file mutates @, which must not leak into other scenes.
    override class var fixtureName: String { "simple-diffstats" }

    func testDiffStatsReloadAfterAmend() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        // @ is the top row with two new files (wip1.txt, wip2.txt) → +2 insertions.
        rows.element(boundBy: 0).click()
        XCTAssertTrue(
            stats(app, insertions: 2, deletions: 0).waitForExistence(timeout: 10),
            "initial stats should reflect both new files"
        )

        // Restore one file to the parent: same change-id, new commit-id, one fewer insertion.
        let fileRow = app.descendants(matching: .any)
            .matching(NSPredicate(format: "identifier == 'file.row.wip2.txt'"))
            .firstMatch
        XCTAssertTrue(fileRow.waitForExistence(timeout: 10), "wip2.txt row missing")
        fileRow.rightClick()
        let restore = app.menuItems["Restore to Parent"]
        XCTAssertTrue(restore.waitForExistence(timeout: 3), "\"Restore to Parent\" menu item missing")
        restore.click()

        // Regression: the header must reload to +1, not keep the stale +2 under the same change-id.
        XCTAssertTrue(
            stats(app, insertions: 1, deletions: 0).waitForExistence(timeout: 10),
            "diff stats should reload after @ is amended"
        )
    }

    private func stats(_ app: XCUIApplication, insertions: Int, deletions: Int) -> XCUIElement {
        let identifier = "detail.diffStats.\(insertions).\(deletions)"
        return app.descendants(matching: .any)
            .matching(NSPredicate(format: "identifier == %@", identifier))
            .firstMatch
    }
}
