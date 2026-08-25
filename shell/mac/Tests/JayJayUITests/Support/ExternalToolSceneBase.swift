import XCTest

class ExternalToolSceneBase: SceneBase {
    override class var opensFixtureOnLaunch: Bool {
        false
    }

    /// Tool mode parses the whole command line and opens no repository window; masking arguments would abort it.
    override class var startsWithDefaultLayout: Bool {
        false
    }

    override func setUpWithError() throws {
        try super.setUpWithError()
        let app = try XCTUnwrap(app)
        XCTAssertTrue(waitForWindowCount(1, in: app), "External tool should open exactly one window")
    }
}
