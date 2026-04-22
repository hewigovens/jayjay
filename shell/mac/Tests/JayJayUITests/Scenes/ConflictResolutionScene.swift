import XCTest

final class ConflictResolutionScene: SceneBase {
    override class var fixtureName: String { "conflict" }

    func testUseOurs() throws {
        let app = try XCTUnwrap(app)
        let useOurs = app.buttons
            .matching(NSPredicate(format: "identifier BEGINSWITH 'conflict.useOurs.'"))
            .firstMatch
        XCTAssertTrue(useOurs.waitForExistence(timeout: 10), "Expected conflict bar from fixture")
        useOurs.click()
    }
}
