import XCTest

final class CommitBoxScene: SceneBase {
    // Dedicated fixture: this scene runs `jj commit`, which mutates the repo.
    override class var fixtureName: String { "simple-commit" }

    func testCommitClearsDraft() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        let originalTopId = rows.element(boundBy: 0).identifier

        let editor = app.textViews[AID.CommitBox.draft]
        XCTAssertTrue(editor.waitForExistence(timeout: 5), "CommitBox text editor not found")
        editor.click()
        editor.typeText("regression: clear draft after commit")

        let commitButton = app.buttons[AID.CommitBox.commit]
        XCTAssertTrue(commitButton.waitForExistence(timeout: 3), "Commit button not found")
        commitButton.click()

        let topRowChanged = NSPredicate { _, _ in
            let current = rows.element(boundBy: 0).identifier
            return !current.isEmpty && current != originalTopId
        }
        XCTAssertEqual(
            XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: topRowChanged, object: nil)], timeout: 10),
            .completed,
            "@ did not advance after commit"
        )

        let draftCleared = NSPredicate { _, _ in
            (editor.value as? String ?? "").isEmpty
        }
        XCTAssertEqual(
            XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: draftCleared, object: nil)], timeout: 5),
            .completed,
            "CommitBox draft was not cleared after commit"
        )
    }
}
