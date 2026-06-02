import XCTest

final class ConflictResolutionScene: SceneBase {
    override class var fixtureName: String {
        "conflict"
    }

    func testConflictFileShowsDiff() throws {
        let app = try XCTUnwrap(app)
        let file = fileRows(of: app)
            .matching(NSPredicate(format: "identifier == %@", AID.FileList.row("file.txt")))
            .firstMatch
        XCTAssertTrue(file.waitForExistence(timeout: 10), "Expected conflicted file row")
        file.click()

        let diff = app.descendants(matching: .any)[AID.Diff.section]
        XCTAssertTrue(diff.waitForExistence(timeout: 5), "Diff section did not appear")
    }
}
