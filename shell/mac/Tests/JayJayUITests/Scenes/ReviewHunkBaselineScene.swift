import XCTest

final class ReviewHunkBaselineScene: SceneBase {
    func testFileReviewToggleUpdatesAccessibilityState() throws {
        let app = try XCTUnwrap(app)
        XCTAssertTrue(dagRows(of: app).element(boundBy: 0).waitForExistence(timeout: 10), "DAG never populated")

        let file = app.descendants(matching: .any)
            .matching(NSPredicate(format: "identifier == %@", AID.FileList.row("wip1.txt")))
            .firstMatch
        XCTAssertTrue(file.waitForExistence(timeout: 5), "wip1.txt row missing")
        file.click()

        let review = app.buttons[AID.FileList.review("wip1.txt")]
        XCTAssertTrue(review.waitForExistence(timeout: 5), "File review control missing")
        XCTAssertEqual(review.label, "Unreviewed")

        review.click()
        let reviewed = NSPredicate { _, _ in
            review.label == "Reviewed"
        }
        XCTAssertEqual(
            XCTWaiter().wait(for: [XCTNSPredicateExpectation(predicate: reviewed, object: nil)], timeout: 5),
            .completed,
            "File review control did not become reviewed"
        )
    }
}
