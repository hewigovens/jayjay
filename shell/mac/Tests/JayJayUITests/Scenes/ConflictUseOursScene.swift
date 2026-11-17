import XCTest

final class ConflictUseOursScene: SceneBase {
    override class var fixtureName: String {
        "conflict-use-ours"
    }

    func testUseOurs() throws {
        let app = try XCTUnwrap(app)
        let useOurs = app.buttons
            .matching(NSPredicate(format: "identifier BEGINSWITH 'conflict.useOurs.'"))
            .firstMatch
        XCTAssertTrue(useOurs.waitForExistence(timeout: 10), "Expected conflict bar from fixture")
        let conflicted = app.buttons[AID.Sidebar.conflictedCount]
        XCTAssertTrue(conflicted.waitForExistence(timeout: 5), "Expected the conflicted count in the change list header")
        XCTAssertEqual(conflicted.label, "1 conflicted")
        conflicted.click()
        let revset = app.buttons[AID.Toolbar.revsetBar]
        let filtered = XCTNSPredicateExpectation(predicate: NSPredicate(format: "value == %@", "conflicts()"), object: revset)
        XCTAssertEqual(XCTWaiter().wait(for: [filtered], timeout: 10), .completed, "The conflicted count did not filter to conflicts()")
        useOurs.click()
        XCTAssertTrue(useOurs.waitForNonExistence(timeout: 10), "Use Ours did not clear the conflict")
        XCTAssertTrue(conflicted.waitForNonExistence(timeout: 10), "The conflicted count outlived the conflict")
        let path = try XCTUnwrap(fixtureURL).appendingPathComponent("conflict.swift")
        let resolved = try String(contentsOf: path, encoding: .utf8)
        XCTAssertTrue(resolved.contains("static let title = \"main build\""))
        XCTAssertTrue(resolved.contains("let retryLimit = 5"))
        XCTAssertFalse(resolved.contains("feature build"))
        XCTAssertFalse(resolved.contains("<<<<<<<"))
    }
}
