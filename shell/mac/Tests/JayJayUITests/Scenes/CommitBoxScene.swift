import XCTest

final class CommitBoxScene: SceneBase {
    override class var fixtureName: String {
        "commit"
    }

    func testCommitClearsDraft() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        let originalTopId = rows.element(boundBy: 0).identifier

        // Summary is required: the Commit button is disabled while it is empty.
        let summary = app.textFields[AID.CommitBox.summary]
        XCTAssertTrue(summary.waitForExistence(timeout: 5), "CommitBox summary field not found")
        let typedSummary = "regression: clear draft after commit"
        summary.click()
        paste(typedSummary)

        let details = app.textViews[AID.CommitBox.draft]
        XCTAssertTrue(details.waitForExistence(timeout: 5), "CommitBox description editor not found")
        details.click()
        paste("body of the change")

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

        // Both fields reset after commit: the summary no longer holds the typed
        // text and the description editor goes empty.
        let draftCleared = NSPredicate { _, _ in
            (summary.value as? String ?? "") != typedSummary && (details.value as? String ?? "").isEmpty
        }
        XCTAssertEqual(
            XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: draftCleared, object: nil)], timeout: 5),
            .completed,
            "CommitBox draft was not cleared after commit"
        )
    }
}
