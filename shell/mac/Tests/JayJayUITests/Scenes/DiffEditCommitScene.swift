import XCTest

final class DiffEditCommitScene: SceneBase {
    override class var fixtureName: String {
        "diff-edit-commit"
    }

    func testCommitSelectedSplitsTheWorkingCopy() throws {
        let app = try XCTUnwrap(app)
        let open = app.buttons[AID.DiffEdit.open]
        XCTAssertTrue(open.waitForExistence(timeout: 10))
        open.click()

        XCTAssertTrue(app.buttons["Deselect All"].waitForExistence(timeout: 10), "The working copy did not start fully selected")
        app.buttons[AID.DiffEdit.description].click()
        let save = app.buttons["Save"]
        XCTAssertTrue(save.waitForExistence(timeout: 5))
        paste("commit both wip files")
        save.click()

        let commit = app.buttons["Commit Selected"]
        XCTAssertTrue(commit.waitForExistence(timeout: 5))
        capture(app.windows.firstMatch, name: "diff-edit-working-copy")
        commit.click()

        let committed = dagRows(of: app).matching(NSPredicate(format: "value CONTAINS 'commit both wip files'"))
        XCTAssertTrue(committed.firstMatch.waitForExistence(timeout: 10), "The committed change did not appear in the DAG")
    }
}
