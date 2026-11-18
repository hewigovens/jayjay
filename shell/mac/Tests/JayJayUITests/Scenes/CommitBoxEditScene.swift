import XCTest

final class CommitBoxEditScene: SceneBase {
    override class var fixtureName: String {
        "edit-description"
    }

    func testEditingDescribedChangeHidesPreviousWorkingCopyDraft() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        let summary = app.textFields[AID.CommitBox.summary]
        XCTAssertTrue(summary.waitForExistence(timeout: 5), "CommitBox summary field not found")
        summary.click()
        paste("stale draft")

        let parent = rows.matching(NSPredicate(format: "value CONTAINS %@", "add feature")).firstMatch
        parent.rightClick()
        let edit = app.windows.menuItems["Edit Change"]
        XCTAssertTrue(edit.waitForExistence(timeout: 3), "Edit menu item missing")
        edit.click()

        let edited = rows.matching(NSPredicate(format: "value CONTAINS %@ AND value CONTAINS %@", "add feature", "working copy")).firstMatch
        XCTAssertTrue(edited.waitForExistence(timeout: 10), "Edit Change did not make the described parent the working copy")
        XCTAssertTrue(summary.waitForNonExistence(timeout: 5), "The sidebar still shows the old draft after Edit Change")
    }
}
