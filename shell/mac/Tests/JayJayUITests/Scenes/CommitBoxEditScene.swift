import XCTest

final class CommitBoxEditScene: SceneBase {
    override class var fixtureName: String {
        "edit-description"
    }

    func testEditReplacesStaleDraftWithCommitDescription() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        // Stale draft typed for the current @.
        let summary = app.textFields[AID.CommitBox.summary]
        XCTAssertTrue(summary.waitForExistence(timeout: 5), "CommitBox summary field not found")
        summary.click()
        paste("stale draft")

        // Context-menu edit of @'s parent, which is described "add feature".
        rows.element(boundBy: 1).rightClick()
        let edit = app.menuItems["Edit (modify this change)"]
        XCTAssertTrue(edit.waitForExistence(timeout: 3), "Edit menu item missing")
        edit.click()

        // The box must show the edited commit's description, not the stale draft (issue #101).
        let seeded = NSPredicate { _, _ in
            (summary.value as? String ?? "") == "add feature"
        }
        XCTAssertEqual(
            XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: seeded, object: nil)], timeout: 10),
            .completed,
            "CommitBox kept the stale draft instead of loading the edited commit's description"
        )
    }
}
