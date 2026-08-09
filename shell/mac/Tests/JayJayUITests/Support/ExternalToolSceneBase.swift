import XCTest

class ExternalToolSceneBase: SceneBase {
    override class var opensFixtureOnLaunch: Bool {
        false
    }

    override func setUpWithError() throws {
        try super.setUpWithError()
        let app = try XCTUnwrap(app)
        let singleWindow = NSPredicate { _, _ in app.windows.count == 1 }
        XCTAssertEqual(
            XCTWaiter().wait(
                for: [XCTNSPredicateExpectation(predicate: singleWindow, object: nil)],
                timeout: 5
            ),
            .completed,
            "External tool should open exactly one window"
        )
    }
}
