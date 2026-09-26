import XCTest

/// Captures docs/imgs/overview from the scripts/demo-fixture.sh repository, whose agent workspaces give Repo Overview several lanes.
final class OverviewScreenshots: ScreenshotScene {
    override class var fixtureRoot: URL {
        URL(fileURLWithPath: environment["JAYJAY_DEMO_ROOT"] ?? "/tmp/jayjay-demo", isDirectory: true)
    }

    override class var launchEnvironment: [String: String] {
        super.launchEnvironment.merging(
            ["JAYJAY_REVIEW_STORE_PATH": fixtureRoot.appendingPathComponent("review-store.json").path]
        ) { $1 }
    }

    func testOverview() throws {
        let app = try XCTUnwrap(app)
        keyStroke("o", modifiers: [.command, .shift])
        let anyLane = NSPredicate(format: "identifier BEGINSWITH %@", AID.Overview.lane(""))
        let lanes = app.descendants(matching: .any).matching(anyLane)
        XCTAssertTrue(lanes.firstMatch.waitForExistence(timeout: 10), "Repo Overview did not open")
        let faresId = Self.fixtureRowId(for: "round fare estimates")
        let fares = try XCTUnwrap(
            lanes.allElementsBoundByIndex.first { faresId.hasPrefix($0.identifier.dropFirst(AID.Overview.lane("").count)) },
            "fares lane missing"
        )
        fares.click()
        keyStroke(.downArrow)
        XCTAssertTrue(app.descendants(matching: .any)[AID.Overview.changePanel].waitForExistence(timeout: 5), "Change panel missing")
        capture("overview", frame: app.windows.containing(anyLane).firstMatch.frame)
    }
}
