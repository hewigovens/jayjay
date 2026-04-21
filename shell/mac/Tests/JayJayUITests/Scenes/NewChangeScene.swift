import XCTest

final class NewChangeScene: SceneBase {
    // Dedicated fixture so the mutation doesn't leak into ReviewSplitScene etc.
    override class var fixtureName: String { "simple-newchange" }

    func testContextMenuNewChange() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        let originalTopId = rows.element(boundBy: 0).identifier

        // Right-click the parent of @ and create a new change on top of it.
        rows.element(boundBy: 1).rightClick()
        let newChange = app.menuItems["New change on top"]
        XCTAssertTrue(newChange.waitForExistence(timeout: 3), "\"New change on top\" menu item missing")
        newChange.click()

        // The new @ should have a different change-id than the previous top row.
        let predicate = NSPredicate { _, _ in
            let current = rows.element(boundBy: 0).identifier
            return !current.isEmpty && current != originalTopId
        }
        let expectation = XCTNSPredicateExpectation(predicate: predicate, object: nil)
        XCTAssertEqual(XCTWaiter().wait(for: [expectation], timeout: 10), .completed, "@ did not change after new-change action")
    }
}
