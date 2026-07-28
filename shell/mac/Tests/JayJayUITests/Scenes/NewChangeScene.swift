import XCTest

final class NewChangeScene: SceneBase {
    override class var fixtureName: String {
        "new-change"
    }

    func testContextMenuNewChangeClearsDraft() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        let originalTopId = rows.element(boundBy: 0).identifier
        let summary = app.textFields[AID.CommitBox.summary]
        XCTAssertTrue(summary.waitForExistence(timeout: 5), "CommitBox summary field not found")
        let oldSummary = "old working-copy summary"
        summary.click()
        paste(oldSummary)

        let details = app.textViews[AID.CommitBox.draft]
        XCTAssertTrue(details.waitForExistence(timeout: 5), "CommitBox description editor not found")
        details.click()
        paste("old working-copy body")

        // Right-click the parent of @ and create a new change on top of it.
        rows.element(boundBy: 1).rightClick()
        let newChange = app.menuItems["New change on top"]
        XCTAssertTrue(newChange.waitForExistence(timeout: 3), "\"New change on top\" menu item missing")
        newChange.click()

        // The new @ should have a different change-id and an empty commit box.
        let predicate = NSPredicate { _, _ in
            let current = rows.element(boundBy: 0).identifier
            let currentSummary = summary.value as? String ?? ""
            let currentDetails = details.value as? String ?? ""
            return !current.isEmpty
                && current != originalTopId
                && currentSummary != oldSummary
                && currentDetails.isEmpty
        }
        let expectation = XCTNSPredicateExpectation(predicate: predicate, object: nil)
        XCTAssertEqual(
            XCTWaiter().wait(for: [expectation], timeout: 10),
            .completed,
            "New working copy carried the previous change's commit message"
        )
    }
}
