import XCTest

/// Runs with state restoration active, as users launch: the launch route must win over whatever SwiftUI restores.
final class LaunchRoutingRestoringScene: SceneBase {
    override class var ignoresWindowRestoration: Bool {
        false
    }

    func testLaunchRepositoryOpensAloneWithRestorationActive() throws {
        let app = try XCTUnwrap(app)
        if !app.windows["simple"].waitForExistence(timeout: 10) {
            let windows = app.windows.allElementsBoundByIndex.map { "\($0.title)|\($0.identifier)" }
            let texts = app.staticTexts.allElementsBoundByIndex.prefix(8).map(\.label)
            XCTFail("Launch repository window missing; windows=\(windows) texts=\(texts) state=\(app.state.rawValue)")
        }
        XCTAssertTrue(dagRows(of: app).firstMatch.waitForExistence(timeout: 10), "Launch repository never loaded")
        XCTAssertFalse(app.staticTexts["Recent Repositories"].exists, "Repository list opened beside the launch repository")
        XCTAssertEqual(app.windows.count, 1, "Restoration opened extra windows: \(app.windows.allElementsBoundByIndex.map(\.title))")
    }
}
