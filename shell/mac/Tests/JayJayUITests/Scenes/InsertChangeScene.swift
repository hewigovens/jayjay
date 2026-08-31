import XCTest

final class InsertChangeScene: SceneBase {
    override class var fixtureName: String {
        "insert-change"
    }

    func testContextMenuNewChangeBeforeInsertsTheWorkingCopyUnderTheTarget() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")
        let originalIds = rows.allElementsBoundByIndex.map(\.identifier)
        let originalTopId = try XCTUnwrap(originalIds.first)

        rows.element(boundBy: 0).rightClick()
        let insertBefore = app.menuItems["New change before"]
        XCTAssertTrue(insertBefore.waitForExistence(timeout: 3), "\"New change before\" menu item missing")
        insertBefore.click()

        let predicate = NSPredicate { _, _ in
            let rowsNow = rows.allElementsBoundByIndex
            let ids = rowsNow.map(\.identifier)
            return ids.count == originalIds.count + 1
                && ids[0] == originalTopId
                && !originalIds.contains(ids[1])
                && rowsNow[1].isSelected
        }
        let expectation = XCTNSPredicateExpectation(predicate: predicate, object: nil)
        let result = XCTWaiter().wait(for: [expectation], timeout: 10)
        let rowsAfter = rows.allElementsBoundByIndex.map { "\($0.identifier) selected=\($0.isSelected)" }
        XCTAssertEqual(result, .completed, "Inserted change did not appear selected under the previous working copy: \(rowsAfter)")
    }
}
