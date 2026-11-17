import XCTest

/// `open -n JayJay.app --args <path>` passes the repository without `--repo`; AppKit treats such an argument as a document to open.
final class LaunchBarePathScene: SceneBase {
    override class var opensFixtureOnLaunch: Bool {
        false
    }

    override class var additionalLaunchArguments: [String] {
        [fixtureRoot.appendingPathComponent(fixtureName, isDirectory: true).path]
    }

    func testBarePathOpensTheRepositoryWindow() throws {
        let app = try XCTUnwrap(app)
        XCTAssertTrue(
            app.windows["simple"].waitForExistence(timeout: 10),
            "No repository window for a bare launch path; windows=\(app.windows.allElementsBoundByIndex.map(\.title))"
        )
        XCTAssertTrue(dagRows(of: app).firstMatch.waitForExistence(timeout: 10), "The launch repository never loaded")
    }
}
