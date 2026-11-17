import XCTest

final class ReviewModeScene: SceneBase {
    func testReviewModeShowsMarksPerChangeOutsideTheWorkingCopy() throws {
        let app = try XCTUnwrap(app)
        let rows = try waitForDAGRows()
        let toggle = app.element(AID.FileList.reviewMode)
        let isOn = NSPredicate(format: "value == 1")
        let isOff = NSPredicate(format: "value == 0")
        rows.firstMatch.click()
        XCTAssertTrue(app.buttons[AID.FileList.review("wip1.txt")].waitForExistence(timeout: 10), "The working copy lost its review circles")
        XCTAssertFalse(toggle.exists, "The working copy offers a review mode toggle")

        let featureChange = rows.matching(NSPredicate(format: "value CONTAINS %@", "add feature")).firstMatch
        featureChange.click()
        let featureRow = app.element(AID.FileList.row("feature.txt"))
        let featureReview = app.buttons[AID.FileList.review("feature.txt")]
        XCTAssertTrue(featureRow.waitForExistence(timeout: 10))
        XCTAssertTrue(toggle.waitForExistence(timeout: 5))
        XCTAssertTrue(isOff.evaluate(with: toggle), "Review mode started on for a change without marks")
        XCTAssertTrue(app.staticTexts["1 file"].exists, "The file count did not stay plain with review mode off")
        XCTAssertFalse(featureReview.exists, "Review circles showed with review mode off")
        XCTAssertFalse(app.buttons[AID.Diff.markReviewed].exists, "Mark Reviewed showed with review mode off")
        featureRow.click()
        keyStroke(.space)

        toggle.click()
        XCTAssertTrue(featureReview.waitForExistence(timeout: 5))
        XCTAssertTrue(isOn.evaluate(with: toggle))
        XCTAssertEqual(featureReview.label, "Unreviewed", "Space marked a file with review mode off")
        XCTAssertTrue(app.staticTexts["0/1 reviewed"].waitForExistence(timeout: 5), "Review mode did not show progress")
        featureReview.click()
        XCTAssertTrue(app.staticTexts["1/1 reviewed"].waitForExistence(timeout: 5))

        rows.matching(NSPredicate(format: "value CONTAINS %@", "add hello")).firstMatch.click()
        XCTAssertTrue(app.element(AID.FileList.row("hello.txt")).waitForExistence(timeout: 10))
        XCTAssertTrue(app.staticTexts["1 file"].waitForExistence(timeout: 5), "Review mode leaked to another change")
        XCTAssertTrue(isOff.evaluate(with: toggle))
        XCTAssertFalse(app.buttons[AID.FileList.review("hello.txt")].exists)

        featureChange.click()
        XCTAssertTrue(app.staticTexts["1/1 reviewed"].waitForExistence(timeout: 10), "The change did not keep its review mode")
        app.menuBars.menuBarItems["Repository"].click()
        let menuItem = app.menuItems["Review Mode"]
        XCTAssertTrue(menuItem.waitForExistence(timeout: 3), "The Repository menu did not offer Review Mode")
        menuItem.click()
        XCTAssertTrue(featureReview.waitForNonExistence(timeout: 5))
        XCTAssertTrue(isOff.evaluate(with: toggle))
    }
}
