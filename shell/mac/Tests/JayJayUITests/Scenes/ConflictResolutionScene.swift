import XCTest

final class ConflictResolutionScene: SceneBase {
    override class var fixtureName: String {
        "conflict"
    }

    func testUseOurs() throws {
        let app = try XCTUnwrap(app)
        let useOurs = app.buttons
            .matching(NSPredicate(format: "identifier BEGINSWITH 'conflict.useOurs.'"))
            .firstMatch
        XCTAssertTrue(useOurs.waitForExistence(timeout: 10), "Expected conflict bar from fixture")
        useOurs.click()
    }

    func testConflictFileShowsDiff() throws {
        let app = try XCTUnwrap(app)
        let file = app.descendants(matching: .any)[AID.FileList.row("file.txt")]
        XCTAssertTrue(file.waitForExistence(timeout: 10), "Expected conflicted file row")
        file.click()

        let diff = app.descendants(matching: .any)[AID.Diff.section]
        XCTAssertTrue(diff.waitForExistence(timeout: 5), "Diff section did not appear")
    }
}
