import XCTest

final class ParallelizeSelectionScene: SceneBase {
    override class var fixtureName: String {
        "parallelize"
    }

    func testParallelizeMenuReportsNothingToDoAndRewritesAConnectedRun() throws {
        let app = try XCTUnwrap(app)
        let rows = dagRows(of: app)
        XCTAssertTrue(rows.element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        select([0, 2], rows: rows, step: "no-op selection")
        rightClickCenter(rows.element(boundBy: 0))
        let noOpItem = app.menuItems["Parallelize 2 selected"]
        let menuLabels = app.menuItems.allElementsBoundByIndex.map(\.label)
        XCTAssertTrue(noOpItem.waitForExistence(timeout: 5), "parallelize menu item missing; menu=\(menuLabels)")
        XCTAssertTrue(noOpItem.isEnabled)
        noOpItem.click()
        XCTAssertTrue(
            app.staticTexts["Nothing to parallelize. The selection would keep its current parents."]
                .waitForExistence(timeout: 10),
            "a no-op must be reported instead of silently doing nothing"
        )

        let changeIds = (0 ..< 3).map { rows.element(boundBy: $0).identifier }
        select([0, 1, 2], rows: rows, step: "run selection")
        rightClickCenter(rows.element(boundBy: 1))
        let parallelize = app.menuItems["Parallelize 3 selected"]
        XCTAssertTrue(parallelize.waitForExistence(timeout: 5), "parallelize menu item missing")
        XCTAssertTrue(parallelize.isEnabled)
        parallelize.click()

        let collapsed = NSPredicate { _, _ in
            rows.allElementsBoundByIndex.filter(\.isSelected).count == 1
        }
        let reloaded = XCTWaiter().wait(
            for: [XCTNSPredicateExpectation(predicate: collapsed, object: nil)],
            timeout: 10
        )
        let idsAfter = rows.allElementsBoundByIndex.map(\.identifier)
        XCTAssertEqual(reloaded, .completed, "parallelize did not reload the graph: \(idsAfter)")
        XCTAssertTrue(
            changeIds.allSatisfy(idsAfter.contains),
            "parallelized changes disappeared from the graph: \(idsAfter)"
        )

        select([0, 2], rows: rows, step: "sibling selection")
        rightClickCenter(rows.element(boundBy: 0))
        let merge = app.menuItems["Merge 2 selected"]
        XCTAssertTrue(merge.waitForExistence(timeout: 5), "merge menu item missing")
        XCTAssertTrue(merge.isEnabled, "parallelized changes should be independent heads")
        let stillChained = app.menuItems["Parallelize 2 selected"]
        XCTAssertTrue(stillChained.waitForExistence(timeout: 5))
        XCTAssertFalse(stillChained.isEnabled, "siblings must not stay parallelizable")
    }

    /// Clicks go through each row's center because a rewrite can leave its own hit point clipped.
    private func select(_ indices: [Int], rows: XCUIElementQuery, step: String) {
        let primary = rows.element(boundBy: indices[0])
        clickCenter(primary)
        // The plain click's selection must land before a command-click toggles against it.
        let primarySelected = NSPredicate { _, _ in primary.isSelected }
        XCTAssertEqual(
            XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: primarySelected, object: nil)], timeout: 5),
            .completed,
            "\(step): the plain click did not select its row"
        )
        XCUIElement.perform(withKeyModifiers: .command) {
            for index in indices.dropFirst() {
                clickCenter(rows.element(boundBy: index))
            }
        }
        let settled = NSPredicate { _, _ in
            indices.allSatisfy { rows.element(boundBy: $0).isSelected }
        }
        let expectation = XCTNSPredicateExpectation(predicate: settled, object: nil)
        let result = XCTWaiter().wait(for: [expectation], timeout: 5)
        let observed = indices.map { index in
            let row = rows.element(boundBy: index)
            return "\(index):\(row.identifier) selected=\(row.isSelected)"
        }
        XCTAssertEqual(result, .completed, "\(step) did not settle: \(observed)")
    }
}
