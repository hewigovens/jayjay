import XCTest

final class ContextExpansionScene: SceneBase {
    override class var fixtureName: String {
        "context-expansion"
    }

    func testRevealCollapsedContextIncrementallyAndAllAtOnce() throws {
        let app = try XCTUnwrap(app)
        XCTAssertTrue(dagRows(of: app).element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        let file = app.descendants(matching: .any)
            .matching(NSPredicate(format: "identifier == %@", AID.FileList.row("context.txt")))
            .firstMatch
        clickCenter(file, timeout: 5, message: "context.txt row missing")

        let showTen = app.links["Show\u{00A0}10"].firstMatch
        XCTAssertTrue(showTen.waitForExistence(timeout: 5), "Collapsed-context Show 10 link missing")
        showTen.click()

        let diffText = app.textViews[AID.Diff.text].firstMatch
        let reducedSeparator = NSPredicate { _, _ in
            (diffText.value as? String ?? "").contains("43 unmodified lines")
        }
        XCTAssertEqual(
            XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: reducedSeparator, object: nil)], timeout: 10),
            .completed,
            "The separator count did not change after revealing 10 lines"
        )

        let showAll = app.links["Show\u{00A0}all"].firstMatch
        XCTAssertTrue(showAll.waitForExistence(timeout: 3), "Collapsed-context Show all link missing")
        showAll.click()
        XCTAssertFalse(
            app.links["Show\u{00A0}all"].firstMatch.waitForExistence(timeout: 3),
            "Show all should remove the expanded region's separator"
        )
    }
}
